//! t87: linked mailboxes get the features of the primary.
//!
//! Real router over HTTP, engine mode, real sign-ins, step-up and link (the
//! t82 fixture). Covered:
//!   - every route family t87 added to the linked scope reaches its handler with
//!     a selector, and the handler acts for the CHILD: features the child's
//!     domain allows and the parent's refuses answer in the linked context and
//!     are refused as the parent; receipts per-sender rules written in the child
//!     never show up for the parent;
//!   - paths kept out (D2 security, D9 masked addresses, OAuth token exchange,
//!     streams) still answer `403 linkedContextUnsupported` with a selector, and
//!     an API key cannot select a link;
//!   - D10: a JMAP call naming an account other than the session's is refused,
//!     before any call runs, in the linked context AND as the primary;
//!   - D1/D3: API keys, OAuth grants (the code and every token minted from it)
//!     and push subscriptions created in the linked mailbox act for the child
//!     and are revoked or removed when the link is removed; the parent's own
//!     keys, grants and subscription survive. One browser endpoint serves the
//!     parent and the child, but never an unrelated account.
//!
//! Run: cargo test -p mw-server --test t87_linked_scope -- --test-threads=1
mod common;

use std::net::SocketAddr;
use std::sync::Arc;

use common::mock_mail::MockPop3;
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{
    DomainConfigRow, OAuthClientRow, ServerKey, Store,
    settings_policy::{SettingPolicyRow, SettingScope, SettingValueRow},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const KEY: &str = "87a1b2c3d4e5f6071829304a5b6c7d8e87a1b2c3d4e5f6071829304a5b6c7d8e";
const PARENT_PASS: &str = "Parent-Passw0rd!";
const CHILD_PASS: &str = "Child-Secret-t87-4e1b";
const REDIRECT: &str = "https://apps.example.org/cb";
const RESOURCE: &str = "https://mail.example.org/mcp";
const VERIFIER: &str = "t87-e1-verifier-abcdefghijklmnopqrstuvwxyz0123";

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
            updated_by: "t87-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
}
async fn domain_value(db: &str, domain: &str, key: &str, value: &str) {
    store(db)
        .await
        .put_setting_value(&SettingValueRow {
            scope: SettingScope::Domain,
            scope_id: domain.into(),
            key: key.into(),
            value: value.into(),
            locked: true,
            updated_by: "t87-fixture".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}
async fn domain(db: &str, name: &str, addr: SocketAddr) {
    store(db)
        .await
        .put_domain_config(&DomainConfigRow {
            name: name.into(),
            upstream_json: json!({"pop3":{"host":"127.0.0.1","port":addr.port(),"tls":"none","allowPrivate":true}}).to_string(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
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

async fn json_of(response: reqwest::Response, expected: u16) -> Value {
    let status = response.status();
    let text = response.text().await.unwrap();
    assert_eq!(status, expected, "{text}");
    body_of(&text)
}
fn body_of(text: &str) -> Value {
    if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(text).unwrap_or(Value::String(text.to_owned()))
    }
}
fn bare() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

async fn sign_in(base: &str, user: &str, pass: &str) -> reqwest::Client {
    let jar = Arc::new(reqwest::cookie::Jar::default());
    let bare = reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(jar.clone())
        .build()
        .unwrap();
    let login = json_of(
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
    headers.insert(
        "x-mailwoman-storage-authority",
        login["storageAuthority"].as_str().unwrap().parse().unwrap(),
    );
    reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(jar)
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap()
}

/// The parent's browser, and the child mailbox linked in it.
struct Fixture {
    base: String,
    db: String,
    http: reqwest::Client,
    link: String,
    revision: String,
    parent_account: String,
    child_account: String,
    child_authority: String,
}

impl Fixture {
    async fn start(name: &str) -> Self {
        unsafe {
            std::env::set_var("MW_ENGINE_TLS", "plaintext");
        }
        let parents = MockPop3::start_accounts(&[("parent@example.org", PARENT_PASS)]).await;
        let child = MockPop3::start_accounts(&[("child@child.test", CHILD_PASS)]).await;
        let dir = common::test_db::unique_dir(name);
        let db = dir.join("mw.db").to_string_lossy().into_owned();
        policy(&db, "signin.retryCooldownSecs", "0").await;
        policy(&db, "accounts.linkingEnabled", "true").await;
        domain(&db, "example.org", parents.addr).await;
        domain(&db, "child.test", child.addr).await;
        // The mock servers live as long as the test process.
        std::mem::forget(parents);
        std::mem::forget(child);
        // Features the CHILD's domain allows and the parent's refuses: in the
        // linked context the child's policy must decide.
        for cap in [
            "cap.ai_assistant",
            "cap.plugins",
            "cap.import",
            "cap.integration.nextcloud",
            "cap.pim.calendar",
        ] {
            policy(&db, cap, "true").await;
            domain_value(&db, "example.org", cap, "false").await;
        }
        let base = serve(&db).await;
        let http = sign_in(&base, "parent@example.org", PARENT_PASS).await;
        common::auth::user_step_up(&http, &base, PARENT_PASS).await;
        let linked = json_of(
            http.post(format!("{base}/api/accounts/link"))
                .json(&json!({"username":"child@child.test","password":CHILD_PASS,"displayName":"Child"}))
                .send()
                .await
                .unwrap(),
            201,
        )
        .await;
        let link = linked["account"]["id"].as_str().unwrap().to_string();
        let revision = linked["account"]["revision"].as_str().unwrap().to_string();
        let parent_me = json_of(
            http.get(format!("{base}/api/me")).send().await.unwrap(),
            200,
        )
        .await;
        let child_me = json_of(
            http.get(format!("{base}/api/me"))
                .header("x-mailwoman-link", &link)
                .send()
                .await
                .unwrap(),
            200,
        )
        .await;
        let parent_account = parent_me["accountId"].as_str().unwrap().to_string();
        let child_account = child_me["accountId"].as_str().unwrap().to_string();
        assert_ne!(parent_account, child_account);
        Fixture {
            base,
            db,
            http,
            link,
            revision,
            parent_account,
            child_account,
            child_authority: child_me["storageAuthority"].as_str().unwrap().to_string(),
        }
    }

    /// A request in the child's context: the selector and the child's own
    /// storage authority (the parent's browser holds both).
    fn child(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.http
            .request(method, format!("{}{path}", self.base))
            .header("x-mailwoman-link", &self.link)
            .header("x-mailwoman-storage-authority", &self.child_authority)
    }
    fn parent(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.http.request(method, format!("{}{path}", self.base))
    }
    async fn child_step_up(&self) {
        json_of(
            self.child(reqwest::Method::POST, "/api/security/step-up")
                .json(&json!({"password": CHILD_PASS}))
                .send()
                .await
                .unwrap(),
            200,
        )
        .await;
    }
}

async fn status_and_body(request: reqwest::RequestBuilder) -> (u16, Value) {
    let response = request.send().await.unwrap();
    let status = response.status().as_u16();
    (status, body_of(&response.text().await.unwrap()))
}
fn linked_refusal(status: u16, body: &Value) -> bool {
    status == 403 && body["code"] == "linkedContextUnsupported"
}

#[tokio::test]
async fn added_route_families_reach_their_handlers_and_act_for_the_child() {
    let f = Fixture::start("mw-t87-scope").await;
    use reqwest::Method as M;

    // Each added family reaches its handler with a selector (not the scope's
    // refusal). A body that is not what the handler wants is fine here: only
    // the scope decision is asserted.
    let reached: &[(M, &str)] = &[
        (M::GET, "/api/account/mdn-senders"),
        (M::GET, "/api/account/mdn-decision/M1"),
        (M::POST, "/api/account/contact-photo"),
        (M::POST, "/api/calendar/subscribe"),
        (M::POST, "/api/calendar/refresh"),
        (M::GET, "/api/import/formats"),
        (M::POST, "/api/import/eml"),
        (M::POST, "/api/import/oft"),
        (M::GET, "/api/account/packages"),
        (M::POST, "/api/account/packages/p1/refresh"),
        (M::GET, "/api/account/nextcloud"),
        (M::GET, "/api/account/nextcloud/pim"),
        (M::GET, "/api/nextcloud/files"),
        (M::GET, "/api/nextcloud/list"),
        (M::POST, "/api/nextcloud/upload/t1/abort"),
        (M::GET, "/api/directory/search?q=a"),
        (M::GET, "/api/assist/config"),
        (M::GET, "/api/push/vapid"),
        (M::GET, "/api/keys"),
        (M::POST, "/oauth/consent"),
        (M::GET, "/api/ui-plugins"),
        (M::POST, "/api/ui-plugins/p1/rpc"),
        (M::GET, "/api/account/appearance/customization"),
    ];
    for (method, path) in reached {
        let (status, body) = status_and_body(f.child(method.clone(), path).json(&json!({}))).await;
        assert!(
            !linked_refusal(status, &body),
            "{method} {path} is in the linked scope: {status} {body}"
        );
    }

    // The child's domain allows these features and the parent's refuses them:
    // the handler resolved the CHILD's session and applied its policy.
    for (method, path, sent) in [
        (M::GET, "/api/assist/config", json!({})),
        (M::GET, "/api/ui-plugins", json!({})),
        (M::GET, "/api/import/formats", json!({})),
        (M::GET, "/api/nextcloud/files?instanceId=i1", json!({})),
        (
            M::POST,
            "/api/calendar/refresh",
            json!({"calendarId": "c1", "url": "https://feeds.invalid/c.ics"}),
        ),
    ] {
        let (status, body) = status_and_body(f.parent(method.clone(), path).json(&sent)).await;
        assert_eq!(
            (status, body["code"].clone()),
            (403, json!("feature_disabled")),
            "the parent's domain refuses {path}: {body}"
        );
        let (status, body) = status_and_body(f.child(method.clone(), path).json(&sent)).await;
        assert!(
            body["code"] != "feature_disabled" && !linked_refusal(status, &body),
            "the child's domain allows {path}: {status} {body}"
        );
    }

    // Receipts per-sender rules belong to the mailbox they were written in.
    let listed = f
        .child(M::GET, "/api/account/mdn-senders")
        .send()
        .await
        .unwrap();
    let etag = listed.headers()["etag"].to_str().unwrap().to_owned();
    json_of(
        f.child(M::PUT, "/api/account/mdn-senders")
            .header("if-match", etag)
            .json(&json!({"address":"boss@child.test","decision":"always"}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let child_rules = json_of(
        f.child(M::GET, "/api/account/mdn-senders")
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let parent_rules = json_of(
        f.parent(M::GET, "/api/account/mdn-senders")
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let addresses = |v: &Value| -> Vec<String> {
        v["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["address"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(addresses(&child_rules), vec!["boss@child.test".to_string()]);
    assert!(addresses(&parent_rules).is_empty(), "{parent_rules}");

    // Kept out on purpose: D2 security, D9 masked addresses, the OAuth token
    // exchange and streams.
    for (method, path) in [
        (M::POST, "/api/password"),
        (M::GET, "/api/account/2fa"),
        (M::POST, "/api/account/2fa/totp/begin"),
        (M::GET, "/api/account/sessions"),
        (M::POST, "/api/session/rotate"),
        (M::GET, "/api/zeroaccess"),
        (M::POST, "/api/zeroaccess/enable"),
        (M::GET, "/api/masked"),
        (M::POST, "/api/masked/x/state"),
        (M::POST, "/oauth/token"),
        (M::GET, "/jmap/eventsource"),
        (M::GET, "/api/analytics/status"),
    ] {
        let (status, body) = status_and_body(f.child(method.clone(), path).json(&json!({}))).await;
        assert!(
            linked_refusal(status, &body),
            "{method} {path} must stay out of the linked scope: {status} {body}"
        );
    }
    // A scoped API key never selects a link.
    let (status, body) = status_and_body(
        f.child(M::GET, "/api/keys")
            .header("x-api-key", "mw_fake_key"),
    )
    .await;
    assert!(linked_refusal(status, &body), "{status} {body}");
}

#[tokio::test]
async fn d10_a_jmap_call_naming_another_account_is_refused_in_every_session() {
    let f = Fixture::start("mw-t87-d10").await;
    use reqwest::Method as M;
    let request = |account: Option<&str>| {
        let mut args = json!({"ids": null});
        if let Some(account) = account {
            args["accountId"] = json!(account);
        }
        json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [["Core/echo", {"accountId": "anything"}, "e"], ["Mailbox/get", args, "m1"]]
        })
    };
    // The linked context with the parent's id: the stale-cache case D10 exists for.
    let (status, body) = status_and_body(
        f.child(M::POST, "/jmap/api")
            .json(&request(Some(&f.parent_account))),
    )
    .await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["code"], "accountNotFound");
    assert_eq!(body["callId"], "m1");
    assert_eq!(body["method"], "Mailbox/get");
    assert_eq!(body["accountId"], json!(f.parent_account));
    assert_eq!(body["sessionAccountId"], json!(f.child_account));
    // Its own id, or none, runs.
    for account in [Some(f.child_account.as_str()), None] {
        let ok = json_of(
            f.child(M::POST, "/jmap/api")
                .json(&request(account))
                .send()
                .await
                .unwrap(),
            200,
        )
        .await;
        assert_eq!(ok["methodResponses"][1][0], "Mailbox/get", "{ok}");
    }
    // The primary naming the child's id: refused too (every session).
    let (status, body) = status_and_body(
        f.parent(M::POST, "/jmap/api")
            .json(&request(Some(&f.child_account))),
    )
    .await;
    assert_eq!(
        (status, body["code"].clone()),
        (400, json!("accountNotFound")),
        "{body}"
    );
    let ok = json_of(
        f.parent(M::POST, "/jmap/api")
            .json(&request(Some(&f.parent_account)))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(ok["methodResponses"][1][0], "Mailbox/get", "{ok}");
}

fn key_scope(account: &str) -> Value {
    json!({
        "read": true, "send": false, "delete": false,
        "accounts": { "subset": [account] }, "folders": "all",
        "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
        "rate_limit": null, "mcp_tools": ["mail.search"],
        "unattended_send": false,
    })
}
fn authorize_params(client_id: &str) -> Value {
    json!({
        "clientId": client_id,
        "redirectUri": REDIRECT,
        "state": "s1",
        "codeChallenge": mw_oauth::challenge_s256(VERIFIER),
        "codeChallengeMethod": "S256",
        "resource": RESOURCE,
    })
}
async fn grant(request: reqwest::RequestBuilder, base: &str, client_id: &str) -> Value {
    let decided = json_of(
        request
            .json(&json!({ "approve": true, "params": authorize_params(client_id) }))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let redirect = decided["redirectUri"].as_str().expect("redirectUri");
    let code = redirect
        .split_once("?code=")
        .unwrap()
        .1
        .split('&')
        .next()
        .unwrap()
        .to_string();
    json_of(
        bare()
            .post(format!("{base}/oauth/token"))
            .json(&json!({
                "grant_type": "authorization_code",
                "code": code,
                "redirect_uri": REDIRECT,
                "client_id": client_id,
                "code_verifier": VERIFIER,
                "resource": RESOURCE,
            }))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await
}
async fn introspect(base: &str, token: &Value) -> Value {
    json_of(
        bare()
            .post(format!("{base}/oauth/introspect"))
            .json(&json!({ "token": token }))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await
}
fn hash(endpoint: &str) -> String {
    Sha256::digest(endpoint.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[tokio::test]
async fn keys_oauth_grants_and_push_made_in_a_linked_mailbox_go_with_the_link() {
    let f = Fixture::start("mw-t87-grants").await;
    use reqwest::Method as M;
    let st = store(&f.db).await;

    // D1 API/MCP keys: minted in the child after the CHILD's step-up, for the
    // child's account, listed only there.
    f.child_step_up().await;
    let child_key = json_of(
        f.child(M::POST, "/api/keys")
            .json(&json!({"label":"t87","scope":key_scope(&f.child_account)}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let child_prefix = child_key["record"]["prefix"].as_str().unwrap().to_string();
    assert_eq!(child_key["record"]["accountId"], json!(f.child_account));
    let parent_key = json_of(
        f.parent(M::POST, "/api/keys")
            .json(&json!({"label":"t87-parent","scope":key_scope(&f.parent_account)}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let parent_prefix = parent_key["record"]["prefix"].as_str().unwrap().to_string();
    let prefixes = |v: &Value| -> Vec<String> {
        v.as_array()
            .or_else(|| v["keys"].as_array())
            .unwrap_or_else(|| panic!("key list {v}"))
            .iter()
            .map(|k| k["prefix"].as_str().unwrap().to_owned())
            .collect()
    };
    let child_list = json_of(f.child(M::GET, "/api/keys").send().await.unwrap(), 200).await;
    let parent_list = json_of(f.parent(M::GET, "/api/keys").send().await.unwrap(), 200).await;
    assert_eq!(prefixes(&child_list), vec![child_prefix.clone()]);
    assert_eq!(prefixes(&parent_list), vec![parent_prefix.clone()]);

    // D1 OAuth: a grant in the child is for the child's account; a refresh keeps
    // the chain tied to the link. The parent has its own grant.
    let client_id = "t87-client";
    st.put_oauth_client(&OAuthClientRow {
        client_id: client_id.into(),
        name: "t87 client".into(),
        redirect_uris_json: json!([REDIRECT]).to_string(),
        approved_by: "root".into(),
        created_at: "2026-10-29T00:00:00Z".into(),
    })
    .await
    .unwrap();
    let child_tokens = grant(f.child(M::POST, "/oauth/decision"), &f.base, client_id).await;
    let first = introspect(&f.base, &child_tokens["access_token"]).await;
    assert_eq!(first["active"], true);
    assert_eq!(first["account_id"], json!(f.child_account));
    let refreshed = json_of(
        bare()
            .post(format!("{}/oauth/token", f.base))
            .json(&json!({
                "grant_type": "refresh_token",
                "refresh_token": child_tokens["refresh_token"],
                "client_id": client_id,
            }))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let parent_tokens = grant(f.parent(M::POST, "/oauth/decision"), &f.base, client_id).await;
    assert_eq!(
        introspect(&f.base, &parent_tokens["access_token"]).await["account_id"],
        json!(f.parent_account)
    );

    // D3 push: both mailboxes enable notifications; one browser endpoint gets a
    // row for each; an endpoint only the child holds still answers the service
    // worker (which asks as the parent); an unrelated owner's endpoint is refused.
    let config = json!({"enabled":true,"rules":[],"quietHours":{"enabled":false,"start":"22:00","end":"07:00","timezone":"UTC"}});
    json_of(
        f.parent(M::PUT, "/api/account/notifications")
            .json(&config)
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    json_of(
        f.child(M::PUT, "/api/account/notifications")
            .json(&config)
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let shared = "https://push.example/t87-browser";
    let only_child = "https://push.example/t87-child-only";
    let subscribe = |endpoint: &str| json!({"transport":"unifiedpush","endpoint":endpoint});
    let parent_sub = json_of(
        f.parent(M::POST, "/api/push/subscribe")
            .json(&subscribe(shared))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let child_sub = json_of(
        f.child(M::POST, "/api/push/subscribe")
            .json(&subscribe(shared))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_ne!(
        parent_sub["id"], child_sub["id"],
        "each mailbox has its own row"
    );
    json_of(
        f.child(M::POST, "/api/push/subscribe")
            .json(&subscribe(only_child))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let permission = |endpoint: &str| {
        f.parent(
            M::GET,
            &format!("/api/push/permission?endpointHash={}", hash(endpoint)),
        )
    };
    let seen = json_of(permission(only_child).send().await.unwrap(), 200).await;
    assert_eq!(seen["registered"], true, "{seen}");
    assert_eq!(seen["allowed"], true, "{seen}");
    st.upsert_personal_push_subscription(&mw_store::PushSubscriptionRow {
        id: "foreign".into(),
        account_id: "another-owner".into(),
        transport: "unifiedpush".into(),
        endpoint: "https://push.example/t87-foreign".into(),
        p256dh: None,
        auth: None,
        app_id: None,
        expires_at: None,
        created_at: "2026-10-29T00:00:00Z".into(),
        last_wake_at: None,
    })
    .await
    .unwrap();
    let (status, body) = status_and_body(
        f.child(M::POST, "/api/push/subscribe")
            .json(&subscribe("https://push.example/t87-foreign")),
    )
    .await;
    assert_eq!(
        (status, body["code"].clone()),
        (409, json!("pushEndpointAlreadyBound"))
    );
    let child_subject = st
        .native_personal_subject(&f.child_account)
        .await
        .unwrap()
        .unwrap();
    let parent_subject = st
        .native_personal_subject(&f.parent_account)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        st.list_personal_push_subscriptions(&child_subject)
            .await
            .unwrap()
            .len(),
        2
    );

    // Unlink.
    json_of(
        f.parent(M::DELETE, &format!("/api/accounts/{}", f.link))
            .header("if-match", format!("\"{}\"", f.revision))
            .send()
            .await
            .unwrap(),
        204,
    )
    .await;

    // Keys: the child's is revoked, the parent's is not.
    assert!(
        st.get_api_key(&child_prefix)
            .await
            .unwrap()
            .unwrap()
            .revoked_at
            .is_some()
    );
    assert!(
        st.get_api_key(&parent_prefix)
            .await
            .unwrap()
            .unwrap()
            .revoked_at
            .is_none()
    );
    // OAuth: every token of the child's chain is dead; the parent's lives.
    for token in [
        &child_tokens["access_token"],
        &refreshed["access_token"],
        &refreshed["refresh_token"],
    ] {
        assert_eq!(introspect(&f.base, token).await["active"], false, "{token}");
    }
    assert_eq!(
        introspect(&f.base, &parent_tokens["access_token"]).await["active"],
        true
    );
    let (status, _) =
        status_and_body(bare().post(format!("{}/oauth/token", f.base)).json(&json!({
            "grant_type": "refresh_token",
            "refresh_token": refreshed["refresh_token"],
            "client_id": client_id,
        })))
        .await;
    assert_eq!(status, 400, "a revoked refresh token mints nothing");
    // Push: the child's rows are gone; the parent's stays.
    assert!(
        st.list_personal_push_subscriptions(&child_subject)
            .await
            .unwrap()
            .is_empty()
    );
    let parent_rows = st
        .list_personal_push_subscriptions(&parent_subject)
        .await
        .unwrap();
    assert_eq!(parent_rows.len(), 1);
    assert_eq!(parent_rows[0].endpoint, shared);
}

