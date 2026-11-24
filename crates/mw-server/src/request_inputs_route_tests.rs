//! t50hp-e22 (W-2 and the realistic-rate gate): published request inputs on
//! the cookie-authenticated GET reads of `request_inputs::WARM_READS`, over the
//! real router, Store and Engine of the native admission fixture (SQLite).
//!
//! Route table (from the code; see `request_inputs::WARM_READS`):
//! | route | auth | reads beyond the common chain | frame consumers |
//! |---|---|---|---|
//! | /api/account/preferences | authed | storage owner, the document | session, gate, subject |
//! | /api/account/appearance | authed | storage owner, locks, deployment defaults | session, gate, subject (matches_session) |
//! | /api/account/appearance/customization | authed + accountId check | locks, owner, document | same |
//! | /api/account/notifications | authed | record owner, notification policy | same |
//! | /api/account/notification-center | authed | record owner, personal records | same |
//! | /api/settings | authed | layer rows, effective settings | same |
//! | /api/features | authed (selector refused) | effective features | same |
//! | /api/policy | authed | open tracking, admin security policy, client timing | same |
//! | /api/bootstrap | none itself; each section is its own request | none | middleware chain only |
//! Every read beyond the common chain is the handler's own statement, not
//! published DATA, so the publication needs to cover only the common chain.
//! Excluded: /api/accounts (child credentials, may renew child sessions and
//! write audit rows), write methods, the selector, Authorization/API keys,
//! queries other than the customization's `accountId`, sign-in and console.
use super::*;
use crate::request_inputs::test_control as inputs_control;
use std::net::SocketAddr;

const PEER: &str = "127.0.0.1:48213";

struct Got {
    status: StatusCode,
    body: Value,
    inputs: inputs_control::Counts,
    sql: mw_store::pool_telemetry::QuerySnapshot,
}

fn request(f: &Fixture, method: &str, uri: &str, body: Option<Value>) -> Request<Body> {
    let csrf = f.writer.session_csrf_token(&f.session);
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("host", "localhost")
        .header("origin", "http://localhost")
        .header("user-agent", "Mozilla/5.0")
        .header(
            "cookie",
            format!("mw_session={}; mw_csrf={csrf}", f.session),
        )
        .header("x-csrf-token", csrf);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    let mut request = builder
        .body(body.map_or_else(Body::empty, |body| {
            Body::from(serde_json::to_vec(&body).unwrap())
        }))
        .unwrap();
    request.extensions_mut().insert(axum::extract::ConnectInfo(
        PEER.parse::<SocketAddr>().unwrap(),
    ));
    request
}
fn get(f: &Fixture, uri: &str) -> Request<Body> {
    request(f, "GET", uri, None)
}
/// The client's list batch: Email/query on the inbox and Email/get by result.
fn list_batch(f: &Fixture) -> Request<Body> {
    request(
        f,
        "POST",
        "/jmap/api",
        Some(json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [
                ["Email/query", {"accountId": f.account, "limit": 50}, "q"],
                ["Email/get", {"accountId": f.account, "#ids": {"resultOf": "q", "name": "Email/query", "path": "/ids"},
                    "properties": ["id", "subject", "receivedAt", "keywords", "mailboxIds"]}, "g"]
            ]
        })),
    )
}

async fn send(f: &Fixture, request: Request<Body>, enabled: bool) -> Got {
    let measured = inputs_control::measure(
        enabled,
        test_control::count(true, f.router.clone().oneshot(request)),
    );
    let (((result, _native), inputs), sql) =
        mw_store::pool_telemetry::measure_queries(measured).await;
    let response = result.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 8 * 1024 * 1024)
        .await
        .unwrap();
    Got {
        status,
        body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        inputs,
        sql,
    }
}
fn framed(got: &Got) -> bool {
    got.inputs.frames_ready == 1 && got.inputs.legacy_session_reads == 0
}

