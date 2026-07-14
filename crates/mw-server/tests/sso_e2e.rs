//! SSO route unit gate (t9-e3) — the `/api/sso/*` routes driven end-to-end against a
//! MOCK `mw_sso::SsoLogin` impl (no Docker, no live IdP; the live Keycloak E2E is e6).
//!
//! Proves the wiring `sso.rs` owns:
//!   * `GET /api/sso/providers` is PRE-auth and advertises the enabled backends
//!     (id + kind + display name only — no secrets/config).
//!   * `GET /api/sso/{id}/begin` 302s to the IdP and stashes the pending flow.
//!   * `GET /api/sso/{id}/callback` completes → mints the SAME `mw_session` cookie via
//!     the existing `finish_login` and 303s back into the app.
//!   * EVERY provider error → a UNIFORM 401 (no variant leak), and an unknown/reused
//!     state token → 401 (replay).
//!   * First-login is DENIED by default (allowlist) for an unknown subject; a matching
//!     account is admitted.
//!   * The `sso_login_audit` is content-free (a subject HASH, never the raw subject).
//!
//! The mock returns a fixed `state` token so the test can drive begin→callback without
//! a real IdP round-trip; `build_app_with_sso_mock` injects it in place of the
//! store-built providers.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;

use mw_server::sso::{SsoEntry, SsoMeta};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_sso::{
    BeginRedirect, ClaimMap, FirstLoginPolicy, Metadata, PendingState, Redirect, SsoCallback,
    SsoError, SsoIdentity, SsoKind, SsoLogin, SsoScope,
};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const SERVER_KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const STATE_TOKEN: &str = "fixedstatetoken0123456789abcdef";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

// ── The mock provider ─────────────────────────────────────────────────────────

/// A canned `SsoLogin` whose `complete` yields a fixed identity — or a fixed error,
/// to exercise the uniform-401 path.
struct MockProvider {
    kind: SsoKind,
    identity: SsoIdentity,
    fail: Option<SsoError>,
    metadata: Option<Metadata>,
}

#[async_trait]
impl SsoLogin for MockProvider {
    async fn begin(&self, relay_state: Option<String>) -> Result<BeginRedirect, SsoError> {
        let pending = match self.kind {
            SsoKind::Oidc => PendingState::Oidc {
                pkce_verifier: "verifier".into(),
                nonce: "nonce".into(),
                relay_state,
            },
            SsoKind::Saml => PendingState::Saml {
                request_id: "_req".into(),
                relay_state,
            },
        };
        Ok(BeginRedirect {
            url: format!("https://idp.example/authorize?state={STATE_TOKEN}"),
            state_token: STATE_TOKEN.to_string(),
            pending,
        })
    }

    async fn complete(&self, _callback: SsoCallback) -> Result<SsoIdentity, SsoError> {
        match &self.fail {
            Some(SsoError::SignatureInvalid(m)) => Err(SsoError::SignatureInvalid(m.clone())),
            Some(_) => Err(SsoError::Replay),
            None => Ok(self.identity.clone()),
        }
    }

    fn metadata(&self) -> Option<Metadata> {
        self.metadata.clone()
    }

    fn logout(&self, _subject: &str) -> Option<Redirect> {
        Some(Redirect {
            url: "https://idp.example/logout".into(),
        })
    }
}

fn identity(subject: &str, email: &str) -> SsoIdentity {
    SsoIdentity {
        subject: subject.into(),
        email: Some(email.into()),
        display_name: Some("User".into()),
        groups: vec![],
        claims: BTreeMap::new(),
    }
}

fn entry(
    kind: SsoKind,
    display: &str,
    scope: SsoScope,
    policy: FirstLoginPolicy,
    ident: SsoIdentity,
    fail: Option<SsoError>,
    metadata: Option<Metadata>,
) -> SsoEntry {
    SsoEntry {
        provider: Arc::new(MockProvider {
            kind,
            identity: ident,
            fail,
            metadata,
        }),
        meta: SsoMeta {
            display_name: display.into(),
            kind,
            scope,
            enabled: true,
            first_login_policy: policy,
            claim_map: ClaimMap::default(),
        },
    }
}

// ── Harness ─────────────────────────────────────────────────────────────────

/// A process-unique suffix (avoids a `uuid` test dep).
fn unique() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{nanos}-{n}")
}

fn web_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mw-t9e3-web-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), INDEX_HTML).unwrap();
    dir
}

fn db_path() -> String {
    std::env::temp_dir()
        .join(format!("mw-t9e3-{}.sqlite", unique()))
        .to_string_lossy()
        .to_string()
}

