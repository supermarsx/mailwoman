//! End-to-end tests for the `mw-oauth` public surface: the scope grant/deny
//! matrix, opaque API keys, the OAuth 2.1 authorization-code + PKCE + resource
//! flow, token expiry/revocation, and the `require_scope` enforcement core.

use chrono::{Duration, Utc};
use mw_oauth::{
    AuthServer, AuthServerConfig, AuthorizeRequest, CollectingAudit, CredentialKind,
    InMemoryOAuthStore, NoopAudit, OAuthClient, OAuthError, OAuthStore, RequestContext, Scope,
    ScopeSelector, TokenRequest, challenge_s256, mint_api_key, verify_api_key, verify_pkce_s256,
};

// ── Scope builders ──────────────────────────────────────────────────────────

/// Read-only, all-accounts, mail-only base scope.
fn base_scope() -> Scope {
    Scope {
        read: true,
        send: false,
        delete: false,
        accounts: ScopeSelector::All,
        folders: ScopeSelector::All,
        mail: true,
        pim: false,
        ip_allowlist: Vec::new(),
        expires_at: None,
        rate_limit: None,
        mcp_tools: Vec::new(),
        unattended_send: false,
    }
}

// ── Scope grant/deny matrix ──────────────────────────────────────────────────

#[test]
fn scope_grants_equal_or_narrower() {
    let granted = base_scope();
    // Requiring exactly what is granted is allowed.
    assert!(granted.allows(&base_scope()));
    // Requiring nothing is trivially allowed.
    let mut nothing = base_scope();
    nothing.read = false;
    nothing.mail = false;
    assert!(granted.allows(&nothing));
}

#[test]
fn scope_denies_verb_escalation() {
    let granted = base_scope(); // read only
    let mut want_send = base_scope();
    want_send.send = true;
    assert!(!granted.allows(&want_send));

    let mut want_delete = base_scope();
    want_delete.delete = true;
    assert!(!granted.allows(&want_delete));
}

#[test]
fn scope_denies_surface_escalation() {
    let granted = base_scope(); // mail only
    let mut want_pim = base_scope();
    want_pim.pim = true;
    assert!(!granted.allows(&want_pim));
}

#[test]
fn scope_account_subset_coverage() {
    let mut granted = base_scope();
    granted.accounts = ScopeSelector::Subset(vec!["a1".into(), "a2".into()]);

    // A subset within the grant is allowed.
    let mut want = base_scope();
    want.accounts = ScopeSelector::Subset(vec!["a1".into()]);
    assert!(granted.allows(&want));

    // An account outside the grant is denied.
    let mut want_other = base_scope();
    want_other.accounts = ScopeSelector::Subset(vec!["a3".into()]);
    assert!(!granted.allows(&want_other));

    // A subset key can never cover `*` (escalation).
    let mut want_all = base_scope();
    want_all.accounts = ScopeSelector::All;
    assert!(!granted.allows(&want_all));

    // `*` covers any subset.
    assert!(base_scope().allows(&want));
}

#[test]
fn scope_folder_subset_coverage() {
    let mut granted = base_scope();
    granted.folders = ScopeSelector::Subset(vec!["INBOX".into()]);
    let mut want_inbox = base_scope();
    want_inbox.folders = ScopeSelector::Subset(vec!["INBOX".into()]);
    let mut want_sent = base_scope();
    want_sent.folders = ScopeSelector::Subset(vec!["Sent".into()]);
    assert!(granted.allows(&want_inbox));
    assert!(!granted.allows(&want_sent));
}

#[test]
fn scope_mcp_tools_and_unattended() {
    let mut granted = base_scope();
    granted.mcp_tools = vec!["mail.search".into(), "mail.read".into()];
    granted.send = true;
    granted.unattended_send = true;

    let mut want_search = base_scope();
    want_search.mcp_tools = vec!["mail.search".into()];
    assert!(granted.allows(&want_search));

    let mut want_send_tool = base_scope();
    want_send_tool.mcp_tools = vec!["mail.send".into()];
    assert!(!granted.allows(&want_send_tool));

    // unattended_send requires the granted key to carry it.
    let mut want_unattended = base_scope();
    want_unattended.send = true;
    want_unattended.unattended_send = true;
    assert!(granted.allows(&want_unattended));
    let mut plain = base_scope();
    plain.send = true;
    assert!(!plain.allows(&want_unattended));
}