/// Requests with published inputs until the session's entry is published.
async fn ready(f: &Fixture) {
    for _ in 0..400 {
        let got = send(f, get(f, "/api/me"), true).await;
        assert_eq!(got.status, StatusCode::OK, "{}", got.body);
        if framed(&got) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("request inputs never published");
}

/// The state after the client was away `secs` (longer than the throttle): no
/// recent touch held in this process, the row `secs` old. `buffered: false`
/// is the original rule (no buffer).
async fn away(f: &Fixture, secs: i64, buffered: bool) {
    f.measured.disable_session_touch_buffer().await.unwrap();
    if buffered {
        assert!(crate::session_persistence::enable_touch_buffer(&f.measured));
    }
    let options = mw_store::encryption::connect_options(&f.db, f.db_key.as_ref(), false);
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::query("UPDATE sessions SET last_seen=?2 WHERE id=?1")
        .bind(&f.session)
        .bind((chrono::Utc::now() - chrono::Duration::seconds(secs)).to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
}

fn customization(f: &Fixture) -> String {
    format!(
        "/api/account/appearance/customization?accountId={}",
        f.account
    )
}
fn warm_uris(f: &Fixture) -> Vec<String> {
    crate::request_inputs::WARM_READS
        .iter()
        .filter(|path| **path != "/api/account/appearance/customization")
        .map(|path| path.to_string())
        .chain([customization(f)])
        .collect()
}

#[test]
fn route_eligibility_table() {
    use axum::http::Method;
    let ok = |m: &Method, p: &str, q: Option<&str>| crate::request_inputs::eligible_route(m, p, q);
    for path in crate::request_inputs::WARM_READS {
        assert!(ok(&Method::GET, path, None), "{path}");
        assert!(
            !ok(&Method::PUT, path, None)
                && !ok(&Method::POST, path, None)
                && !ok(&Method::DELETE, path, None),
            "{path}: writes"
        );
        assert!(
            !ok(&Method::GET, path, Some("mwLink=x")),
            "{path}: selector query"
        );
    }
    assert!(ok(&Method::GET, "/api/me", None) && ok(&Method::POST, "/jmap/api", None));
    let custom = "/api/account/appearance/customization";
    assert!(ok(&Method::GET, custom, Some("accountId=acct-1")));
    assert!(ok(&Method::GET, custom, Some("accountId=a%40b")));
    for query in [
        "accountId=a&mwLink=b",
        "mwLink=b&accountId=a",
        "mw%4Cink=b",
        "accountId=",
        "accountid=a",
        "accountId=a b",
        "x=1",
    ] {
        assert!(!ok(&Method::GET, custom, Some(query)), "{query}");
    }
    for path in [
        "/api/accounts",
        "/api/login",
        "/api/logout",
        "/api/signin/challenge",
        "/api/login/2fa",
        "/api/account/2fa",
        "/api/sso/providers",
        "/admin",
        "/admin/api/users",
        "/api/account/sessions",
        "/api/account/security/history",
        "/api/public-config",
        "/jmap/upload/x",
        "/api/account/drafts",
    ] {
        assert!(!ok(&Method::GET, path, None), "{path}");
    }
    assert!(
        !ok(&Method::GET, "/api/me", Some("x=1")) && !ok(&Method::POST, "/jmap/api", Some("x=1"))
    );
}

/// Each newly eligible route runs on published inputs while the session's
/// touch is due, and answers exactly as the original readers do.
#[test]
fn warm_reads_use_published_inputs_with_the_same_answer() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        ready(&f).await;
        for uri in warm_uris(&f) {
            if uri == "/api/bootstrap" {
                continue; // sections run in tasks of their own: next test
            }
            let original = send(&f, get(&f, &uri), false).await;
            away(&f, 60, true).await;
            let published = send(&f, get(&f, &uri), true).await;
            assert!(
                framed(&published),
                "{uri}: {:?} {}",
                published.inputs,
                published.body
            );
            assert_eq!(published.status, original.status, "{uri}");
            assert_eq!(published.body, original.body, "{uri}");
            assert_eq!(
                published.sql.write.calls, 0,
                "{uri}: the due touch is not written by the request"
            );
            assert!(
                f.measured.buffered_last_seen(&f.session).is_some(),
                "{uri}: touch recorded"
            );
        }
    });
}

