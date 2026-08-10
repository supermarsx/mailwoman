//! Behaviour tests for the mock JMAP surface (t19-e9, coverage fill A).
//!
//! `mw-mock-jmap` is test infrastructure, so the risk it carries is unusual: when
//! the mock is *too permissive*, the suites that drive it pass for the wrong
//! reason. If the mock served data without checking `Authorization`, every
//! mw-server test that claims to prove "the proxy injects upstream credentials"
//! would go green with no credentials injected at all. So the auth refusals below
//! are not politeness — they are what makes the proxy assertions mean anything.
//!
//! The rest pins the shapes other crates read: the RFC 8620 §3.6.1 error envelope,
//! result-reference resolution (§3.7), the download/upload echoes the proxy tests
//! assert on, and the frozen V4 crypto/security fixture families that
//! `mw-engine/tests/security.rs` compares the real engine against.
//!
//! Kept in `tests/` rather than as an inline `#[cfg(test)] mod tests` on purpose:
//! `docs/testing/coverage.md` records that inline modules land in `llvm-cov`'s
//! denominator, so an integration test raises the measured figure for the same
//! work instead of diluting it.

use std::net::SocketAddr;

use base64::Engine as _;
use mw_mock_jmap::{ACCOUNT_ID, PASS, USER, router, security_case};
use serde_json::{Value, json};

async fn spawn() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router()).await.unwrap();
    });
    addr
}

fn auth() -> String {
    format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!("{USER}:{PASS}"))
    )
}

/// POST a JMAP request with valid auth and return the parsed response.
async fn call(addr: SocketAddr, calls: Value) -> Value {
    reqwest::Client::new()
        .post(format!("http://{addr}/jmap"))
        .header("authorization", auth())
        .json(&json!({ "using": ["urn:ietf:params:jmap:mail"], "methodCalls": calls }))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()
}

// ── auth: the mock must never serve without credentials ────────────────────────