// ── PKCE S256 ────────────────────────────────────────────────────────────────

#[test]
fn pkce_s256_verifies_and_rejects() {
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let challenge = challenge_s256(verifier);
    assert!(verify_pkce_s256(verifier, &challenge));
    assert!(!verify_pkce_s256("wrong-verifier", &challenge));
    assert!(!verify_pkce_s256(verifier, "not-the-challenge"));
}

// ── API keys ─────────────────────────────────────────────────────────────────

#[test]
fn api_key_hash_round_trip_and_shown_once() {
    let minted = mint_api_key("acct-1", base_scope());
    let token = minted.display_token.clone();
    assert!(token.starts_with("mwk_"));
    assert!(token.contains('.'));

    // Shown-once: the stored record never holds the plaintext secret.
    let secret = token.rsplit('.').next().unwrap();
    assert!(!minted.record.hash.contains(secret));
    assert!(minted.record.hash.starts_with("$argon2id$"));

    // Correct token verifies; tampered ones do not.
    assert!(verify_api_key(&token, &minted.record));
    assert!(!verify_api_key("mwk_deadbeef.wrongsecret", &minted.record));
    assert!(!verify_api_key("not-even-a-key", &minted.record));
}

#[test]
fn api_key_revocation_rejected() {
    let mut minted = mint_api_key("acct-1", base_scope());
    let token = minted.display_token.clone();
    assert!(verify_api_key(&token, &minted.record));
    minted.record.revoked_at = Some(Utc::now().to_rfc3339());
    assert!(!verify_api_key(&token, &minted.record));
}

// ── OAuth 2.1 flow helpers ───────────────────────────────────────────────────

async fn server_with_client(config: AuthServerConfig) -> AuthServer<InMemoryOAuthStore> {
    let server = AuthServer::with_config(InMemoryOAuthStore::new(), config);
    server
        .store()
        .put_client(OAuthClient {
            client_id: "client-1".into(),
            name: "Test App".into(),
            redirect_uris: vec!["https://app.example/cb".into()],
            approved_by: "admin".into(),
            created_at: Utc::now().to_rfc3339(),
        })
        .await
        .unwrap();
    server
}

fn authorize_req(challenge: &str, resource: &str) -> AuthorizeRequest {
    AuthorizeRequest {
        response_type: "code".into(),
        client_id: "client-1".into(),
        redirect_uri: "https://app.example/cb".into(),
        scope: base_scope(),
        state: Some("xyz".into()),
        code_challenge: challenge.into(),
        code_challenge_method: "S256".into(),
        resource: resource.into(),
    }
}

#[tokio::test]
async fn oauth_authorization_code_happy_path() {
    let server = server_with_client(AuthServerConfig::default()).await;
    let verifier = "verifier-abc-123-verifier-abc-123-verifier";
    let challenge = challenge_s256(verifier);
    let resource = "https://api.example/mail";

    let auth = server
        .authorize(&authorize_req(&challenge, resource), "acct-9")
        .await
        .unwrap();
    assert_eq!(auth.state.as_deref(), Some("xyz"));

    let tokens = server
        .token(&TokenRequest::AuthorizationCode {
            code: auth.code.clone(),
            redirect_uri: "https://app.example/cb".into(),
            client_id: "client-1".into(),
            code_verifier: verifier.into(),
            resource: resource.into(),
        })
        .await
        .unwrap();
    assert_eq!(tokens.token_type, "Bearer");
    assert!(tokens.refresh_token.is_some());
    assert_eq!(tokens.resource.as_deref(), Some(resource));

    // Introspection reports the access token active with its bound resource.
    let info = server.introspect(&tokens.access_token).await.unwrap();
    assert!(info.active);
    assert_eq!(info.account_id.as_deref(), Some("acct-9"));
    assert_eq!(info.resource.as_deref(), Some(resource));

    // Auth codes are single-use.
    let replay = server
        .token(&TokenRequest::AuthorizationCode {
            code: auth.code,
            redirect_uri: "https://app.example/cb".into(),
            client_id: "client-1".into(),
            code_verifier: verifier.into(),
            resource: resource.into(),
        })
        .await;
    assert!(matches!(replay, Err(OAuthError::InvalidGrant)));
}

