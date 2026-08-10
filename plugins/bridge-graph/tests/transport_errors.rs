//! Error classification, request shape, and paging/cursor edges of the pure Graph
//! mapping — the paths the recorded-fixture suite in `tests/mapping.rs` cannot reach
//! because every fixture is a 200.
//!
//! These drive a **scripted** [`Transport`] (a programmable responder that also
//! records every request it was handed) rather than the fixture replayer, so a test
//! can pin:
//!
//! * how an HTTP status becomes a [`BridgeError`] variant — the guest maps that
//!   variant 1:1 onto the WIT `plugin-error`, so mis-classifying a 404 as a transport
//!   failure would change what the engine does about it;
//! * that a request is only ever issued **with** a freshly-acquired bearer token, and
//!   never at all when the token could not be acquired;
//! * that an absolute `@odata.deltaLink`/`nextLink` is used verbatim and a relative
//!   path is resolved against `GRAPH_BASE` — re-basing a deltaLink would silently
//!   restart every sync from scratch;
//! * that a flag mutation Graph cannot express issues **no** HTTP request at all.
//!
//! Everything here is host-target pure Rust: no wasm toolchain, no live tenant.

use std::cell::RefCell;

use bridge_graph::caps;
use bridge_graph::graph::{BridgeError, GraphClient, HttpRequestSpec, HttpResponseData, Transport};
use bridge_graph::mail;
use bridge_graph::types::{Flag, MailboxRef, MessageRef, SyncCursor, KEYWORD_FOCUSED};

const ACCOUNT: &str = "user@example.test";
const TOKEN: &str = "SCRIPTED.ACCESS.TOKEN";

type BridgeResult<T> = std::result::Result<T, BridgeError>;

/// A [`Transport`] whose responses are supplied by a closure and which records every
/// request it received. `token_fails` simulates the host refusing to mint a token.
struct Scripted<F>
where
    F: Fn(&HttpRequestSpec) -> BridgeResult<HttpResponseData>,
{
    responder: F,
    seen: RefCell<Vec<HttpRequestSpec>>,
    token_fails: bool,
}

impl<F> Scripted<F>
where
    F: Fn(&HttpRequestSpec) -> BridgeResult<HttpResponseData>,
{
    fn new(responder: F) -> Self {
        Self {
            responder,
            seen: RefCell::new(Vec::new()),
            token_fails: false,
        }
    }

    fn with_failing_token(responder: F) -> Self {
        Self {
            responder,
            seen: RefCell::new(Vec::new()),
            token_fails: true,
        }
    }

    fn requests(&self) -> Vec<HttpRequestSpec> {
        self.seen.borrow().clone()
    }

    fn urls(&self) -> Vec<String> {
        self.seen.borrow().iter().map(|r| r.url.clone()).collect()
    }
}

impl<F> Transport for Scripted<F>
where
    F: Fn(&HttpRequestSpec) -> BridgeResult<HttpResponseData>,
{
    fn token(&self, _account: &str) -> BridgeResult<String> {
        if self.token_fails {
            Err(BridgeError::Auth("host refused to mint a token".into()))
        } else {
            Ok(TOKEN.to_string())
        }
    }

    fn fetch(&self, req: HttpRequestSpec) -> BridgeResult<HttpResponseData> {
        self.seen.borrow_mut().push(req.clone());
        (self.responder)(&req)
    }
}

/// A JSON 200.
fn json_ok(value: serde_json::Value) -> HttpResponseData {
    HttpResponseData {
        status: 200,
        headers: vec![("Content-Type".into(), "application/json".into())],
        body: serde_json::to_vec(&value).expect("serialize fixture body"),
    }
}

/// A body-less response with an arbitrary status.
fn bare(status: u16) -> HttpResponseData {
    HttpResponseData {
        status,
        headers: Vec::new(),
        body: Vec::new(),
    }
}

fn header<'a>(req: &'a HttpRequestSpec, name: &str) -> Option<&'a str> {
    req.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn inbox() -> MailboxRef {
    MailboxRef {
        name: "inbox".into(),
        uidvalidity: 1,
    }
}