/// Every route that carries data refuses an unauthenticated request. A mock that
/// answered anyway would make the proxy's credential-injection tests vacuous.
#[tokio::test]
async fn every_data_route_refuses_unauthenticated_requests() {
    let addr = spawn().await;
    let c = reqwest::Client::new();

    for path in [
        "/.well-known/jmap",
        "/jmap/session",
        "/jmap/download/acct-1/blob-e1/report.pdf",
    ] {
        let resp = c.get(format!("http://{addr}{path}")).send().await.unwrap();
        assert_eq!(resp.status(), 401, "GET {path} must require auth");
    }

    let resp = c
        .post(format!("http://{addr}/jmap"))
        .json(&json!({ "methodCalls": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "POST /jmap must require auth");

    let resp = c
        .post(format!("http://{addr}/jmap/upload/acct-1"))
        .body("bytes")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "POST /jmap/upload must require auth");
}

/// `/healthz` is deliberately open — a container health check has no credentials.
/// Pinned so nobody "fixes" it into an auth-gated route and breaks compose.
#[tokio::test]
async fn healthz_is_open_by_design() {
    let addr = spawn().await;
    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/healthz"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "ok");
}

/// Each way of getting Basic auth wrong is refused: absent header, a non-Basic
/// scheme, undecodable base64, and correctly-encoded wrong credentials.
#[tokio::test]
async fn malformed_or_wrong_credentials_are_refused() {
    let addr = spawn().await;
    let c = reqwest::Client::new();
    let wrong = base64::engine::general_purpose::STANDARD.encode(format!("{USER}:not-the-pass"));

    let cases: [(&str, Option<String>); 5] = [
        ("no authorization header", None),
        ("bearer token, not Basic", Some("Bearer abc123".into())),
        ("Basic with undecodable base64", Some("Basic !!!!".into())),
        ("Basic with wrong password", Some(format!("Basic {wrong}"))),
        ("empty Basic payload", Some("Basic ".into())),
    ];

    for (why, header) in cases {
        let mut req = c.get(format!("http://{addr}/jmap/session"));
        if let Some(h) = header {
            req = req.header("authorization", h);
        }
        assert_eq!(req.send().await.unwrap().status(), 401, "{why}");
    }
}

// ── the session document ───────────────────────────────────────────────────────

/// The session advertises the two Mailwoman capability URNs both globally and on
/// the account, and both aliases (`/.well-known/jmap` and `/jmap/session`) return
/// the same document. The web client reads `primaryAccounts` to pick an account.
#[tokio::test]
async fn session_advertises_the_mailwoman_capability_urns() {
    let addr = spawn().await;
    let c = reqwest::Client::new();

    let mut docs = Vec::new();
    for path in ["/.well-known/jmap", "/jmap/session"] {
        docs.push(
            c.get(format!("http://{addr}{path}"))
                .header("authorization", auth())
                .send()
                .await
                .unwrap()
                .json::<Value>()
                .await
                .unwrap(),
        );
    }
    assert_eq!(docs[0], docs[1], "both session aliases serve one document");

    let s = &docs[0];
    for urn in ["urn:mailwoman:crypto", "urn:mailwoman:security"] {
        assert!(s["capabilities"][urn].is_object(), "missing {urn}");
        assert!(
            s["accounts"][ACCOUNT_ID]["accountCapabilities"][urn].is_object(),
            "account is missing {urn}"
        );
        assert_eq!(s["primaryAccounts"][urn], ACCOUNT_ID);
    }
    assert_eq!(s["username"], USER);
    assert_eq!(s["apiUrl"], "/jmap");
}

// ── RFC 8620 §3.6.1: method errors use the "error" name ────────────────────────

/// A method the mock does not implement comes back as `["error", {type}, callId]`,
/// NOT `["<Method>", …]`. Clients key their error handling on `found[0] == "error"`,
/// so tagging it with the method name would look like success.
#[tokio::test]
async fn unimplemented_method_returns_the_error_envelope() {
    let addr = spawn().await;
    let res = call(addr, json!([["Frobnicate/get", {}, "c9"]])).await;
    let r = &res["methodResponses"][0];

    assert_eq!(r[0], "error", "method errors carry the `error` name");
    assert_eq!(r[1]["type"], "unknownMethod");
    assert_eq!(r[2], "c9", "the call id is echoed");
}

/// Malformed calls are skipped, not fatal: a non-array entry and a too-short array
/// produce no response, while a valid call in the same batch still answers.
#[tokio::test]
async fn malformed_method_calls_are_skipped_without_dropping_the_batch() {
    let addr = spawn().await;
    let res = call(
        addr,
        json!([
            "not-an-array",
            [["Email/query"]],
            ["Email/query", { "filter": { "inMailbox": "mb-inbox" } }, "good"]
        ]),
    )
    .await;

    let responses = res["methodResponses"].as_array().unwrap();
    assert_eq!(responses.len(), 1, "only the well-formed call answers");
    assert_eq!(responses[0][2], "good");
}

/// A request body that is not JSON is a 400, and a request with no `methodCalls`
/// key is an empty (but well-formed) response rather than an error.
#[tokio::test]
async fn bad_body_is_400_and_missing_method_calls_is_empty() {
    let addr = spawn().await;
    let c = reqwest::Client::new();

    let resp = c
        .post(format!("http://{addr}/jmap"))
        .header("authorization", auth())
        .body("{not json")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    let res = c
        .post(format!("http://{addr}/jmap"))
        .header("authorization", auth())
        .json(&json!({ "using": [] }))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(res["methodResponses"], json!([]));
    assert_eq!(res["sessionState"], "session-0");
}

// ── RFC 8620 §3.7: result references ───────────────────────────────────────────

/// A reference naming a call id that never ran is DROPPED — the `#`-prefixed key
/// is removed and no un-prefixed key is invented. `Email/get` then sees no `ids`
/// and returns an empty list. Documented behaviour; previously unexercised.
#[tokio::test]
async fn unresolvable_result_reference_is_dropped() {
    let addr = spawn().await;
    let res = call(
        addr,
        json!([[
            "Email/get",
            { "#ids": { "resultOf": "never-ran", "name": "Email/query", "path": "/ids" } },
            "g"
        ]]),
    )
    .await;

    let args = &res["methodResponses"][0][1];
    assert_eq!(
        args["list"],
        json!([]),
        "no ids resolved ⇒ nothing returned"
    );
    assert_eq!(args["notFound"], json!([]));
}

/// A reference whose `path` does not exist in the referenced response is likewise
/// dropped rather than resolving to null.
#[tokio::test]
async fn result_reference_with_a_bad_path_is_dropped() {
    let addr = spawn().await;
    let res = call(
        addr,
        json!([
            ["Email/query", { "filter": { "inMailbox": "mb-inbox" } }, "q"],
            [
                "Email/get",
                { "#ids": { "resultOf": "q", "name": "Email/query", "path": "/nope" } },
                "g"
            ]
        ]),
    )
    .await;
    assert_eq!(res["methodResponses"][1][1]["list"], json!([]));
}

// ── the mail store ─────────────────────────────────────────────────────────────

/// `Mailbox/get` counts are computed from the live store, not hardcoded: sending a
/// draft moves it into Sent and both counts follow. Pins `count_in`, which the
/// seeded-fixture assertions elsewhere would not distinguish from a constant.
#[tokio::test]
async fn mailbox_counts_follow_the_store_after_a_send() {
    let addr = spawn().await;

    let before = call(addr, json!([["Mailbox/get", {}, "m"]])).await;
    let list = before["methodResponses"][0][1]["list"].as_array().unwrap();
    let count = |list: &[Value], id: &str| -> u64 {
        list.iter()
            .find(|m| m["id"] == id)
            .and_then(|m| m["totalEmails"].as_u64())
            .unwrap()
    };
    assert_eq!(count(list, "mb-inbox"), 3, "three seeded inbox messages");
    assert_eq!(count(list, "mb-sent"), 0);

    call(
        addr,
        json!([
            ["Email/set", { "create": { "d": {
                "mailboxIds": { "mb-drafts": true },
                "from": [{ "email": USER }],
                "to": [{ "email": USER }],
                "subject": "counts"
            } } }, "c1"],
            ["EmailSubmission/set", { "create": { "s": { "emailId": "#d" } } }, "c2"]
        ]),
    )
    .await;

    let after = call(addr, json!([["Mailbox/get", {}, "m"]])).await;
    let list = after["methodResponses"][0][1]["list"].as_array().unwrap();
    assert_eq!(
        count(list, "mb-sent"),
        1,
        "the submitted draft landed in Sent"
    );
    assert_eq!(count(list, "mb-inbox"), 3, "Inbox is untouched by a send");
}

/// `Email/get` separates hits from misses: known ids come back in `list`, unknown
/// ids in `notFound`. A mock that silently dropped misses would hide a client bug
/// that requests stale ids.
#[tokio::test]
async fn email_get_reports_unknown_ids_as_not_found() {
    let addr = spawn().await;
    let res = call(
        addr,
        json!([["Email/get", { "ids": ["e1", "ghost"] }, "g"]]),
    )
    .await;
    let args = &res["methodResponses"][0][1];

    assert_eq!(args["list"].as_array().unwrap().len(), 1);
    assert_eq!(args["list"][0]["id"], "e1");
    assert_eq!(args["notFound"], json!(["ghost"]));
}

/// `Email/query` filters by mailbox and defaults to the Inbox when no filter is
/// given, so a query for an empty mailbox returns an empty (not seeded) list.
#[tokio::test]
async fn email_query_filters_by_mailbox_and_defaults_to_inbox() {
    let addr = spawn().await;

    let defaulted = call(addr, json!([["Email/query", {}, "q"]])).await;
    assert_eq!(defaulted["methodResponses"][0][1]["total"], 3);

    let sent = call(
        addr,
        json!([["Email/query", { "filter": { "inMailbox": "mb-sent" } }, "q"]]),
    )
    .await;
    assert_eq!(sent["methodResponses"][0][1]["ids"], json!([]));
    assert_eq!(sent["methodResponses"][0][1]["total"], 0);
}

/// A draft created with no `mailboxIds` is filed into Drafts, and an
/// `EmailSubmission/set` naming a REAL id (not a `#creation-id`) moves that
/// message — the non-reference branch of the submit path.
#[tokio::test]
async fn submission_by_real_id_moves_that_message_to_sent() {
    let addr = spawn().await;

    let created = call(
        addr,
        json!([["Email/set", { "create": { "d": { "subject": "no mailbox" } } }, "c1"]]),
    )
    .await;
    let id = created["methodResponses"][0][1]["created"]["d"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let in_drafts = call(
        addr,
        json!([["Email/query", { "filter": { "inMailbox": "mb-drafts" } }, "q"]]),
    )
    .await;
    assert_eq!(
        in_drafts["methodResponses"][0][1]["ids"],
        json!([id]),
        "a draft with no mailboxIds is filed into Drafts"
    );

    call(
        addr,
        json!([["EmailSubmission/set", { "create": { "s": { "emailId": id } } }, "c2"]]),
    )
    .await;

    let sent = call(
        addr,
        json!([["Email/query", { "filter": { "inMailbox": "mb-sent" } }, "q"]]),
    )
    .await;
    assert_eq!(sent["methodResponses"][0][1]["ids"], json!([id]));
}

// ── blob download / upload: the echoes the proxy tests assert on ───────────────

/// The download echoes its URL coordinates verbatim and sets an attachment
/// disposition. mw-server's proxy test reads this body to prove the request was
/// forwarded unchanged, so the format is a contract, not an implementation detail.
#[tokio::test]
async fn download_echoes_its_coordinates_with_an_attachment_disposition() {
    let addr = spawn().await;
    let resp = reqwest::Client::new()
        .get(format!(
            "http://{addr}/jmap/download/acct-9/blob-42/quarterly.pdf"
        ))
        .header("authorization", auth())
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["content-type"], "application/octet-stream");
    assert_eq!(
        resp.headers()["content-disposition"],
        "attachment; filename=\"quarterly.pdf\""
    );
    assert_eq!(
        resp.text().await.unwrap(),
        "BLOB:acct-9:blob-42:quarterly.pdf"
    );
}

/// The upload response echoes the byte count and the CLIENT-declared content type
/// — the two things a proxy test needs to prove the body and header crossed the
/// proxy unchanged.
#[tokio::test]
async fn upload_echoes_the_byte_count_and_client_content_type() {
    let addr = spawn().await;
    let body = vec![7u8; 1234];
    let res = reqwest::Client::new()
        .post(format!("http://{addr}/jmap/upload/acct-9"))
        .header("authorization", auth())
        .header("content-type", "application/pdf")
        .body(body)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    assert_eq!(res["accountId"], "acct-9");
    assert_eq!(res["blobId"], "upload-1234");
    assert_eq!(res["size"], 1234);
    assert_eq!(res["type"], "application/pdf");
}

/// With no `Content-Type` the upload falls back to `application/octet-stream`
/// rather than failing or echoing an empty type.
#[tokio::test]
async fn upload_without_a_content_type_falls_back_to_octet_stream() {
    let addr = spawn().await;
    let res = reqwest::Client::new()
        .post(format!("http://{addr}/jmap/upload/acct-1"))
        .header("authorization", auth())
        .body(Vec::<u8>::new())
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    assert_eq!(res["type"], "application/octet-stream");
    assert_eq!(res["size"], 0);
    assert_eq!(res["blobId"], "upload-0");
}

// ── the frozen V4 crypto/security fixtures ─────────────────────────────────────

/// `security_case` answers for exactly the frozen crypto/security family and
/// returns `None` for everything else — the `None` is what routes a method on to
/// the `unknownMethod` error, so a family accidentally dropped here degrades to a
/// client-visible error rather than a wrong shape.
///
/// `mw-engine/tests/security.rs` compares the real engine against three of these
/// (`CryptoKey/get`, `SecurityVerdict/get`, `MailRule/get`); the other ten had no
/// test naming them at all.
#[test]
fn security_case_covers_the_frozen_families_and_only_those() {
    let args = json!({});
    for name in [
        "CryptoKey/get",
        "CryptoKey/query",
        "CryptoKey/lookup",
        "CryptoKey/set",
        "CryptoKey/setTrust",
        "CryptoKey/changes",
        "SecurityVerdict/get",
        "SenderControl/set",
        "MailRule/get",
        "MailRule/set",
        "MailRule/changes",
        "Dlp/getRules",
        "Dlp/scan",
    ] {
        assert!(
            security_case(name, &args).is_some(),
            "{name} is part of the frozen family and must answer"
        );
    }

    for name in ["Email/get", "CryptoKey/destroy", "Dlp/", "", "MailRule"] {
        assert!(
            security_case(name, &args).is_none(),
            "{name:?} is not a crypto/security method and must fall through"
        );
    }
}

/// The `/get`-shaped and `/query`-shaped families carry the JMAP envelope keys
/// their real counterparts do, all scoped to the mock's single account.
#[test]
fn security_case_shapes_carry_the_jmap_envelope() {
    let args = json!({});
    for name in ["CryptoKey/get", "SecurityVerdict/get", "MailRule/get"] {
        let v = security_case(name, &args).unwrap();
        assert_eq!(v["accountId"], ACCOUNT_ID, "{name}");
        assert!(v["state"].is_string(), "{name} needs a state token");
        assert!(v["list"].is_array(), "{name}");
        assert_eq!(v["notFound"], json!([]), "{name}");
    }

    let q = security_case("CryptoKey/query", &args).unwrap();
    assert_eq!(q["ids"], json!(["key-pgp-1"]));
    assert_eq!(q["total"], 1);
    assert_eq!(q["canCalculateChanges"], false);

    for name in ["CryptoKey/set", "MailRule/set"] {
        let v = security_case(name, &args).unwrap();
        assert!(
            v["oldState"].is_string() && v["newState"].is_string(),
            "{name}"
        );
        assert_ne!(v["oldState"], v["newState"], "{name} must advance state");
    }

    for name in ["CryptoKey/changes", "MailRule/changes"] {
        let v = security_case(name, &args).unwrap();
        assert_eq!(v["hasMoreChanges"], false, "{name}");
        assert_eq!(v["oldState"], v["newState"], "{name} reports no changes");
    }
}

/// `SecurityVerdict/get` returns one verdict per requested id, and falls back to a
/// single verdict for `e1` when no ids are given. The per-id branch is what makes
/// a multi-message reader test meaningful.
#[test]
fn security_verdict_get_answers_per_requested_id() {
    let many = security_case("SecurityVerdict/get", &json!({ "ids": ["e2", "e3", "e7"] })).unwrap();
    let list = many["list"].as_array().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0]["emailId"], "e2");
    assert_eq!(list[2]["emailId"], "e7");

    let defaulted = security_case("SecurityVerdict/get", &json!({})).unwrap();
    assert_eq!(defaulted["list"].as_array().unwrap().len(), 1);
    assert_eq!(defaulted["list"][0]["emailId"], "e1");

    // A non-string entry in `ids` is skipped rather than stringified.
    let mixed = security_case("SecurityVerdict/get", &json!({ "ids": ["e2", 5, null] })).unwrap();
    assert_eq!(mixed["list"].as_array().unwrap().len(), 1);
}

