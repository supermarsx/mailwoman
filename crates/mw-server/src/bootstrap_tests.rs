//! t52w-e6: `GET /api/bootstrap` through the production router.
//!
//! The differential matrix sends, for every caller kind, one bootstrap and then
//! each section's route directly with the same headers, and requires the
//! section's status and body to equal the direct answer.
use super::*;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request as HttpRequest, StatusCode},
};
use mw_store::{
    AccountKind, Credentials, DomainConfigRow, NewAccount, ServerKey, Store,
    settings_policy::SettingPolicyRow,
};
use serde_json::{Value, json};
use std::future::Future;

/// One row per section: name and the direct route it must equal. A section
/// added to [`SECTIONS`] without a row here fails `every_section_has_a_matrix_row`.
const ROWS: &[(&str, &str)] = &[
    ("me", "/api/me"),
    ("jmapSession", "/jmap/session"),
    ("publicConfig", "/api/public-config"),
    ("cookiesConfig", "/api/cookies/config"),
    ("accounts", "/api/accounts"),
    ("settings", "/api/settings"),
    ("appearance", "/api/account/appearance"),
    ("preferences", "/api/account/preferences"),
    ("features", "/api/features"),
    ("notifications", "/api/account/notifications"),
    ("policy", "/api/policy"),
    ("restriction", "/api/privacy/restriction"),
    ("uiPlugins", "/api/ui-plugins"),
    ("assistConfig", "/api/assist/config"),
    ("version", "/api/version"),
    ("focusedSenders", "/api/account/focused-senders"),
    ("drafts", "/api/account/drafts"),
    ("notificationCenter", "/api/account/notification-center"),
    ("appearanceCustomization", "/api/account/appearance/customization"),
];

const USER: &str = "boot@bootstrap.test";
const ADMIN: &str = "boot-admin";
const PASSWORD: &str = "fixture-only-not-a-real-password";

#[allow(dead_code)]
#[path = "../tests/common/mock_imap.rs"]
mod mock_imap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Caller {
    Anonymous,
    Normal,
    PasswordChangeHeld,
    Restricted,
    FeatureOff,
    LinkedScope,
    Admin,
}

const CALLERS: &[Caller] = &[
    Caller::Anonymous,
    Caller::Normal,
    Caller::PasswordChangeHeld,
    Caller::Restricted,
    Caller::FeatureOff,
    Caller::LinkedScope,
    Caller::Admin,
];

struct Fixture {
    _mail: mock_imap::MockImap,
    router: Router,
    writer: Store,
    session: String,
    admin_token: String,
}