/// The bootstrap's sections are requests of their own: with published inputs
/// they install their own frames, and the document is unchanged.
#[test]
fn bootstrap_sections_use_published_inputs() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        ready(&f).await;
        let before = mw_store::pool_telemetry::process_snapshot();
        let off = send(&f, get(&f, "/api/bootstrap"), false).await;
        let off_sql = mw_store::pool_telemetry::process_snapshot();
        assert_eq!(off.status, StatusCode::OK, "{}", off.body);
        away(&f, 60, true).await;
        let start = mw_store::pool_telemetry::process_snapshot();
        let (on, frames) =
            inputs_control::process_wide(send(&f, get(&f, "/api/bootstrap"), true)).await;
        let end = mw_store::pool_telemetry::process_snapshot();
        assert_eq!(on.status, StatusCode::OK);
        let sections = |body: &Value| {
            body["sections"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(name, section)| {
                    (
                        name.clone(),
                        section["status"].clone(),
                        section["body"].clone(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(sections(&on.body), sections(&off.body), "same sections");
        // me + the eligible sections (jmapSession only in bounded mode) + outer.
        let eligible = crate::bootstrap::SECTIONS
            .iter()
            .filter(|section| {
                crate::request_inputs::WARM_READS.contains(&section.path)
                    || section.path == "/api/me"
            })
            .count() as u64;
        assert!(frames >= eligible, "frames {frames} < {eligible}");
        let statements = |a: &mw_store::pool_telemetry::QuerySnapshot,
                          b: &mw_store::pool_telemetry::QuerySnapshot| {
            (
                b.read.calls - a.read.calls,
                b.write.calls - a.write.calls,
                b.other.calls - a.other.calls,
            )
        };
        let (off_reads, off_writes, off_other) = statements(&before, &off_sql);
        let (on_reads, on_writes, on_other) = statements(&start, &end);
        eprintln!(
            "{}",
            json!({"kind":"t50hp-e22-bootstrap-statements","backend":"sqlite-off",
            "original":{"reads":off_reads,"writes":off_writes,"other":off_other},
            "published":{"reads":on_reads,"writes":on_writes,"other":on_other},"frames":frames})
        );
        assert!(
            on_reads < off_reads,
            "fewer reads: {on_reads} vs {off_reads}"
        );
    });
}

/// Excluded classes keep the original readers.
#[test]
fn excluded_routes_keep_the_original_readers() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        ready(&f).await;
        let mut cases: Vec<(&str, Request<Body>)> = vec![
            ("linked accounts", get(&f, "/api/accounts")),
            ("sign-in challenge", get(&f, "/api/signin/challenge")),
            ("account 2fa", get(&f, "/api/account/2fa")),
            ("sessions list", get(&f, "/api/account/sessions")),
            ("console", get(&f, "/admin/api/users")),
            (
                "write",
                request(&f, "PUT", "/api/account/preferences", Some(json!({}))),
            ),
            (
                "selector query",
                get(
                    &f,
                    &format!("{}&mwLink={}", customization(&f), uuid::Uuid::new_v4()),
                ),
            ),
            ("other query", get(&f, "/api/settings?x=1")),
        ];
        let mut selector = get(&f, "/api/settings");
        selector.headers_mut().insert(
            crate::linked_accounts::SELECTOR,
            uuid::Uuid::new_v4().to_string().parse().unwrap(),
        );
        cases.push(("selector header", selector));
        let mut bearer = get(&f, "/api/settings");
        bearer
            .headers_mut()
            .insert("authorization", "Bearer not-a-token".parse().unwrap());
        cases.push(("authorization", bearer));
        let mut key = get(&f, "/api/policy");
        key.headers_mut()
            .insert("x-api-key", "not-a-key".parse().unwrap());
        cases.push(("api key", key));
        for (label, request) in cases {
            away(&f, 60, true).await;
            let got = send(&f, request, true).await;
            assert_eq!(
                got.inputs.frames_ready, 0,
                "{label}: {} {:?}",
                got.status, got.inputs
            );
            assert_eq!(got.inputs.first_fences, 0, "{label}: no published lookup");
        }
    });
}

