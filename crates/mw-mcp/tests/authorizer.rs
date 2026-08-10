//! The `OAuthAuthorizer` **deny** paths, and the audit trail each one leaves.
//!
//! `tests/mcp.rs` proves the grant path and one bogus-credential denial. Every other
//! way a credential can be refused — an unknown prefix, a real prefix with the wrong
//! secret, a revoked key, an expired scope, a token that does not introspect — is
//! part of the per-call authorization surface in front of every MCP tool, and each
//! is expected to emit an audit event saying *which* check refused it. An audit line
//! that says only "denied" is not much use to whoever is reading it after the fact.
//!
//! The wrong-secret case is the load-bearing one: an API key's prefix is public (it
//! is the lookup key), so a test that only tried a wholly-invented token would still
//! pass if `verify_api_key` were dropped from the path.

use std::sync::Arc;

use chrono::{Duration, Utc};
use serde_json::{Value, json};

use mw_mcp::mock::MockBackend;
use mw_mcp::{Credential, McpServer, OAuthAuthorizer};
use mw_oauth::{
    AuditEvent, AuthServer, CollectingAudit, InMemoryOAuthStore, MintedApiKey, OAuthStore, Scope,
    mint_api_key,
};

type Server = McpServer<MockBackend, OAuthAuthorizer<InMemoryOAuthStore, CollectingAudit>>;

fn search_scope() -> Scope {
    let mut scope = Scope::read_only("acct1");
    scope.mail = true;
    scope.mcp_tools = vec!["mail.search".to_string()];
    scope
}

/// Mint a `mail.search` key, store it, and build a server whose authorizer is the
/// real `OAuthAuthorizer` over the same store.
async fn fixture(scope: Scope) -> (Server, MintedApiKey, Arc<CollectingAudit>) {
    let minted = mint_api_key("acct1", scope);
    let auth_server = Arc::new(AuthServer::new(InMemoryOAuthStore::new()));
    auth_server
        .store()
        .put_api_key(minted.record.clone())
        .await
        .expect("store the key");
    let audit = Arc::new(CollectingAudit::new());
    let authz = Arc::new(OAuthAuthorizer::without_countersign(
        auth_server,
        audit.clone(),
    ));
    (
        McpServer::new(Arc::new(MockBackend::new()), authz),
        minted,
        audit,
    )
}

fn search(token: &str) -> (Credential<'_>, Value) {
    (
        Credential {
            token,
            source_ip: None,
            resource: None,
        },
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": { "name": "mail.search", "arguments": { "account": "acct1" } } }),
    )
}

fn last(audit: &CollectingAudit) -> AuditEvent {
    audit.events().last().cloned().expect("an audit event")
}

#[tokio::test]
async fn a_real_prefix_with_the_wrong_secret_is_denied() {
    // The prefix half of `mwk_<prefix>.<secret>` is the public lookup key. Finding the
    // row must not be the same as authenticating against it.
    let (server, minted, audit) = fixture(search_scope()).await;
    let forged = format!("mwk_{}.definitely-not-the-secret", minted.record.prefix);
    let (cred, req) = search(&forged);

    let resp = server.handle_rpc(&cred, req).await.expect("response");
    assert_eq!(resp["error"]["code"], json!(-32001), "{resp}");

    let event = last(&audit);
    assert!(!event.allowed);
    assert_eq!(event.actor, minted.record.prefix, "the prefix is the actor");
    assert_eq!(event.actor_kind, "api-key");
    assert_eq!(event.action, "mcp/tools_call");
    assert_eq!(event.reason.as_deref(), Some("invalid credential"));

    // Sanity: the SAME server accepts the genuine token, so the denial above is the
    // secret check and not a broken fixture.
    let (cred, req) = search(&minted.display_token);
    let ok = server.handle_rpc(&cred, req).await.expect("response");
    assert!(ok["result"].is_object(), "{ok}");
    assert!(last(&audit).allowed);
}

#[tokio::test]
async fn an_unknown_key_prefix_is_denied_and_named_as_such() {
    let (server, _minted, audit) = fixture(search_scope()).await;
    let (cred, req) = search("mwk_nosuchprefix.secret");
    let resp = server.handle_rpc(&cred, req).await.expect("response");
    assert_eq!(resp["error"]["code"], json!(-32001), "{resp}");

    let event = last(&audit);
    assert!(!event.allowed);
    assert_eq!(event.reason.as_deref(), Some("unknown key"));
    // The presented prefix is recorded, so a scan against many prefixes is visible in
    // the audit log rather than collapsing to one anonymous line.
    assert_eq!(event.actor, "nosuchprefix");
}

#[tokio::test]
async fn a_revoked_key_stops_working_immediately() {
    // Revocation is checked inside `verify_api_key`, so it lands on the same deny
    // path as a bad secret — but it is the check an admin relies on after a leak.
    let minted = mint_api_key("acct1", search_scope());
    let mut record = minted.record.clone();
    record.revoked_at = Some(Utc::now().to_rfc3339());

    let auth_server = Arc::new(AuthServer::new(InMemoryOAuthStore::new()));
    auth_server
        .store()
        .put_api_key(record)
        .await
        .expect("store the revoked key");
    let audit = Arc::new(CollectingAudit::new());
    let server = McpServer::new(
        Arc::new(MockBackend::new()),
        Arc::new(OAuthAuthorizer::without_countersign(
            auth_server,
            audit.clone(),
        )),
    );

    let (cred, req) = search(&minted.display_token);
    let resp = server.handle_rpc(&cred, req).await.expect("response");
    assert_eq!(
        resp["error"]["code"],
        json!(-32001),
        "a revoked key must not call a tool: {resp}"
    );
    assert!(!last(&audit).allowed);
}