async fn policy(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "t52w-e6-fixture".into(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
}

impl Fixture {
    /// A fresh database and router; the caller kind's state is written before
    /// the router starts, so no cache can hide it.
    async fn new(caller: Caller) -> Self {
        let directory =
            std::env::temp_dir().join(format!("mw-t52w-boot-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let db = directory.join("mail.db");
        let key = ServerKey::generate();
        let mail = mock_imap::MockImap::start(USER, PASSWORD).await;
        let writer = Store::open(db.to_str().unwrap(), key.clone()).await.unwrap();
        writer
            .put_domain_config(&DomainConfigRow {
                name: "bootstrap.test".into(),
                upstream_json: json!({"imap":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","auth":"password","allowPrivate":true}}).to_string(),
                allowlist_json: "[]".into(),
                blocklist_json: "[]".into(),
                enabled: true,
                preferred: false,
                settings_json: "{}".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        let credentials = Credentials {
            username: USER.into(),
            password: PASSWORD.into(),
        };
        let account = writer
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "127.0.0.1",
                    port: mail.addr.port(),
                    tls: "plaintext",
                    username: USER,
                    sync_policy_json: "{}",
                },
                &credentials,
            )
            .await
            .unwrap();
        writer.remember_account_names(&account, &[USER]).await.unwrap();
        let session = writer
            .create_session(&account, USER, "engine", "engine", &credentials)
            .await
            .unwrap();
        writer.set_session_authentication(&session, "password").await.unwrap();
        writer
            .bind_verified_primary(
                &session,
                mw_store::identity_principal::VerifiedPrimary {
                    account_id: &account,
                    kind: "mailbox",
                    authority: "t52w-e6-fixture",
                    subject: USER,
                },
                chrono::Utc::now().timestamp(),
            )
            .await
            .unwrap()
            .unwrap();
        policy(&writer, "access.protocols", json!(["imap"])).await;
        match caller {
            Caller::PasswordChangeHeld => {
                crate::account_gate::write_flags(
                    &writer,
                    USER,
                    mw_admin::UserFeatureFlags {
                        force_password_change: true,
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
            }
            Caller::Restricted => {
                let now = chrono::Utc::now().timestamp();
                writer
                    .restriction_apply(&mw_store::privacy::RestrictionRow {
                        id: uuid::Uuid::new_v4().to_string(),
                        account_id: account.clone(),
                        address: USER.into(),
                        domain: "bootstrap.test".into(),
                        scopes: mw_store::privacy::RESTRICTION_SCOPES
                            .iter()
                            .map(|s| (*s).to_owned())
                            .collect(),
                        mail_delivery: "keep".into(),
                        reason: "fixture".into(),
                        source: "admin".into(),
                        request_id: None,
                        created_by: "fixture".into(),
                        created_at: now,
                        expires_at: None,
                        lifted_at: None,
                        lifted_by: None,
                        lift_reason: None,
                    })
                    .await
                    .unwrap();
            }
            Caller::FeatureOff => {
                for key in [
                    "cap.plugins",
                    "cap.ai_assistant",
                    "cap.appearance.custom",
                    "cap.appearance.custom_code",
                ] {
                    policy(&writer, key, json!(false)).await;
                }
            }
            _ => {}
        }
        let admin_token = ServerKey::generate().to_hex();
        if caller == Caller::Admin {
            let rows: Vec<(&str, &str)> = crate::capabilities::BUILTIN_ROLES
                .iter()
                .map(|r| (r.id, r.name))
                .collect();
            writer.ensure_builtin_admin_roles(&rows).await.unwrap();
            let now = chrono::Utc::now().timestamp();
            writer
                .insert_admin_account_with_roles(
                    &mw_store::AdminAccountRow {
                        id: uuid::Uuid::new_v4().to_string(),
                        username: ADMIN.into(),
                        display_name: "Fixture".into(),
                        password_hash: None,
                        disabled: false,
                        source: "db".into(),
                        created_at: now,
                        updated_at: now,
                        last_login_at: None,
                    },
                    &[mw_store::AdminRoleGrant {
                        role_id: crate::capabilities::SUPER_ADMIN.into(),
                        domains: None,
                    }],
                    false,
                )
                .await
                .unwrap();
            writer
                .put_admin_session_with_timeouts(
                    &crate::push_relay::hash_token(&admin_token),
                    ADMIN,
                    now,
                    7200,
                    86400,
                )
                .await
                .unwrap();
        }
        let (router, _) = crate::build_app_inner(
            crate::AppConfig {
                db_path: db.to_string_lossy().into_owned(),
                server_key_hex: Some(key.to_hex()),
                web_dir: None,
                cookie_secure: false,
                mode: crate::ServerMode::Engine,
                hardening: Default::default(),
                security: Default::default(),
            },
            crate::V6Config {
                admin_enabled: caller == Caller::Admin,
                ..Default::default()
            },
            crate::sso::SsoProviderSource::Store,
            None,
            crate::core_test_build_info::info().unwrap(),
        )
        .await
        .unwrap();
        Self { _mail: mail, router, writer, session, admin_token }
    }

    fn csrf(&self) -> String {
        self.writer.session_csrf_token(&self.session)
    }

    /// The headers a browser's load-time GET carries for this caller.
    fn headers(&self, caller: Caller) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("host", "localhost".parse().unwrap());
        headers.insert("user-agent", "Mozilla/5.0 (t52w)".parse().unwrap());
        headers.insert("accept", "*/*".parse().unwrap());
        let cookie = match caller {
            Caller::Anonymous => None,
            Caller::Admin => Some(format!(
                "{}={}",
                mw_admin::config::session_cookie_name(),
                self.admin_token
            )),
            _ => Some(format!("mw_session={}; mw_csrf={}", self.session, self.csrf())),
        };
        if let Some(cookie) = cookie {
            headers.insert("cookie", cookie.parse().unwrap());
        }
        if caller == Caller::LinkedScope {
            headers.insert(
                crate::linked_accounts::SELECTOR,
                uuid::Uuid::new_v4().to_string().parse().unwrap(),
            );
        }
        headers
    }

    async fn send(&self, uri: &str, headers: &HeaderMap) -> (StatusCode, HeaderMap, Bytes) {
        let mut request = HttpRequest::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        *request.headers_mut() = headers.clone();
        request.extensions_mut().insert(axum::extract::ConnectInfo(
            "127.0.0.1:48213".parse::<std::net::SocketAddr>().unwrap(),
        ));
        let response = tokio::time::timeout(std::time::Duration::from_secs(90), self.router.clone().oneshot(request))
            .await
            .unwrap_or_else(|_| panic!("{uri} did not answer within 90 s"))
            .unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = to_bytes(response.into_body(), 64 * 1024 * 1024).await.unwrap();
        (status, headers, bytes)
    }
}

fn run<F, Fut>(factory: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()>,
{
    let thread = std::thread::Builder::new()
        .name("t52w-bootstrap".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(factory())
        })
        .unwrap();
    if let Err(panic) = thread.join() {
        std::panic::resume_unwind(panic);
    }
}

/// What a section body must equal, from the direct answer's bytes.
fn expected_body(headers: &HeaderMap, bytes: &[u8]) -> Value {
    if bytes.is_empty() {
        return Value::Null;
    }
    let json = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json") || v.starts_with("application/problem+json"));
    match json.then(|| serde_json::from_slice::<Value>(bytes).ok()).flatten() {
        Some(value) => value,
        None => Value::String(String::from_utf8_lossy(bytes).into_owned()),
    }
}

fn direct_uri(name: &str, path: &str, me: &Value) -> String {
    let section = SECTIONS.iter().find(|s| s.name == name).unwrap();
    let account = (me["status"] == 200)
        .then(|| me["body"]["accountId"].as_str())
        .flatten();
    let pq = path_and_query(section, account);
    assert_eq!(pq.path(), path);
    pq.to_string()
}

#[test]
fn every_section_has_a_matrix_row() {
    let sections: Vec<(&str, &str)> = SECTIONS.iter().map(|s| (s.name, s.path)).collect();
    assert_eq!(sections, ROWS, "each section needs exactly one differential row");
    assert_eq!(SECTIONS[0].name, "me", "me is dispatched first");
    let mut names: Vec<_> = SECTIONS.iter().map(|s| s.name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), SECTIONS.len(), "section names are unique");
    assert!(
        SECTIONS.iter().all(|s| s.path != "/api/bootstrap"),
        "the endpoint never dispatches itself"
    );
}

#[test]
fn a_derived_query_value_cannot_change_the_path() {
    let section = SECTIONS.iter().find(|s| s.target == Target::WithMeAccountId).unwrap();
    for hostile in [
        "acct",
        "../../admin/settings",
        "x?y=1#z",
        "a&accountId=b",
        "%2e%2e/%2f",
        "/api/login",
        "näme 空 \u{0}",
    ] {
        let pq = path_and_query(section, Some(hostile));
        assert_eq!(pq.path(), section.path, "{hostile}");
        let query = pq.query().unwrap();
        assert_eq!(query.matches('=').count(), 1, "{hostile}: {query}");
        assert!(query.starts_with("accountId="));
        assert!(!query.contains(['&', '#', '/', '?']), "{hostile}: {query}");
    }
    for section in SECTIONS.iter().filter(|s| s.target == Target::Path) {
        assert_eq!(path_and_query(section, Some("x")).as_str(), section.path);
    }
}

#[test]
fn the_document_embeds_route_bytes_verbatim() {
    let outcome = |status: StatusCode, json: bool, body: &'static [u8], etag: Option<&str>| Outcome {
        status,
        etag: etag.map(str::to_owned),
        json,
        body: Some(Bytes::from_static(body)),
        set_cookie: Vec::new(),
    };
    let (bytes, tag) = document(&[
        ("me", outcome(StatusCode::OK, true, br#"{"b":1, "a":[2]}"#, Some("\"x\""))),
        ("text", outcome(StatusCode::BAD_GATEWAY, false, b"server error", None)),
        ("empty", outcome(StatusCode::NO_CONTENT, true, b"", None)),
        ("bad", outcome(StatusCode::OK, true, b"{\"a\":1}trailing", None)),
    ]);
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(text.contains(r#""me":{"status":200,"etag":"\"x\"","body":{"b":1, "a":[2]}}"#), "{text}");
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["v"], 1);
    assert_eq!(value["sections"]["text"]["body"], "server error");
    assert_eq!(value["sections"]["empty"]["body"], Value::Null);
    assert_eq!(value["sections"]["bad"]["body"], "{\"a\":1}trailing");
    assert!(tag.unwrap().starts_with("\"bs1-"));
    let mut large = outcome(StatusCode::OK, true, b"", None);
    large.body = None;
    let (bytes, tag) = document(&[("drafts", large)]);
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["sections"]["drafts"]["error"], "bodyTooLarge");
    assert!(tag.is_none(), "an incomplete document has no validator");
}

#[test]
fn only_the_me_csrf_cookie_is_forwarded() {
    let outcome = Outcome {
        status: StatusCode::OK,
        etag: None,
        json: true,
        body: Some(Bytes::new()),
        set_cookie: vec![
            HeaderValue::from_static("mw_csrf=abc; Path=/; SameSite=Strict"),
            HeaderValue::from_static("mw_session=stolen; Path=/; HttpOnly"),
            HeaderValue::from_static("mw_device=1; Path=/"),
        ],
    };
    let me = forwarded_cookies("me", &outcome);
    assert_eq!(me.len(), 1);
    assert!(me[0].to_str().unwrap().starts_with("mw_csrf="));
    for name in SECTIONS.iter().map(|s| s.name).filter(|n| *n != "me") {
        assert!(forwarded_cookies(name, &outcome).is_empty(), "{name}");
    }
}

#[test]
fn the_contract_keeps_bootstrap_on_http() {
    let contract: Value =
        serde_json::from_str(include_str!("../../../apps/web/src/api/websocket-action-contract.json"))
            .unwrap();
    let listed = contract
        .as_object()
        .unwrap()
        .values()
        .filter_map(|v| v.get("httpRequired").and_then(Value::as_array))
        .flatten()
        .any(|e| e["exact"] == "/api/bootstrap" && e["reason"] == "cookie-bootstrap");
    assert!(listed, "/api/bootstrap must be listed as HTTP-only cookie-bootstrap");
}

/// The differential matrix: 7 caller kinds x 19 sections.
#[test]
fn every_section_equals_its_direct_route_for_every_caller() {
    run(|| async {
        let mut summary = Vec::new();
        for &caller in CALLERS {
            let fixture = Fixture::new(caller).await;
            let headers = fixture.headers(caller);
            let (status, _, bytes) = fixture.send("/api/bootstrap", &headers).await;
            if caller == Caller::LinkedScope {
                // The linked-scope guard refuses the selector on this path before
                // the handler runs: no section is produced for a linked context.
                let body: Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
                assert_eq!(body["code"], "linkedContextUnsupported");
                assert!(body.get("sections").is_none());
                let (direct, _, direct_bytes) = fixture.send("/api/accounts", &headers).await;
                assert_eq!(direct, status);
                assert_eq!(direct_bytes, bytes, "the same refusal a primary-only route gets");
                summary.push(format!("{caller:?}: whole request 403 linkedContextUnsupported"));
                continue;
            }
            assert_eq!(status, StatusCode::OK, "{caller:?}: {}", String::from_utf8_lossy(&bytes));
            let document: Value = serde_json::from_slice(&bytes).unwrap();
            let sections = document["sections"].as_object().unwrap();
            let mut expected_names = ROWS.iter().map(|r| r.0).collect::<Vec<_>>();
            expected_names.sort_unstable();
            assert_eq!(
                sections.keys().map(String::as_str).collect::<Vec<_>>(),
                expected_names,
                "{caller:?}: every section"
            );
            // Order on the wire (serde_json's map here is sorted, so read bytes).
            let text = String::from_utf8_lossy(&bytes);
            let positions: Vec<usize> = ROWS
                .iter()
                .map(|(name, _)| text.find(&format!("\"{name}\":{{\"status\":")).unwrap())
                .collect();
            assert!(positions.windows(2).all(|w| w[0] < w[1]), "{caller:?}: section order");
            let me = sections["me"].clone();
            let mut statuses = Vec::new();
            for (name, path) in ROWS {
                let section = &sections[*name];
                let uri = direct_uri(name, path, &me);
                let (direct, direct_headers, direct_bytes) = fixture.send(&uri, &headers).await;
                assert_eq!(
                    section["status"], direct.as_u16(),
                    "{caller:?} {name}: section {section} vs direct {}",
                    String::from_utf8_lossy(&direct_bytes)
                );
                assert_eq!(
                    section["body"],
                    expected_body(&direct_headers, &direct_bytes),
                    "{caller:?} {name}"
                );
                if let Some(etag) = direct_headers.get(header::ETAG) {
                    assert_eq!(section["etag"], etag.to_str().unwrap(), "{caller:?} {name}");
                }
                // Beyond equality: no mailbox data for a caller without a
                // mailbox session, and only `me` for a held account.
                let public = matches!(*name, "publicConfig" | "cookiesConfig");
                let refused = match caller {
                    Caller::Anonymous | Caller::Admin => !public,
                    Caller::PasswordChangeHeld => !public && *name != "me",
                    _ => false,
                };
                if refused {
                    assert_ne!(section["status"], 200, "{caller:?} {name}: {section}");
                }
                statuses.push(format!("{name}={}", section["status"]));
            }
            if matches!(caller, Caller::Anonymous | Caller::Admin) {
                assert_eq!(sections["publicConfig"]["status"], 200);
                assert_eq!(sections["cookiesConfig"]["status"], 200);
            }
            summary.push(format!("{caller:?}: {}", statuses.join(" ")));
        }
        eprintln!("t52w-e6 differential matrix:\n{}", summary.join("\n"));
    });
}

#[test]
fn a_matching_validator_answers_304_and_headers_keep_it_private() {
    run(|| async {
        let fixture = Fixture::new(Caller::Normal).await;
        let headers = fixture.headers(Caller::Normal);
        let (status, first, bytes) = fixture.send("/api/bootstrap", &headers).await;
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&bytes));
        assert_eq!(first[header::CACHE_CONTROL], "private, no-cache");
        assert!(first.get_all(header::VARY).iter().any(|v| v.to_str().unwrap().contains("Cookie")));
        assert!(first.get(header::SET_COOKIE).is_none(), "the CSRF cookie was already current");
        let etag = first[header::ETAG].to_str().unwrap().to_owned();
        assert!(etag.starts_with("\"bs1-"), "{etag}");
        let mut conditional = headers.clone();
        conditional.insert(header::IF_NONE_MATCH, etag.parse().unwrap());
        let (status, again, bytes) = fixture.send("/api/bootstrap", &conditional).await;
        assert_eq!(status, StatusCode::NOT_MODIFIED);
        assert!(bytes.is_empty());
        assert_eq!(again[header::ETAG], etag.as_str());
        assert_eq!(again[header::CACHE_CONTROL], "private, no-cache");
        conditional.insert(header::IF_NONE_MATCH, "\"bs1-other\"".parse().unwrap());
        let (status, _, _) = fixture.send("/api/bootstrap", &conditional).await;
        assert_eq!(status, StatusCode::OK);
        // A query string or extra header cannot add, drop or steer a section.
        let mut steering = headers.clone();
        steering.insert("x-bootstrap-section", "/api/login".parse().unwrap());
        let (status, _, steered) = fixture
            .send("/api/bootstrap?path=/admin/settings&section=accounts&v=2", &steering)
            .await;
        assert_eq!(status, StatusCode::OK);
        let plain: Value = serde_json::from_slice(
            &fixture.send("/api/bootstrap", &headers).await.2,
        )
        .unwrap();
        let steered: Value = serde_json::from_slice(&steered).unwrap();
        assert_eq!(steered, plain);
    });
}

#[test]
fn the_me_csrf_cookie_is_forwarded_and_blocks_304() {
    run(|| async {
        let fixture = Fixture::new(Caller::Normal).await;
        let mut headers = fixture.headers(Caller::Normal);
        // A browser that has the session cookie but not yet its CSRF cookie.
        headers.insert("cookie", format!("mw_session={}", fixture.session).parse().unwrap());
        let (status, first, _) = fixture.send("/api/bootstrap", &headers).await;
        assert_eq!(status, StatusCode::OK);
        let cookies: Vec<_> = first
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_owned())
            .collect();
        assert_eq!(
            cookies.iter().filter(|c| c.starts_with("mw_csrf=")).count(),
            1,
            "{cookies:?}"
        );
        assert!(cookies.iter().all(|c| !c.starts_with("mw_session=")), "{cookies:?}");
        let etag = first[header::ETAG].to_str().unwrap().to_owned();
        headers.insert(header::IF_NONE_MATCH, etag.parse().unwrap());
        let (status, again, _) = fixture.send("/api/bootstrap", &headers).await;
        assert_eq!(status, StatusCode::OK, "a response that sets a cookie is never 304");
        assert!(again.get(header::SET_COOKIE).is_some());
    });
}

/// Records how many SQL statements one bootstrap costs today. Each section
/// runs its route's whole guard chain, so the cost is about the sum of the
/// direct calls plus the outer request's own chain.
#[test]
fn statements_per_bootstrap_are_recorded() {
    run(|| async {
        let fixture = Fixture::new(Caller::Normal).await;
        let headers = fixture.headers(Caller::Normal);
        let total = |s: mw_store::pool_telemetry::QuerySnapshot| {
            (s.read.calls, s.write.calls, s.other.calls)
        };
        let delta = |a: (u64, u64, u64), b: (u64, u64, u64)| (b.0 - a.0, b.1 - a.1, b.2 - a.2);
        // Warm caches once.
        assert_eq!(fixture.send("/api/bootstrap", &headers).await.0, StatusCode::OK);
        let mut boots = Vec::new();
        let mut directs = Vec::new();
        for _ in 0..3 {
            let before = total(mw_store::pool_telemetry::process_snapshot());
            let (status, _, bytes) = fixture.send("/api/bootstrap", &headers).await;
            let after = total(mw_store::pool_telemetry::process_snapshot());
            assert_eq!(status, StatusCode::OK);
            boots.push(delta(before, after));
            let me: Value = serde_json::from_slice::<Value>(&bytes).unwrap()["sections"]["me"].clone();
            let before = total(mw_store::pool_telemetry::process_snapshot());
            for (name, path) in ROWS {
                fixture.send(&direct_uri(name, path, &me), &headers).await;
            }
            let after = total(mw_store::pool_telemetry::process_snapshot());
            directs.push(delta(before, after));
        }
        let sum = |v: &(u64, u64, u64)| v.0 + v.1 + v.2;
        let boot = boots.iter().map(sum).min().unwrap();
        let direct = directs.iter().map(sum).min().unwrap();
        eprintln!(
            "t52w-e6 statements: bootstrap (read,write,other) {boots:?} min {boot}; \
             {} direct calls {directs:?} min {direct}",
            ROWS.len()
        );
        assert!(boot > 0);
        // Not an optimisation target here: the bootstrap may not cost less than
        // the routes it runs, and its own outer chain adds little.
        assert!(boot + 5 >= direct, "bootstrap {boot} vs direct {direct}");
    });
}