// ── status → BridgeError classification ──────────────────────────────────────

#[test]
fn auth_statuses_map_to_the_auth_variant() {
    // 401/403 must NOT be reported as a generic transport failure: the guest maps
    // `Auth` onto the WIT `plugin-error::auth`, which is what tells the host to
    // refresh/re-consent rather than retry the same request.
    for status in [401u16, 403] {
        let tr = Scripted::new(move |_: &HttpRequestSpec| Ok(bare(status)));
        let c = GraphClient::new(&tr, ACCOUNT);
        let err = mail::list_mailboxes(&c).expect_err("an auth status must be an error");
        assert!(
            matches!(err, BridgeError::Auth(_)),
            "{status} should classify as Auth, got {err:?}"
        );
        assert!(
            err.to_string().contains(&status.to_string()),
            "the status belongs in the message: {err}"
        );
    }
}

#[test]
fn not_found_maps_to_mailbox_not_found() {
    // A 404 means the folder/message is gone, which the engine handles differently
    // from a transport failure (drop the mailbox vs retry later).
    let tr = Scripted::new(|_: &HttpRequestSpec| Ok(bare(404)));
    let c = GraphClient::new(&tr, ACCOUNT);
    let err = mail::list_mailboxes(&c).expect_err("404 must be an error");
    assert!(
        matches!(err, BridgeError::MailboxNotFound(_)),
        "404 should classify as MailboxNotFound, got {err:?}"
    );
}

#[test]
fn other_error_statuses_map_to_transport() {
    for status in [400u16, 429, 500, 503] {
        let tr = Scripted::new(move |_: &HttpRequestSpec| Ok(bare(status)));
        let c = GraphClient::new(&tr, ACCOUNT);
        let err = mail::list_mailboxes(&c).expect_err("an error status must be an error");
        assert!(
            matches!(err, BridgeError::Transport(_)),
            "{status} should classify as Transport, got {err:?}"
        );
    }
}