#[tokio::test]
async fn oauth_rejects_pkce_mismatch() {
    let server = server_with_client(AuthServerConfig::default()).await;
    let challenge = challenge_s256("the-real-verifier-the-real-verifier-xx");
    let resource = "https://api.example/mail";
    let auth = server
        .authorize(&authorize_req(&challenge, resource), "acct-9")
        .await
        .unwrap();

    let bad = server
        .token(&TokenRequest::AuthorizationCode {
            code: auth.code,
            redirect_uri: "https://app.example/cb".into(),
            client_id: "client-1".into(),
            code_verifier: "a-different-verifier-entirely-nope-nope".into(),
            resource: resource.into(),
        })
        .await;
    assert!(matches!(bad, Err(OAuthError::PkceFailed)));
}

#[tokio::test]
async fn oauth_requires_pkce_s256_and_resource() {
    let server = server_with_client(AuthServerConfig::default()).await;
    // `plain` PKCE is refused.
    let mut req = authorize_req("challenge", "https://api.example/mail");
    req.code_challenge_method = "plain".into();
    assert!(matches!(
        server.authorize(&req, "a").await,
        Err(OAuthError::PkceFailed)
    ));
    // Missing resource indicator is refused.
    let mut req2 = authorize_req("challenge", "");
    req2.code_challenge_method = "S256".into();
    assert!(matches!(
        server.authorize(&req2, "a").await,
        Err(OAuthError::InvalidScope)
    ));
}

#[tokio::test]
async fn oauth_resource_indicator_binding() {
    let server = server_with_client(AuthServerConfig::default()).await;
    let verifier = "verifier-for-resource-binding-test-1234";
    let challenge = challenge_s256(verifier);
    let auth = server
        .authorize(&authorize_req(&challenge, "https://api.example/mail"), "a")
        .await
        .unwrap();

    // Token request with a different resource than authorized is refused.
    let mismatched = server
        .token(&TokenRequest::AuthorizationCode {
            code: auth.code,
            redirect_uri: "https://app.example/cb".into(),
            client_id: "client-1".into(),
            code_verifier: verifier.into(),
            resource: "https://evil.example/other".into(),
        })
        .await;
    assert!(matches!(mismatched, Err(OAuthError::InvalidScope)));
}

#[tokio::test]
async fn oauth_refresh_rotation() {
    let server = server_with_client(AuthServerConfig::default()).await;
    let verifier = "refresh-rotation-verifier-abcdefghijkl";
    let challenge = challenge_s256(verifier);
    let resource = "https://api.example/mail";
    let auth = server
        .authorize(&authorize_req(&challenge, resource), "acct-9")
        .await
        .unwrap();
    let first = server
        .token(&TokenRequest::AuthorizationCode {
            code: auth.code,
            redirect_uri: "https://app.example/cb".into(),
            client_id: "client-1".into(),
            code_verifier: verifier.into(),
            resource: resource.into(),
        })
        .await
        .unwrap();
    let refresh = first.refresh_token.unwrap();

    // Exchange the refresh token for a fresh pair.
    let second = server
        .token(&TokenRequest::RefreshToken {
            refresh_token: refresh.clone(),
            client_id: "client-1".into(),
            resource: Some(resource.into()),
        })
        .await
        .unwrap();
    assert_ne!(second.access_token, first.access_token);

    // The old refresh token is rotated out (single-use).
    let reuse = server
        .token(&TokenRequest::RefreshToken {
            refresh_token: refresh,
            client_id: "client-1".into(),
            resource: None,
        })
        .await;
    assert!(matches!(reuse, Err(OAuthError::InvalidGrant)));
}

#[tokio::test]
async fn oauth_token_expiry_and_revocation() {
    // Access tokens minted already-expired (negative TTL).
    let expired_cfg = AuthServerConfig {
        auth_code_ttl: Duration::minutes(10),
        access_ttl: Duration::seconds(-1),
        refresh_ttl: Duration::days(30),
    };
    let server = server_with_client(expired_cfg).await;
    let verifier = "expiry-test-verifier-abcdefghijklmnop";
    let challenge = challenge_s256(verifier);
    let resource = "https://api.example/mail";
    let auth = server
        .authorize(&authorize_req(&challenge, resource), "a")
        .await
        .unwrap();
    let tokens = server
        .token(&TokenRequest::AuthorizationCode {
            code: auth.code,
            redirect_uri: "https://app.example/cb".into(),
            client_id: "client-1".into(),
            code_verifier: verifier.into(),
            resource: resource.into(),
        })
        .await
        .unwrap();
    // Already expired → introspection inactive.
    let info = server.introspect(&tokens.access_token).await.unwrap();
    assert!(!info.active);

    // Revocation path (fresh server, live token).
    let server2 = server_with_client(AuthServerConfig::default()).await;
    let auth2 = server2
        .authorize(&authorize_req(&challenge, resource), "a")
        .await
        .unwrap();
    let t2 = server2
        .token(&TokenRequest::AuthorizationCode {
            code: auth2.code,
            redirect_uri: "https://app.example/cb".into(),
            client_id: "client-1".into(),
            code_verifier: verifier.into(),
            resource: resource.into(),
        })
        .await
        .unwrap();
    assert!(server2.introspect(&t2.access_token).await.unwrap().active);
    server2.revoke(&t2.access_token).await.unwrap();
    assert!(!server2.introspect(&t2.access_token).await.unwrap().active);
}