/// The realistic-rate gate, simulated: every request is a client's first in
/// 30 s (no touch held in this process younger than the 15 s throttle, the row
/// 30 s old), on identity, the list batch and the polls. Original rule: such a
/// request always took the original path.
#[test]
fn realistic_rate_requests_use_published_inputs() {
    run_fixture(|| async {
        let f = Fixture::new().await;
        ready(&f).await;
        let polls = [
            "/api/account/preferences",
            "/api/account/appearance",
            "/api/account/notifications",
            "/api/account/notification-center",
            "/api/settings",
            "/api/features",
            "/api/policy",
        ];
        let mut report = serde_json::Map::new();
        for buffered in [false, true] {
            let mut groups = vec![
                ("identity", 0u32, 0u32),
                ("listBatch", 0, 0),
                ("polls", 0, 0),
            ];
            for _visit in 0..10 {
                for (index, label) in ["identity", "listBatch", "polls"].iter().enumerate() {
                    let requests: Vec<Request<Body>> = match *label {
                        "identity" => vec![get(&f, "/api/me")],
                        "listBatch" => vec![list_batch(&f)],
                        _ => polls
                            .iter()
                            .map(|uri| get(&f, uri))
                            .chain([get(&f, &customization(&f))])
                            .collect(),
                    };
                    for request in requests {
                        away(&f, 30, buffered).await;
                        let got = send(&f, request, true).await;
                        assert_eq!(got.status, StatusCode::OK, "{label}: {}", got.body);
                        groups[index].1 += 1;
                        if framed(&got) {
                            groups[index].2 += 1;
                        }
                    }
                }
            }
            for (label, total, used) in &groups {
                let ratio = f64::from(*used) / f64::from(*total);
                report.insert(
                    format!("{label}{}", if buffered { "Buffered" } else { "Original" }),
                    json!({"requests": total, "framesUsed": used, "ratio": ratio}),
                );
                if buffered {
                    assert!(ratio >= 0.95, "{label}: frames used {used}/{total}");
                } else {
                    assert_eq!(
                        *used, 0,
                        "{label}: original rule, a due touch takes the original path"
                    );
                }
            }
        }
        assert!(crate::session_persistence::enable_touch_buffer(&f.measured));
        eprintln!(
            "{}",
            json!({"kind":"t50hp-e22-realistic-rate","simulatedSecondsBetweenRequests":30,"groups":report})
        );
    });
}

/// Reads per warm request on published inputs, touch due, SQLite (measured
/// 2026-11-24; original readers in parentheses). Upper bounds.
const WARM_BUDGETS: &[(&str, u64)] = &[
    ("GET /api/me", 11),                               // (17)
    ("POST /jmap/api list batch", 18),                 // (40)
    ("GET /api/account/preferences", 12),              // (18)
    ("GET /api/account/appearance", 44),               // (62)
    ("GET /api/account/notifications", 42),            // (52)
    ("GET /api/account/notification-center", 13),      // (19)
    ("GET /api/settings", 15),                         // (23)
    ("GET /api/features", 11),                         // (19)
    ("GET /api/policy", 14),                           // (22)
    ("GET /api/account/appearance/customization", 39), // (55)
];

/// Statements per warm request (reads, writes), original readers versus
/// published inputs on a touch-due session. Budgets are upper bounds.
#[test]
fn warm_request_statement_budgets() {
    run_fixture(|| async {
        let fixture = Fixture::new().await;
        let f = &fixture;
        ready(f).await;
        let mut routes: Vec<(String, Box<dyn Fn() -> Request<Body> + '_>)> = vec![
            ("GET /api/me".into(), Box::new(move || get(f, "/api/me"))),
            (
                "POST /jmap/api list batch".into(),
                Box::new(move || list_batch(f)),
            ),
        ];
        for uri in warm_uris(f)
            .into_iter()
            .filter(|uri| uri != "/api/bootstrap")
        {
            let label = format!("GET {}", uri.split('?').next().unwrap());
            routes.push((label, Box::new(move || get(f, &uri))));
        }
        for (label, make) in &routes {
            for _ in 0..3 {
                assert_eq!(
                    send(f, make(), false).await.status,
                    StatusCode::OK,
                    "{label}"
                );
            }
            let original = send(f, make(), false).await;
            away(f, 60, true).await;
            let published = send(f, make(), true).await;
            assert!(framed(&published), "{label}");
            eprintln!(
                "{}",
                json!({"kind":"t50hp-e22-warm-statements","backend":"sqlite","route":label,
                "original":{"reads":original.sql.read.calls,"writes":original.sql.write.calls},
                "published":{"reads":published.sql.read.calls,"writes":published.sql.write.calls}})
            );
            assert_eq!(
                published.sql.write.calls, 0,
                "{label}: no write on a due touch"
            );
            let budget = WARM_BUDGETS
                .iter()
                .find(|(route, _)| route == label)
                .map(|(_, reads)| *reads)
                .unwrap_or_else(|| panic!("{label}: no budget"));
            assert!(
                published.sql.read.calls <= budget,
                "{label}: {} reads > budget {budget}",
                published.sql.read.calls
            );
            assert!(
                published.sql.read.calls < original.sql.read.calls,
                "{label}: published {} reads >= original {}",
                published.sql.read.calls,
                original.sql.read.calls
            );
        }
    });
}