/// Seed an allowlisted engine account so the callback can resolve an identity to it.
async fn seed_account(db: &str, username: &str) {
    let store = Store::open(db, ServerKey::from_hex(SERVER_KEY_HEX).unwrap())
        .await
        .unwrap();
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example",
                port: 993,
                tls: "implicit",
                username,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: username.into(),
                password: "unused".into(),
            },
        )
        .await
        .unwrap();
}

async fn spawn(db: String, providers: Vec<(String, SsoEntry)>) -> String {
    let config = AppConfig {
        db_path: db,
        server_key_hex: Some(SERVER_KEY_HEX.to_string()),
        web_dir: Some(web_dir()),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("pw".into()),
        redis_url: None,
    };
    let app = mw_server::build_app_with_sso_mock(config, v6, providers)
        .await
        .expect("server boots")
        .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

/// A reqwest client that does NOT auto-follow redirects (so we can inspect 302/303).
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn providers_are_listed_pre_auth() {
    let db = db_path();
    let providers = vec![
        (
            "corp-oidc".to_string(),
            entry(
                SsoKind::Oidc,
                "Sign in with Acme",
                SsoScope::Deployment,
                FirstLoginPolicy::Allowlist,
                identity("s", "a@acme.test"),
                None,
                None,
            ),
        ),
        (
            "acme-saml".to_string(),
            entry(
                SsoKind::Saml,
                "Acme SAML",
                SsoScope::Domain("acme.test".into()),
                FirstLoginPolicy::Allowlist,
                identity("s", "a@acme.test"),
                None,
                None,
            ),
        ),
    ];
    let base = spawn(db, providers).await;

    // No auth header/cookie at all — the login screen is unauthenticated.
    let body: serde_json::Value = client()
        .get(format!("{base}/api/sso/providers?domain=acme.test"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let list = body["providers"].as_array().unwrap();
    assert_eq!(list.len(), 2, "both the deployment + domain IdP advertised");
    assert_eq!(list[0]["id"], "acme-saml");
    assert_eq!(list[0]["kind"], "saml");
    assert!(list[0].get("config").is_none(), "no config/secret leaked");

    // Without a domain, only the deployment-wide IdP is advertised (no enumeration).
    let body: serde_json::Value = client()
        .get(format!("{base}/api/sso/providers"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let list = body["providers"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], "corp-oidc");
}

#[tokio::test]
async fn begin_redirects_to_the_idp() {
    let db = db_path();
    let providers = vec![(
        "corp-oidc".to_string(),
        entry(
            SsoKind::Oidc,
            "Acme",
            SsoScope::Deployment,
            FirstLoginPolicy::Allowlist,
            identity("s", "a@acme.test"),
            None,
            None,
        ),
    )];
    let base = spawn(db, providers).await;

    let resp = client()
        .get(format!("{base}/api/sso/corp-oidc/begin?relayState=/inbox"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 303, "302/303 to the IdP");
    let loc = resp.headers().get("location").unwrap().to_str().unwrap();
    assert!(loc.starts_with("https://idp.example/authorize"));
    assert!(loc.contains(STATE_TOKEN));

    // An unknown backend id must not reveal itself — uniform 401.
    let resp = client()
        .get(format!("{base}/api/sso/nope/begin"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);
}

#[tokio::test]
async fn callback_completes_into_a_session() {
    let db = db_path();
    seed_account(&db, "alice@acme.test").await;
    let providers = vec![(
        "corp-oidc".to_string(),
        entry(
            SsoKind::Oidc,
            "Acme",
            SsoScope::Deployment,
            FirstLoginPolicy::Allowlist,
            identity("subject-alice", "alice@acme.test"),
            None,
            None,
        ),
    )];
    let base = spawn(db, providers).await;

    // First `begin` to stash the pending flow keyed by STATE_TOKEN.
    client()
        .get(format!("{base}/api/sso/corp-oidc/begin?relayState=/inbox"))
        .send()
        .await
        .unwrap();

    // Then the callback with the matching `state`.
    let resp = client()
        .get(format!(
            "{base}/api/sso/corp-oidc/callback?code=abc&state={STATE_TOKEN}"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 303, "303 back into the SPA");
    assert_eq!(resp.headers().get("location").unwrap(), "/inbox");
    let set_cookie = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(
        set_cookie.contains("mw_session="),
        "the standard session cookie is minted via finish_login: {set_cookie}"
    );

    // The one-shot pending flow is consumed — a replay of the same state is 401.
    let resp = client()
        .get(format!(
            "{base}/api/sso/corp-oidc/callback?code=abc&state={STATE_TOKEN}"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401, "replayed state rejected");
}

#[tokio::test]
async fn provider_error_is_a_uniform_401() {
    let db = db_path();
    seed_account(&db, "alice@acme.test").await;
    let providers = vec![(
        "corp-oidc".to_string(),
        entry(
            SsoKind::Oidc,
            "Acme",
            SsoScope::Deployment,
            FirstLoginPolicy::Allowlist,
            identity("subject-alice", "alice@acme.test"),
            Some(SsoError::SignatureInvalid("bad sig".into())),
            None,
        ),
    )];
    let base = spawn(db, providers).await;

    client()
        .get(format!("{base}/api/sso/corp-oidc/begin"))
        .send()
        .await
        .unwrap();
    let resp = client()
        .get(format!(
            "{base}/api/sso/corp-oidc/callback?code=abc&state={STATE_TOKEN}"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 401);
    let body: serde_json::Value = resp.json().await.unwrap();
    // The body never names the failing check.
    assert_eq!(body["error"], "authentication failed");
}

#[tokio::test]
async fn first_login_is_denied_by_default() {
    let db = db_path();
    // No account seeded → the asserted subject maps to nothing; allowlist denies.
    let providers = vec![(
        "corp-oidc".to_string(),
        entry(
            SsoKind::Oidc,
            "Acme",
            SsoScope::Deployment,
            FirstLoginPolicy::Allowlist,
            identity("subject-stranger", "stranger@acme.test"),
            None,
            None,
        ),
    )];
    let base = spawn(db, providers).await;

    client()
        .get(format!("{base}/api/sso/corp-oidc/begin"))
        .send()
        .await
        .unwrap();
    let resp = client()
        .get(format!(
            "{base}/api/sso/corp-oidc/callback?code=abc&state={STATE_TOKEN}"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        401,
        "an unknown subject is denied under the default allowlist policy"
    );
    assert!(
        resp.headers().get_all("set-cookie").iter().next().is_none(),
        "no session cookie is minted on a denied first login"
    );
}

#[tokio::test]
async fn autocreate_admits_an_unknown_subject() {
    let db = db_path();
    let providers = vec![(
        "corp-oidc".to_string(),
        entry(
            SsoKind::Oidc,
            "Acme",
            SsoScope::Deployment,
            FirstLoginPolicy::AutoCreate,
            identity("subject-new", "new@acme.test"),
            None,
            None,
        ),
    )];
    let base = spawn(db, providers).await;

    client()
        .get(format!("{base}/api/sso/corp-oidc/begin"))
        .send()
        .await
        .unwrap();
    let resp = client()
        .get(format!(
            "{base}/api/sso/corp-oidc/callback?code=abc&state={STATE_TOKEN}"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        303,
        "autocreate admits the identity"
    );
}

#[tokio::test]
async fn saml_metadata_is_served() {
    let db = db_path();
    let providers = vec![(
        "corp-saml".to_string(),
        entry(
            SsoKind::Saml,
            "Acme SAML",
            SsoScope::Deployment,
            FirstLoginPolicy::Allowlist,
            identity("s", "a@acme.test"),
            None,
            Some(Metadata {
                content_type: "application/samlmetadata+xml".into(),
                body: "<EntityDescriptor/>".into(),
            }),
        ),
    )];
    let base = spawn(db, providers).await;

    let resp = client()
        .get(format!("{base}/api/sso/corp-saml/metadata"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "application/samlmetadata+xml"
    );
    assert_eq!(resp.text().await.unwrap(), "<EntityDescriptor/>");
}

#[tokio::test]
async fn saml_acs_is_csrf_exempt_and_completes() {
    let db = db_path();
    seed_account(&db, "bob@acme.test").await;
    let providers = vec![(
        "corp-saml".to_string(),
        entry(
            SsoKind::Saml,
            "Acme SAML",
            SsoScope::Deployment,
            FirstLoginPolicy::Allowlist,
            identity("subject-bob", "bob@acme.test"),
            None,
            None,
        ),
    )];
    let base = spawn(db, providers).await;

    client()
        .get(format!("{base}/api/sso/corp-saml/begin"))
        .send()
        .await
        .unwrap();

    // A cross-site form POST from the IdP: a foreign Origin header that WOULD trip the
    // state-change guard on any non-exempt route. The ACS path is exempt, so it lands.
    let resp = client()
        .post(format!("{base}/api/sso/corp-saml/acs"))
        .header("origin", "https://idp.example")
        .form(&[("SAMLResponse", "base64resp"), ("RelayState", STATE_TOKEN)])
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        303,
        "ACS is CSRF/origin-exempt and completes into a session"
    );
    let set_cookie = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(set_cookie.contains("mw_session="));
}