#[tokio::test]
async fn oauth_expired_auth_code_rejected() {
    let cfg = AuthServerConfig {
        auth_code_ttl: Duration::seconds(-1),
        access_ttl: Duration::hours(1),
        refresh_ttl: Duration::days(30),
    };
    let server = server_with_client(cfg).await;
    let verifier = "expired-code-verifier-abcdefghijklmnop";
    let challenge = challenge_s256(verifier);
    let resource = "https://api.example/mail";
    let auth = server
        .authorize(&authorize_req(&challenge, resource), "a")
        .await
        .unwrap();
    let res = server
        .token(&TokenRequest::AuthorizationCode {
            code: auth.code,
            redirect_uri: "https://app.example/cb".into(),
            client_id: "client-1".into(),
            code_verifier: verifier.into(),
            resource: resource.into(),
        })
        .await;
    assert!(matches!(res, Err(OAuthError::InvalidGrant)));
}

#[tokio::test]
async fn oauth_unknown_client_rejected() {
    let server = server_with_client(AuthServerConfig::default()).await;
    let mut req = authorize_req("challenge", "https://api.example/mail");
    req.client_id = "ghost".into();
    assert!(matches!(
        server.authorize(&req, "a").await,
        Err(OAuthError::InvalidClient)
    ));
    // Unregistered redirect URI is also refused.
    let mut req2 = authorize_req("challenge", "https://api.example/mail");
    req2.redirect_uri = "https://attacker.example/cb".into();
    assert!(matches!(
        server.authorize(&req2, "a").await,
        Err(OAuthError::InvalidClient)
    ));
}

// ── Enforcement core (require_scope) ─────────────────────────────────────────

async fn seed_api_key(server: &AuthServer<InMemoryOAuthStore>, scope: Scope) -> String {
    let minted = mint_api_key("acct-1", scope);
    server.store().put_api_key(minted.record).await.unwrap();
    minted.display_token
}

#[tokio::test]
async fn require_scope_grants_within_and_denies_outside() {
    let server = AuthServer::new(InMemoryOAuthStore::new());
    let token = seed_api_key(&server, base_scope()).await;
    let audit = CollectingAudit::new();

    let ctx = RequestContext {
        credential: &token,
        source_ip: None,
        resource: None,
    };
    // In scope (read/mail) → granted.
    let granted = server
        .require_scope(&ctx, &base_scope(), &audit)
        .await
        .unwrap();
    assert_eq!(granted.account_id, "acct-1");
    assert_eq!(granted.via, CredentialKind::ApiKey);

    // Out of scope (send) → denied.
    let mut want_send = base_scope();
    want_send.send = true;
    let denied = server.require_scope(&ctx, &want_send, &audit).await;
    assert!(matches!(denied, Err(OAuthError::InvalidScope)));

    // One audit event per call, with the right allow/deny verdict.
    let events = audit.events();
    assert_eq!(events.len(), 2);
    assert!(events[0].allowed);
    assert!(!events[1].allowed);
    assert_eq!(events[0].actor_kind, "api-key");
}