/// e15a follow-up: after each revocation kind the next /api/bootstrap answers
/// within a time bound and a statement bound, and no authenticated section
/// answers with data. (The reviewer's run that did not finish used the
/// t49pg fixture, whose account points at 127.0.0.1:9: on this host the
/// Windows "Simple TCP/IP Services" discard port accepts and never greets, so
/// the start-up registration held the account's connect lock; see
/// logs/t50hp-e22.md. This fixture registers a backend, as production does
/// after sign-in.)
async fn bootstrap_after_revocation(kind: &'static str) {
    let f = Fixture::new().await;
    ready(&f).await;
    let (before, _) = inputs_control::process_wide(send(&f, get(&f, "/api/bootstrap"), true)).await;
    assert_eq!(before.status, StatusCode::OK, "{kind}");
    assert_eq!(
        before.body["sections"]["me"]["status"], 200,
        "{kind}: {}",
        before.body
    );
    match kind {
        "sign-out" => {
            let out = send(&f, request(&f, "POST", "/api/logout", None), true).await;
            assert!(
                out.status.is_success(),
                "{kind}: {} {}",
                out.status,
                out.body
            );
        }
        "session-revoke" => {
            f.measured.delete_session(&f.session).await.unwrap();
        }
        "account-disable" => {
            crate::account_gate::update_flags(&f.measured, &f.account, |flags| {
                flags.disabled = true
            })
            .await
            .unwrap();
        }
        "password-change" => {
            crate::account_gate::update_flags(&f.measured, &f.account, |flags| {
                flags.force_password_change = true
            })
            .await
            .unwrap();
        }
        _ => unreachable!(),
    }
    let start = mw_store::pool_telemetry::process_snapshot();
    let started = std::time::Instant::now();
    let (after, frames) = tokio::time::timeout(
        Duration::from_secs(30),
        inputs_control::process_wide(send(&f, get(&f, "/api/bootstrap"), true)),
    )
    .await
    .unwrap_or_else(|_| {
        panic!("{kind}: bootstrap after the revocation did not answer within 30 s")
    });
    let elapsed = started.elapsed();
    let end = mw_store::pool_telemetry::process_snapshot();
    let statements = (end.read.calls - start.read.calls)
        + (end.write.calls - start.write.calls)
        + (end.other.calls - start.other.calls);
    eprintln!(
        "{}",
        json!({"kind":"t50hp-e22-bootstrap-after-revocation","revocation":kind,
        "ms":elapsed.as_millis() as u64,"statements":statements,"frames":frames,"status":after.status.as_u16()})
    );
    assert_eq!(after.status, StatusCode::OK, "{kind}: the document itself");
    assert!(elapsed < Duration::from_secs(10), "{kind}: {elapsed:?}");
    assert!(
        statements <= BOOTSTRAP_AFTER_REVOCATION_STATEMENTS,
        "{kind}: {statements} statements"
    );
    for (name, section) in after.body["sections"].as_object().unwrap() {
        let status = section["status"].as_u64().unwrap();
        let public = matches!(
            name.as_str(),
            "publicConfig"
                | "cookiesConfig"
                | "version"
                | "uiPlugins"
                | "assistConfig"
                | "restriction"
        );
        if public {
            continue;
        }
        if kind == "password-change" && name == "me" {
            assert_eq!(status, 200, "{kind}: me");
            assert_eq!(
                section["body"]["passwordChangeRequired"], true,
                "{kind}: {section}"
            );
            continue;
        }
        assert!(
            status == 401 || status == 403 || (name == "appearanceCustomization" && status == 409),
            "{kind}: section {name} answered {status}: {section}"
        );
        assert!(
            section["body"].get("accountId").is_none(),
            "{kind}: {name} carried data"
        );
    }
}

/// Upper bound on statements for one /api/bootstrap after a revocation
/// (process-wide count; measured below it, see the eprintln line).
const BOOTSTRAP_AFTER_REVOCATION_STATEMENTS: u64 = 300; // measured 180-238

#[test]
fn bootstrap_after_sign_out_is_bounded() {
    run_fixture(|| async { bootstrap_after_revocation("sign-out").await });
}
#[test]
fn bootstrap_after_session_revoke_is_bounded() {
    run_fixture(|| async { bootstrap_after_revocation("session-revoke").await });
}
#[test]
fn bootstrap_after_account_disable_is_bounded() {
    run_fixture(|| async { bootstrap_after_revocation("account-disable").await });
}
#[test]
fn bootstrap_after_password_change_is_bounded() {
    run_fixture(|| async { bootstrap_after_revocation("password-change").await });
}