#[test]
fn a_2xx_with_undecodable_json_is_a_protocol_error() {
    // Graph returning a 200 whose body is not the shape we model is a protocol
    // problem, not a transport one — and must never panic the guest.
    let tr = Scripted::new(|_: &HttpRequestSpec| {
        Ok(HttpResponseData {
            status: 200,
            headers: Vec::new(),
            body: b"<html>not json</html>".to_vec(),
        })
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let err = mail::list_mailboxes(&c).expect_err("undecodable body must be an error");
    assert!(
        matches!(err, BridgeError::Protocol(_)),
        "expected Protocol, got {err:?}"
    );
}

#[test]
fn a_transport_level_failure_propagates_unchanged() {
    // The host's own fetch failure (no HTTP status at all) reaches the caller as-is.
    let tr = Scripted::new(|_: &HttpRequestSpec| {
        Err(BridgeError::Transport(
            "host fetch refused by allowlist".into(),
        ))
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let err = mail::list_mailboxes(&c).expect_err("host failure must surface");
    assert_eq!(
        err,
        BridgeError::Transport("host fetch refused by allowlist".into())
    );
}

#[test]
fn bridge_error_display_names_its_kind() {
    // The Display prefix is what appears in host logs; a mislabelled kind would send
    // an operator after the wrong problem.
    assert_eq!(BridgeError::Protocol("x".into()).to_string(), "protocol: x");
    assert_eq!(BridgeError::Auth("x".into()).to_string(), "auth: x");
    assert_eq!(
        BridgeError::Transport("x".into()).to_string(),
        "transport: x"
    );
    assert_eq!(
        BridgeError::Unsupported("x".into()).to_string(),
        "unsupported: x"
    );
    assert_eq!(
        BridgeError::MailboxNotFound("x".into()).to_string(),
        "mailbox not found: x"
    );
}

// ── credential handling ──────────────────────────────────────────────────────

#[test]
fn a_token_failure_issues_no_request_at_all() {
    // The guest must never reach Graph unauthenticated. If the host declines to mint
    // a token the call fails BEFORE `fetch`, so nothing leaves the sandbox.
    let tr = Scripted::with_failing_token(|_: &HttpRequestSpec| {
        panic!("fetch must not be reached when the token could not be acquired")
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let err = mail::list_mailboxes(&c).expect_err("no token ⇒ no call");
    assert!(matches!(err, BridgeError::Auth(_)), "got {err:?}");
    assert!(
        tr.requests().is_empty(),
        "zero requests must have been issued, saw {:?}",
        tr.urls()
    );
}

#[test]
fn every_request_carries_the_freshly_acquired_bearer() {
    // One token acquisition per call, attached transiently. Asserting on the header
    // (rather than just that a call happened) is what proves the token reaches the
    // wire at all.
    let tr = Scripted::new(|_: &HttpRequestSpec| Ok(json_ok(serde_json::json!({ "value": [] }))));
    let c = GraphClient::new(&tr, ACCOUNT);
    mail::list_mailboxes(&c).expect("empty folder list");
    let reqs = tr.requests();
    assert_eq!(reqs.len(), 1);
    assert_eq!(
        header(&reqs[0], "Authorization"),
        Some("Bearer SCRIPTED.ACCESS.TOKEN")
    );
    assert_eq!(header(&reqs[0], "Accept"), Some("application/json"));
    assert_eq!(reqs[0].method, "GET");
    assert!(
        reqs[0].body.is_none(),
        "a GET must not carry a body: {:?}",
        reqs[0].body
    );
}

// ── URL resolution: absolute vs relative ─────────────────────────────────────

#[test]
fn an_absolute_delta_link_is_used_verbatim() {
    // A stored `@odata.deltaLink` is absolute. Re-basing it onto GRAPH_BASE would
    // produce a nonsense URL and silently restart the sync from empty — the kind of
    // failure that looks like "Graph resent everything" rather than like a bug.
    const LINK: &str =
        "https://graph.microsoft.com/v1.0/me/mailFolders/inbox/messages/delta?$deltatoken=ABC123";
    let tr = Scripted::new(|_: &HttpRequestSpec| Ok(json_ok(serde_json::json!({ "value": [] }))));
    let c = GraphClient::new(&tr, ACCOUNT);
    mail::sync_mailbox(
        &c,
        &inbox(),
        &SyncCursor {
            opaque: LINK.as_bytes().to_vec(),
        },
    )
    .expect("delta with a stored cursor");
    assert_eq!(tr.urls(), vec![LINK.to_string()]);
}

#[test]
fn an_empty_cursor_builds_the_relative_initial_delta_path() {
    let tr = Scripted::new(|_: &HttpRequestSpec| Ok(json_ok(serde_json::json!({ "value": [] }))));
    let c = GraphClient::new(&tr, ACCOUNT);
    mail::sync_mailbox(&c, &inbox(), &SyncCursor::default()).expect("initial delta");
    let url = &tr.urls()[0];
    assert!(
        url.starts_with("https://graph.microsoft.com/v1.0/me/mailFolders/inbox/messages/delta"),
        "relative path must be resolved against GRAPH_BASE: {url}"
    );
    // `$select` keeps the delta payload to the properties the flag mapping reads.
    for prop in ["isRead", "flag", "inferenceClassification"] {
        assert!(url.contains(prop), "{prop} missing from $select: {url}");
    }
}

#[test]
fn a_non_utf8_cursor_is_a_protocol_error_not_a_panic() {
    // The cursor is persisted by the engine and handed back opaquely. A corrupted one
    // must surface as a typed error the host can act on (drop the cursor, resync).
    let tr = Scripted::new(|_: &HttpRequestSpec| Ok(json_ok(serde_json::json!({ "value": [] }))));
    let c = GraphClient::new(&tr, ACCOUNT);
    let err = mail::sync_mailbox(
        &c,
        &inbox(),
        &SyncCursor {
            opaque: vec![0xff, 0xfe, 0x00],
        },
    )
    .expect_err("a non-UTF-8 cursor must not be used as a URL");
    assert!(matches!(err, BridgeError::Protocol(_)), "got {err:?}");
    assert!(
        tr.requests().is_empty(),
        "a bad cursor must be rejected before any request"
    );
}

// ── folder listing: paging, roles, addressing keys ───────────────────────────

#[test]
fn list_mailboxes_follows_next_link_paging() {
    // A tenant with >100 folders pages. Only following `@odata.nextLink` yields the
    // whole list; stopping at page 1 silently hides folders.
    let page2 = "https://graph.microsoft.com/v1.0/me/mailFolders?$skip=100";
    let tr = Scripted::new(move |req: &HttpRequestSpec| {
        if req.url.contains("$skip=100") {
            Ok(json_ok(serde_json::json!({
                "value": [ { "id": "id-b", "displayName": "Later", "totalItemCount": 2, "unreadItemCount": 1 } ]
            })))
        } else {
            Ok(json_ok(serde_json::json!({
                "value": [ { "id": "id-a", "displayName": "First", "wellKnownName": "inbox" } ],
                "@odata.nextLink": page2,
            })))
        }
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let boxes = mail::list_mailboxes(&c).expect("paged folder list");
    assert_eq!(
        tr.urls().len(),
        2,
        "both pages must be fetched: {:?}",
        tr.urls()
    );
    assert_eq!(
        tr.urls()[1],
        page2,
        "page 2 is fetched at the absolute nextLink"
    );
    let names: Vec<&str> = boxes.iter().map(|m| m.mailbox_ref.name.as_str()).collect();
    assert_eq!(names, ["inbox", "id-b"]);
    assert_eq!(boxes[1].total, 2);
    assert_eq!(boxes[1].unread, 1);
}

#[test]
fn well_known_names_map_to_jmap_roles() {
    // The role drives special-use display and the engine's Sent/Trash/Drafts routing;
    // an unmapped Graph name must degrade to "none", never to a guessed role.
    let tr = Scripted::new(|_: &HttpRequestSpec| {
        Ok(json_ok(serde_json::json!({ "value": [
            { "id": "1", "wellKnownName": "inbox" },
            { "id": "2", "wellKnownName": "archive" },
            { "id": "3", "wellKnownName": "drafts" },
            { "id": "4", "wellKnownName": "sentitems" },
            { "id": "5", "wellKnownName": "deleteditems" },
            { "id": "6", "wellKnownName": "junkemail" },
            { "id": "7", "wellKnownName": "outbox" },
            { "id": "8", "wellKnownName": "conversationhistory" },
            { "id": "9" },
        ] })))
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let roles: Vec<String> = mail::list_mailboxes(&c)
        .expect("folder list")
        .into_iter()
        .map(|m| m.role)
        .collect();
    assert_eq!(
        roles,
        [
            "inbox", "archive", "drafts", "sent", "trash", "junk",
            // `outbox` and anything unrecognised both fall back to "none".
            "none", "none", "none",
        ]
    );
}

#[test]
fn an_empty_well_known_name_falls_back_to_the_folder_id() {
    // Graph can return `wellKnownName: ""`. Using it verbatim would address
    // `/me/mailFolders//messages/delta` — a 404 on every sync of that folder.
    let tr = Scripted::new(|req: &HttpRequestSpec| {
        if req.url.contains("/messages/delta") {
            Ok(json_ok(serde_json::json!({ "value": [] })))
        } else {
            Ok(json_ok(serde_json::json!({ "value": [
                { "id": "AAMkOpaqueFolderId", "displayName": "Custom", "wellKnownName": "" }
            ] })))
        }
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let boxes = mail::list_mailboxes(&c).expect("folder list");
    assert_eq!(boxes[0].mailbox_ref.name, "AAMkOpaqueFolderId");

    // …and the ref it produced re-addresses that folder correctly.
    mail::sync_mailbox(&c, &boxes[0].mailbox_ref, &SyncCursor::default()).expect("sync");
    assert!(
        tr.urls()[1].contains("/me/mailFolders/AAMkOpaqueFolderId/messages/delta"),
        "folder must be re-addressed by id: {}",
        tr.urls()[1]
    );
}

// ── delta semantics ──────────────────────────────────────────────────────────

#[test]
fn removed_entries_are_removed_and_never_also_added() {
    // A delta entry carrying `@removed` is a deletion. Counting it as an addition too
    // would resurrect deleted mail on every sync.
    let tr = Scripted::new(|_: &HttpRequestSpec| {
        Ok(json_ok(serde_json::json!({
            "value": [
                { "id": "keep", "isRead": true, "inferenceClassification": "focused" },
                { "id": "gone", "@removed": { "reason": "deleted" } },
            ],
            "@odata.deltaLink": "https://graph.microsoft.com/v1.0/next-delta",
        })))
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let delta = mail::sync_mailbox(&c, &inbox(), &SyncCursor::default()).expect("delta");

    let added: Vec<&str> = delta.added.iter().map(|m| m.raw.as_str()).collect();
    let removed: Vec<&str> = delta.removed.iter().map(|m| m.raw.as_str()).collect();
    assert_eq!(added, ["keep"]);
    assert_eq!(removed, ["gone"]);
    // A removed message also gets no flag change (nothing to apply it to).
    assert_eq!(delta.flag_changes.len(), 1);
    assert_eq!(delta.flag_changes[0].0.raw, "keep");
    assert_eq!(
        delta.flag_changes[0].1,
        vec![Flag::Seen, Flag::Keyword(KEYWORD_FOCUSED.into())]
    );
    assert_eq!(
        delta.next_cursor.opaque,
        b"https://graph.microsoft.com/v1.0/next-delta".to_vec()
    );
}

#[test]
fn a_mid_page_delta_returns_the_next_link_as_the_cursor() {
    // Mid-page there is no deltaLink, only a nextLink; returning an EMPTY cursor there
    // would make the following sync restart from the beginning and re-add everything.
    let tr = Scripted::new(|_: &HttpRequestSpec| {
        Ok(json_ok(serde_json::json!({
            "value": [ { "id": "m1" } ],
            "@odata.nextLink": "https://graph.microsoft.com/v1.0/more?$skiptoken=X",
        })))
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let delta = mail::sync_mailbox(&c, &inbox(), &SyncCursor::default()).expect("delta");
    assert_eq!(
        String::from_utf8(delta.next_cursor.opaque).unwrap(),
        "https://graph.microsoft.com/v1.0/more?$skiptoken=X"
    );
}

#[test]
fn a_delta_with_neither_link_yields_an_empty_cursor() {
    let tr = Scripted::new(|_: &HttpRequestSpec| Ok(json_ok(serde_json::json!({ "value": [] }))));
    let c = GraphClient::new(&tr, ACCOUNT);
    let delta = mail::sync_mailbox(&c, &inbox(), &SyncCursor::default()).expect("delta");
    assert!(delta.next_cursor.opaque.is_empty());
    // The watch loop has no server-push socket in this ABI — the poll is empty by
    // design and the host drives `sync_mailbox` on its own cadence.
    assert!(mail::poll_changes().is_empty());
}

// ── flag mutation: what does and does not reach Graph ────────────────────────

#[test]
fn a_flag_change_graph_cannot_express_issues_no_request() {
    // IMAP-only flags (Answered/Draft/an arbitrary keyword) have no Graph field. The
    // bridge must no-op successfully — not fail, and not PATCH an empty body.
    let tr = Scripted::new(|_: &HttpRequestSpec| {
        panic!("no request may be issued for a non-Graph-expressible flag change")
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let refs = [MessageRef {
        raw: "m1".into(),
        mailbox: inbox(),
    }];
    mail::store_flags(
        &c,
        &refs,
        &[Flag::Answered, Flag::Keyword("$Label1".into())],
        &[Flag::Draft],
    )
    .expect("a non-expressible mutation is a no-op success");
    assert!(tr.requests().is_empty());
}

#[test]
fn clearing_seen_and_flagged_patches_the_negative_graph_values() {
    // The `remove` side must write `isRead:false` / `flagStatus:"notFlagged"` — not
    // simply omit the field, which would leave the message read on the server.
    let tr = Scripted::new(|_: &HttpRequestSpec| Ok(bare(200)));
    let c = GraphClient::new(&tr, ACCOUNT);
    let refs = [MessageRef {
        raw: "m1".into(),
        mailbox: inbox(),
    }];
    mail::store_flags(&c, &refs, &[], &[Flag::Seen, Flag::Flagged]).expect("clear flags");

    let reqs = tr.requests();
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].method, "PATCH");
    assert_eq!(header(&reqs[0], "Content-Type"), Some("application/json"));
    let body: serde_json::Value =
        serde_json::from_slice(reqs[0].body.as_ref().expect("PATCH body")).unwrap();
    assert_eq!(body["isRead"], serde_json::json!(false));
    assert_eq!(body["flag"]["flagStatus"], serde_json::json!("notFlagged"));
}

#[test]
fn a_focused_keyword_add_writes_the_inference_classification() {
    let tr = Scripted::new(|_: &HttpRequestSpec| Ok(bare(200)));
    let c = GraphClient::new(&tr, ACCOUNT);
    let refs = [MessageRef {
        raw: "m1".into(),
        mailbox: inbox(),
    }];
    mail::store_flags(
        &c,
        &refs,
        &[
            Flag::Seen,
            Flag::Flagged,
            Flag::Keyword(KEYWORD_FOCUSED.into()),
        ],
        &[],
    )
    .expect("set flags");
    let reqs = tr.requests();
    let body: serde_json::Value =
        serde_json::from_slice(reqs[0].body.as_ref().expect("PATCH body")).unwrap();
    assert_eq!(body["isRead"], serde_json::json!(true));
    assert_eq!(body["flag"]["flagStatus"], serde_json::json!("flagged"));
    assert_eq!(
        body["inferenceClassification"],
        serde_json::json!("focused"),
        "the $Focused keyword is what carries Focused-Inbox state to Graph"
    );
}

#[test]
fn store_flags_patches_every_ref_and_stops_at_the_first_failure() {
    // A partial failure must surface; silently continuing would report success for a
    // mutation that only half-applied.
    let calls = std::cell::Cell::new(0u32);
    let tr = Scripted::new(|_: &HttpRequestSpec| {
        calls.set(calls.get() + 1);
        if calls.get() == 2 {
            Ok(bare(500))
        } else {
            Ok(bare(200))
        }
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let refs: Vec<MessageRef> = ["a", "b", "c"]
        .iter()
        .map(|id| MessageRef {
            raw: (*id).into(),
            mailbox: inbox(),
        })
        .collect();
    let err = mail::store_flags(&c, &refs, &[Flag::Seen], &[]).expect_err("second PATCH fails");
    assert!(matches!(err, BridgeError::Transport(_)), "got {err:?}");
    assert_eq!(tr.requests().len(), 2, "it stops rather than pressing on");
}

// ── submit ───────────────────────────────────────────────────────────────────

#[test]
fn submit_base64_encodes_the_mime_for_send_mail() {
    // Graph's MIME send path takes base64 as `text/plain`; sending raw bytes is
    // rejected by the service, so the encoding is load-bearing.
    let tr = Scripted::new(|_: &HttpRequestSpec| Ok(bare(202)));
    let c = GraphClient::new(&tr, ACCOUNT);
    let raw = b"From: a@b\r\nSubject: hi\r\n\r\nbody\r\n";
    let sent = mail::submit(&c, &inbox(), raw).expect("submit");

    let reqs = tr.requests();
    assert_eq!(reqs.len(), 1);
    assert!(reqs[0].url.ends_with("/me/sendMail"), "{}", reqs[0].url);
    assert_eq!(header(&reqs[0], "Content-Type"), Some("text/plain"));
    let body = String::from_utf8(reqs[0].body.clone().expect("send body")).unwrap();
    assert_eq!(
        body, "RnJvbTogYUBiDQpTdWJqZWN0OiBoaQ0KDQpib2R5DQo=",
        "the body must be standard base64 of the RFC 5322 bytes"
    );
    // sendMail returns no server id, so the ref is synthetic and self-describing.
    assert_eq!(sent.raw, "sent:inbox");
}

// ── best-effort caps degrade rather than fail ────────────────────────────────

#[test]
fn a_reaction_degrades_to_false_but_an_auth_failure_still_propagates() {
    // Reactions are advertised best-effort: a tenant that does not support them must
    // not surface as a broken mailbox. An auth failure is different — it means the
    // credential needs attention and must reach the host.
    let unsupported = Scripted::new(|_: &HttpRequestSpec| Ok(bare(501)));
    let c = GraphClient::new(&unsupported, ACCOUNT);
    assert!(
        !caps::react(&c, "m1", "👍").expect("degrades, never errors"),
        "an unsupported reaction reports 'not applied', not a failure"
    );

    let unauthorized = Scripted::new(|_: &HttpRequestSpec| Ok(bare(401)));
    let c = GraphClient::new(&unauthorized, ACCOUNT);
    let err = caps::react(&c, "m1", "👍").expect_err("auth must not be swallowed");
    assert!(matches!(err, BridgeError::Auth(_)), "got {err:?}");
}

#[test]
fn recall_is_declined_outright_once_the_message_has_been_read() {
    // The honesty matrix: an already-read message cannot be recalled, so the bridge
    // must not issue the recall verb and then report a hopeful outcome.
    let tr = Scripted::new(|req: &HttpRequestSpec| {
        assert_ne!(
            req.method, "POST",
            "no recall verb for an already-read message"
        );
        Ok(json_ok(serde_json::json!({ "id": "m1", "isRead": true })))
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let outcome = caps::recall(&c, "m1").expect("recall check");
    assert!(!outcome.requested);
    assert!(!outcome.guaranteed);
    assert_eq!(tr.requests().len(), 1, "only the state check is issued");

    // An unread message does issue the verb — and is STILL never reported guaranteed.
    let tr = Scripted::new(|req: &HttpRequestSpec| {
        if req.method == "POST" {
            Ok(bare(202))
        } else {
            Ok(json_ok(serde_json::json!({ "id": "m1", "isRead": false })))
        }
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let outcome = caps::recall(&c, "m1").expect("recall attempt");
    assert!(outcome.requested);
    assert!(
        !outcome.guaranteed,
        "Graph never guarantees recall — reporting otherwise would be a false claim"
    );
    assert_eq!(tr.requests().len(), 2);
}

// ── client primitives ────────────────────────────────────────────────────────

#[test]
fn post_json_tolerates_an_empty_accepted_body() {
    // Graph answers many POSTs with 202/204 and no body. Decoding that as JSON would
    // fail; the client synthesizes `null` so an `Option` target lands as `None`.
    let tr = Scripted::new(|_: &HttpRequestSpec| Ok(bare(202)));
    let c = GraphClient::new(&tr, ACCOUNT);
    let decoded: Option<serde_json::Value> = c
        .post_json(
            "/me/messages/m1/move",
            &serde_json::json!({ "destinationId": "archive" }),
        )
        .expect("an empty 202 is not a decode failure");
    assert!(decoded.is_none());
}

#[test]
fn get_bytes_returns_the_body_untouched() {
    // `$value` is raw RFC 5322 MIME, not JSON: it must reach the engine byte-exact.
    let mime = b"From: a@b\r\n\r\n\x00\x01binary-ish\r\n".to_vec();
    let expected = mime.clone();
    let tr = Scripted::new(move |_: &HttpRequestSpec| {
        Ok(HttpResponseData {
            status: 200,
            headers: Vec::new(),
            body: expected.clone(),
        })
    });
    let c = GraphClient::new(&tr, ACCOUNT);
    let refs = [MessageRef {
        raw: "m1".into(),
        mailbox: inbox(),
    }];
    let fetched = mail::fetch_raw(&c, &refs).expect("fetch raw");
    assert_eq!(fetched[0].raw, mime);
    assert!(tr.urls()[0].ends_with("/me/messages/m1/$value"));
}