#[tokio::test]
async fn require_scope_ip_allowlist_enforced() {
    let server = AuthServer::new(InMemoryOAuthStore::new());
    let mut scope = base_scope();
    scope.ip_allowlist = vec!["10.0.0.0/8".into(), "192.168.1.5".into()];
    let token = seed_api_key(&server, scope).await;

    let ok = server
        .require_scope(
            &RequestContext {
                credential: &token,
                source_ip: Some("10.1.2.3".parse().unwrap()),
                resource: None,
            },
            &base_scope(),
            &NoopAudit,
        )
        .await;
    assert!(ok.is_ok());

    let blocked = server
        .require_scope(
            &RequestContext {
                credential: &token,
                source_ip: Some("172.16.0.1".parse().unwrap()),
                resource: None,
            },
            &base_scope(),
            &NoopAudit,
        )
        .await;
    assert!(matches!(blocked, Err(OAuthError::IpDenied)));

    // Allowlist set but source IP unknown → deny.
    let unknown = server
        .require_scope(
            &RequestContext {
                credential: &token,
                source_ip: None,
                resource: None,
            },
            &base_scope(),
            &NoopAudit,
        )
        .await;
    assert!(matches!(unknown, Err(OAuthError::IpDenied)));
}

#[tokio::test]
async fn require_scope_rate_limit_enforced() {
    let server = AuthServer::new(InMemoryOAuthStore::new());
    let mut scope = base_scope();
    scope.rate_limit = Some(2);
    let token = seed_api_key(&server, scope).await;
    let ctx = RequestContext {
        credential: &token,
        source_ip: None,
        resource: None,
    };
    assert!(
        server
            .require_scope(&ctx, &base_scope(), &NoopAudit)
            .await
            .is_ok()
    );
    assert!(
        server
            .require_scope(&ctx, &base_scope(), &NoopAudit)
            .await
            .is_ok()
    );
    let third = server.require_scope(&ctx, &base_scope(), &NoopAudit).await;
    assert!(matches!(third, Err(OAuthError::RateLimited)));
}

#[tokio::test]
async fn require_scope_expired_key_rejected() {
    let server = AuthServer::new(InMemoryOAuthStore::new());
    let mut scope = base_scope();
    scope.expires_at = Some((Utc::now() - Duration::hours(1)).to_rfc3339());
    let token = seed_api_key(&server, scope).await;
    let res = server
        .require_scope(
            &RequestContext {
                credential: &token,
                source_ip: None,
                resource: None,
            },
            &base_scope(),
            &NoopAudit,
        )
        .await;
    assert!(matches!(res, Err(OAuthError::Expired)));
}

#[tokio::test]
async fn require_scope_unknown_credential_rejected() {
    let server = AuthServer::new(InMemoryOAuthStore::new());
    let res = server
        .require_scope(
            &RequestContext {
                credential: "mwk_deadbeef.nope",
                source_ip: None,
                resource: None,
            },
            &base_scope(),
            &NoopAudit,
        )
        .await;
    assert!(matches!(res, Err(OAuthError::InvalidGrant)));
}

#[tokio::test]
async fn require_scope_oauth_token_and_resource_audience() {
    let server = server_with_client(AuthServerConfig::default()).await;
    let verifier = "enforce-oauth-verifier-abcdefghijklmnop";
    let challenge = challenge_s256(verifier);
    let resource = "https://api.example/mail";
    let auth = server
        .authorize(&authorize_req(&challenge, resource), "acct-9")
        .await
        .unwrap();
    let tokens = server
        .token(&TokenRequest::AuthorizationCode {
            code: auth.code,
            redirect_uri: "https://app.example/cb".into(),
            client_id: "client-1".into(),
            code_verifier: verifier.into(),
            resource: resource.into(),
        })
        .await
        .unwrap();

    // Correct audience → granted, resolved via OAuth token.
    let granted = server
        .require_scope(
            &RequestContext {
                credential: &tokens.access_token,
                source_ip: None,
                resource: Some(resource),
            },
            &base_scope(),
            &NoopAudit,
        )
        .await
        .unwrap();
    assert_eq!(granted.via, CredentialKind::OAuthToken);
    assert_eq!(granted.resource.as_deref(), Some(resource));

    // Wrong audience → denied (RFC 8707 binding).
    let wrong = server
        .require_scope(
            &RequestContext {
                credential: &tokens.access_token,
                source_ip: None,
                resource: Some("https://other.example/api"),
            },
            &base_scope(),
            &NoopAudit,
        )
        .await;
    assert!(matches!(wrong, Err(OAuthError::InvalidScope)));

    // A refresh token is not accepted as an access credential.
    let refresh = tokens.refresh_token.unwrap();
    let refused = server
        .require_scope(
            &RequestContext {
                credential: &refresh,
                source_ip: None,
                resource: None,
            },
            &base_scope(),
            &NoopAudit,
        )
        .await;
    assert!(matches!(refused, Err(OAuthError::InvalidGrant)));
}