/// The DLP fixtures ship the shapes the composer's block path reads: a rule with
/// a `block` action, and a clean scan returning no findings.
#[test]
fn dlp_fixtures_expose_a_block_rule_and_a_clean_scan() {
    let rules = security_case("Dlp/getRules", &json!({})).unwrap();
    let rule = &rules["list"][0];
    assert_eq!(rule["action"], "block");
    assert_eq!(rule["enabled"], true);
    assert_eq!(rule["conditions"]["detectors"], json!(["pan"]));

    let scan = security_case("Dlp/scan", &json!({})).unwrap();
    assert_eq!(scan["list"], json!([]), "a clean draft has no findings");
}

/// The crypto/security families are reachable through the HTTP surface too, not
/// only via the `security_case` helper the parity test calls directly.
#[tokio::test]
async fn crypto_family_is_dispatched_over_http() {
    let addr = spawn().await;
    let res = call(addr, json!([["CryptoKey/get", {}, "k"]])).await;
    let r = &res["methodResponses"][0];

    assert_eq!(r[0], "CryptoKey/get", "a known method keeps its own name");
    assert_eq!(r[1]["list"][0]["fingerprint"].as_str().unwrap().len(), 40);
    assert_eq!(r[1]["list"][0]["trust"], "verified");
}

// ── the hostile seed ───────────────────────────────────────────────────────────

/// The seeded `e2` body carries the sanitizer bait — a script, an inline event
/// handler, a `javascript:` href and a tracking pixel. If this seed were ever
/// softened, every "the sanitizer is wired" assertion downstream would still pass
/// while proving nothing.
#[tokio::test]
async fn the_seeded_hostile_body_still_carries_every_sanitizer_bait() {
    let addr = spawn().await;
    let res = call(addr, json!([["Email/get", { "ids": ["e2"] }, "g"]])).await;
    let body = res["methodResponses"][0][1]["list"][0]["bodyValues"]["1"]["value"]
        .as_str()
        .unwrap()
        .to_string();

    for bait in [
        "<script>",
        "onclick=",
        "javascript:alert(1)",
        "tracker.evil.example",
    ] {
        assert!(body.contains(bait), "hostile seed lost its {bait} bait");
    }
}