#[tokio::test]
async fn an_expired_scope_is_refused_even_though_the_secret_verifies() {
    // Expiry is a separate gate from authentication: the credential is genuine, the
    // grant is over.
    let mut scope = search_scope();
    scope.expires_at = Some((Utc::now() - Duration::minutes(1)).to_rfc3339());
    let (server, minted, audit) = fixture(scope).await;

    let (cred, req) = search(&minted.display_token);
    let resp = server.handle_rpc(&cred, req).await.expect("response");
    assert_eq!(resp["error"]["code"], json!(-32001), "{resp}");
    assert_eq!(last(&audit).reason.as_deref(), Some("expired"));
}

#[tokio::test]
async fn a_scope_that_expires_in_the_future_is_still_accepted() {
    // The mirror of the case above — otherwise "deny everything with an expiry set"
    // would pass the expiry test just as well.
    let mut scope = search_scope();
    scope.expires_at = Some((Utc::now() + Duration::hours(1)).to_rfc3339());
    let (server, minted, audit) = fixture(scope).await;

    let (cred, req) = search(&minted.display_token);
    let resp = server.handle_rpc(&cred, req).await.expect("response");
    assert!(resp["result"].is_object(), "{resp}");
    assert!(last(&audit).allowed);
}

#[tokio::test]
async fn a_malformed_expiry_is_treated_as_expired() {
    // Fail closed: an unparseable timestamp must not read as "no expiry".
    let mut scope = search_scope();
    scope.expires_at = Some("not-a-timestamp".to_string());
    let (server, minted, audit) = fixture(scope).await;

    let (cred, req) = search(&minted.display_token);
    let resp = server.handle_rpc(&cred, req).await.expect("response");
    assert_eq!(resp["error"]["code"], json!(-32001), "{resp}");
    assert_eq!(last(&audit).reason.as_deref(), Some("expired"));
}

#[tokio::test]
async fn a_bearer_that_is_not_an_api_key_goes_down_the_introspection_path() {
    // Anything without the `mwk_` scheme is treated as an OAuth access token. One
    // that does not introspect is inactive, and is audited as an oauth-token denial
    // — the actor_kind matters, because the two credential types are revoked and
    // rotated by different operators.
    let (server, _minted, audit) = fixture(search_scope()).await;
    let (cred, req) = search("an-access-token-that-was-never-issued");
    let resp = server.handle_rpc(&cred, req).await.expect("response");
    assert_eq!(resp["error"]["code"], json!(-32001), "{resp}");

    let event = last(&audit);
    assert!(!event.allowed);
    assert_eq!(event.actor_kind, "oauth-token");
    assert_eq!(event.reason.as_deref(), Some("inactive token"));
}

#[tokio::test]
async fn a_key_granted_one_tool_cannot_reach_another_and_the_audit_says_why() {
    // Scope denial is distinct from credential failure, and the reason recorded has
    // to distinguish them or the log cannot tell a leak from a mis-provisioned key.
    let (server, minted, audit) = fixture(search_scope()).await;
    let cred = Credential {
        token: &minted.display_token,
        source_ip: None,
        resource: None,
    };
    let resp = server
        .handle_rpc(
            &cred,
            json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                    "params": { "name": "mail.read",
                                "arguments": { "account": "acct1", "message_id": "m1" } } }),
        )
        .await
        .expect("response");
    assert_eq!(resp["error"]["code"], json!(-32001), "{resp}");

    let event = last(&audit);
    assert!(!event.allowed);
    assert_eq!(event.reason.as_deref(), Some("scope denied"));
    assert_eq!(event.actor_kind, "api-key");
    assert!(!event.ts.is_empty(), "an audit event needs a timestamp");
}

#[tokio::test]
async fn a_key_for_one_account_cannot_be_used_against_another() {
    // The required fragment names the account from the tool arguments, so a key
    // scoped to `acct1` must not serve a call that says `account: "acct2"`.
    let (server, minted, audit) = fixture(search_scope()).await;
    let cred = Credential {
        token: &minted.display_token,
        source_ip: None,
        resource: None,
    };
    let resp = server
        .handle_rpc(
            &cred,
            json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                    "params": { "name": "mail.search",
                                "arguments": { "account": "acct2" } } }),
        )
        .await
        .expect("response");
    assert_eq!(resp["error"]["code"], json!(-32001), "{resp}");
    assert_eq!(last(&audit).reason.as_deref(), Some("scope denied"));
}

#[tokio::test]
async fn every_attempt_is_audited_exactly_once_under_one_action_name() {
    // A denial that emitted nothing would be invisible; a grant that emitted twice
    // would inflate any rate/anomaly signal built on this log.
    let (server, minted, audit) = fixture(search_scope()).await;
    for token in [
        minted.display_token.as_str(),
        "mwk_bogus.secret",
        "not-a-key",
    ] {
        let (cred, req) = search(token);
        let _ = server.handle_rpc(&cred, req).await;
    }
    let events = audit.events();
    assert_eq!(events.len(), 3, "one event per attempt: {events:?}");
    assert!(events.iter().all(|e| e.action == "mcp/tools_call"));
    assert_eq!(events.iter().filter(|e| e.allowed).count(), 1);
    assert!(
        events
            .iter()
            .filter(|e| !e.allowed)
            .all(|e| e.reason.is_some()),
        "a denial without a reason is not actionable: {events:?}"
    );
}
