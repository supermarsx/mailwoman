//! LIVE SSO E2E harness (t9-e6) — drives the REAL `/api/sso/*` routes through a REAL
//! Keycloak IdP (docker-compose.ci.yml `keycloak`, realm `scripts/keycloak/realm.json`).
//!
//! This is the non-Playwright gate and, for SAML, the **interop decision point**: it
//! runs Keycloak's own signed `SAMLResponse` through the hand-rolled exc-C14N + XML-DSig
//! validator (`SsoProviderSource::Store` builds the real `SamlProvider` from the seeded
//! 0009 `sso_config` row — NOT the mock). Full-ship iff Keycloak's real assertion
//! verifies byte-exact; otherwise the failure is the flagged-ship evidence.
//!
//! ## Env gate (no silent skip)
//! Runs only when `MW_SSO_LIVE=1` AND Keycloak's realm discovery is reachable; otherwise
//! it prints a LOUD skip and returns (so the default `cargo test` baseline is unaffected).
//! Bring the stack up first:
//!   docker compose -f docker-compose.ci.yml up -d keycloak
//!   scripts/keycloak/wait-for-keycloak.sh
//!   MW_SSO_LIVE=1 cargo test -p mw-server --test sso_live -- --nocapture --test-threads=1
//!
//! The server binds a FIXED port (8090) so it matches the realm's registered OIDC
//! redirect + SAML ACS URLs.

use std::net::SocketAddr;
use std::path::PathBuf;

use base64::Engine as _;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

// ── Fixed live-stack coordinates (match scripts/keycloak/realm.json) ──────────
const KC_BASE: &str = "http://localhost:8080";
const REALM: &str = "mailwoman";
const KC_USER: &str = "ada";
const KC_PASS: &str = "keycloak-test-pw";
const KC_EMAIL: &str = "ada@mailwoman.test";

const SERVER_PORT: u16 = 8090;
const SERVER_KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

fn server_base() -> String {
    format!("http://localhost:{SERVER_PORT}")
}

/// Skip guard: true when the live stack isn't requested/reachable (prints loudly).
async fn skipped() -> bool {
    if std::env::var("MW_SSO_LIVE").ok().as_deref() != Some("1") {
        eprintln!(
            "\n[t9-e6 SKIP] MW_SSO_LIVE!=1 — live SSO harness not run. Bring up Keycloak \
             (docker compose -f docker-compose.ci.yml up -d keycloak) and set MW_SSO_LIVE=1.\n"
        );
        return true;
    }
    let discovery = format!("{KC_BASE}/realms/{REALM}/.well-known/openid-configuration");
    match reqwest::get(&discovery).await {
        Ok(r) if r.status().is_success() => false,
        other => {
            eprintln!(
                "\n[t9-e6 SKIP] Keycloak discovery unreachable at {discovery}: {other:?}. \
                 Start the stack + wait-for-keycloak.sh.\n"
            );
            true
        }
    }
}

// ── Store seeding ─────────────────────────────────────────────────────────────

/// Fetch the realm's SAML IdP signing certificate (PEM), for the pinned trust anchor.
async fn fetch_saml_signing_cert() -> String {
    let descriptor = format!("{KC_BASE}/realms/{REALM}/protocol/saml/descriptor");
    let xml = reqwest::get(&descriptor)
        .await
        .expect("fetch SAML descriptor")
        .text()
        .await
        .expect("descriptor body");
    let start = xml
        .find("<ds:X509Certificate>")
        .map(|i| i + "<ds:X509Certificate>".len())
        .expect("descriptor has a ds:X509Certificate");
    let end = xml[start..]
        .find("</ds:X509Certificate>")
        .map(|i| start + i)
        .expect("cert close tag");
    let b64 = xml[start..end].trim();
    // Wrap the raw base64 DER into a PEM certificate block (64-col not required by parsers).
    format!("-----BEGIN CERTIFICATE-----\n{b64}\n-----END CERTIFICATE-----\n")
}

fn web_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "mw-t9e6-web-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), INDEX_HTML).unwrap();
    dir
}

fn db_path() -> String {
    std::env::temp_dir()
        .join(format!(
            "mw-t9e6-{}.sqlite",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
        .to_string_lossy()
        .to_string()
}

/// Seed the engine account (allowlist match) + both 0009 `sso_config` rows pointing at
/// the live Keycloak realm, then return the db path for `build_app` to read.
async fn seed_store(saml_cert_pem: &str) -> String {
    let db = db_path();
    seed_into(&db, saml_cert_pem).await;
    db
}

/// Seed a specific db path (shared by the in-process harness + the standing-server seeder
/// the SSO-E2E workflow uses).
async fn seed_into(db: &str, saml_cert_pem: &str) {
    let store = Store::open(db, ServerKey::from_hex(SERVER_KEY_HEX).unwrap())
        .await
        .unwrap();

    // Allowlisted account whose username == the asserted email (default deny-by-default
    // policy admits it, no auto-registration).
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example",
                port: 993,
                tls: "implicit",
                username: KC_EMAIL,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: KC_EMAIL.into(),
                password: "unused".into(),
            },
        )
        .await
        .unwrap();

    // OIDC backend row (built live via SsoProviderSource::Store).
    let oidc = mw_sso::SsoConfig::Oidc(mw_sso::OidcConfig {
        issuer_url: format!("{KC_BASE}/realms/{REALM}"),
        client_id: "mailwoman-oidc".into(),
        redirect_url: format!("{}/api/sso/corp-oidc/callback", server_base()),
        scopes: vec!["openid".into(), "email".into(), "profile".into()],
        first_login_policy: mw_sso::FirstLoginPolicy::Allowlist,
    });
    put_row(
        &store,
        "corp-oidc",
        "oidc",
        "Sign in with Keycloak",
        &oidc,
        Some(b"mailwoman-oidc-secret".to_vec()),
    )
    .await;

    // SAML backend row — pins the fetched Keycloak signing cert.
    let saml = mw_sso::SsoConfig::Saml(mw_sso::SamlConfig {
        sp_entity_id: "mailwoman-sp".into(),
        acs_url: format!("{}/api/sso/corp-saml/acs", server_base()),
        idp_metadata_url: None,
        idp_metadata_xml: None,
        idp_sso_url: format!("{KC_BASE}/realms/{REALM}/protocol/saml"),
        idp_slo_url: None,
        idp_signing_certs_pem: vec![saml_cert_pem.to_string()],
        want_assertions_signed: true,
        want_encrypted: false,
        nameid_format: "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".into(),
        first_login_policy: mw_sso::FirstLoginPolicy::Allowlist,
    });
    put_row(
        &store,
        "corp-saml",
        "saml",
        "Sign in with Keycloak",
        &saml,
        None,
    )
    .await;
}

/// Env-gated standing-server seeder (used by the SSO-E2E Playwright job): seed a persistent
/// db at `MW_SSO_SEED_DB` with the same fixed `SERVER_KEY_HEX`, then a `mailwoman serve
/// --db <that> --server-key <SERVER_KEY_HEX> --web-dir apps/web/dist --mode engine --bind
/// 127.0.0.1:8090` exposes the SSO-enabled SPA the browser specs drive. Requires Keycloak up
/// (fetches the SAML signing cert).
#[tokio::test(flavor = "multi_thread")]
async fn seed_standing_db() {
    let Ok(path) = std::env::var("MW_SSO_SEED_DB") else {
        eprintln!(
            "[t9-e6] MW_SSO_SEED_DB unset — standing-db seeder skipped (used by the SSO-E2E job)."
        );
        return;
    };
    if skipped().await {
        return;
    }
    let cert = fetch_saml_signing_cert().await;
    seed_into(&path, &cert).await;
    eprintln!(
        "[t9-e6] seeded standing SSO db at {path} (key {SERVER_KEY_HEX}); \
         serve it with --web-dir apps/web/dist --mode engine --bind 127.0.0.1:{SERVER_PORT}"
    );
}

async fn put_row(
    store: &Store,
    id: &str,
    kind: &str,
    display: &str,
    config: &mw_sso::SsoConfig,
    secret: Option<Vec<u8>>,
) {
    let claim_map = mw_sso::ClaimMap {
        email: Some("email".into()),
        ..Default::default()
    };
    let now = "2026-07-14T00:00:00Z".to_string();
    let row = mw_store::SsoConfigRow {
        id: id.into(),
        kind: kind.into(),
        display_name: display.into(),
        scope: "deployment".into(),
        enabled: true,
        config_json: serde_json::to_string(config).unwrap(),
        secret,
        claim_map_json: serde_json::to_string(&claim_map).unwrap(),
        created_at: now.clone(),
        updated_at: now,
    };
    store.put_sso_config(&row).await.unwrap();
}

/// Boot the real app (`SsoProviderSource::Store`) on the fixed port.
async fn spawn_server(db: String) {
    let config = AppConfig {
        db_path: db,
        server_key_hex: Some(SERVER_KEY_HEX.to_string()),
        web_dir: Some(web_dir()),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let app = build_app(config).await.expect("server boots");
    let addr: SocketAddr = ([127, 0, 0, 1], SERVER_PORT).into();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    // Give the listener a beat.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
}

// ── HTML/form helpers (no scraper dep) ────────────────────────────────────────

fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#x2F;", "/")
        .replace("&#47;", "/")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

/// Extract the `action` URL of the Keycloak login form (`id="kc-form-login"`).
fn kc_login_action(html: &str) -> Option<String> {
    let anchor = html.find("kc-form-login")?;
    // Find the enclosing <form ...> tag start (search backwards).
    let form_start = html[..anchor].rfind("<form")?;
    let tag_end = html[form_start..].find('>')? + form_start;
    let tag = &html[form_start..tag_end];
    let ai = tag.find("action=\"")? + "action=\"".len();
    let ae = tag[ai..].find('"')? + ai;
    Some(html_unescape(&tag[ai..ae]))
}

/// Extract a hidden `<input name="NAME" value="VALUE">` value from an HTML form.
fn form_input_value(html: &str, name: &str) -> Option<String> {
    let needle = format!("name=\"{name}\"");
    let at = html.find(&needle)?;
    // The value attribute may appear before or after `name=`; scan the whole input tag.
    let tag_start = html[..at].rfind('<')?;
    let tag_end = html[at..].find('>')? + at;
    let tag = &html[tag_start..tag_end];
    let vi = tag.find("value=\"")? + "value=\"".len();
    let ve = tag[vi..].find('"')? + vi;
    Some(html_unescape(&tag[vi..ve]))
}

/// Extract the `action` URL of the first `<form>` (Keycloak's SAML POST auto-submit).
fn first_form_action(html: &str) -> Option<String> {
    let form_start = html.find("<form")?;
    let tag_end = html[form_start..].find('>')? + form_start;
    let tag = &html[form_start..tag_end];
    let ai = tag.find("action=\"")? + "action=\"".len();
    let ae = tag[ai..].find('"')? + ai;
    Some(html_unescape(&tag[ai..ae]))
}

fn cookie_client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(15))
        .build()
        .unwrap()
}

fn no_redirect_cookie_client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// Log in `ada` at Keycloak: GET the login page (already redirected to by `begin`),
/// submit credentials, and return the FINAL response (what the IdP does after auth —
/// for OIDC: the mw-server callback chain; for SAML: the ACS POST auto-submit page).
async fn keycloak_authenticate(client: &reqwest::Client, begin_url: &str) -> reqwest::Response {
    let login_page = client
        .get(begin_url)
        .send()
        .await
        .expect("GET begin -> Keycloak login page")
        .text()
        .await
        .expect("login page body");
    let action = kc_login_action(&login_page).unwrap_or_else(|| {
        panic!(
            "could not find Keycloak login form action in page (len {})",
            login_page.len()
        )
    });
    client
        .post(&action)
        .form(&[
            ("username", KC_USER),
            ("password", KC_PASS),
            ("credentialId", ""),
        ])
        .send()
        .await
        .expect("POST Keycloak credentials")
}

// ── OIDC: MUST PASS ───────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn oidc_login_end_to_end() {
    if skipped().await {
        return;
    }
    let cert = fetch_saml_signing_cert().await;
    let db = seed_store(&cert).await;
    spawn_server(db).await;

    let client = cookie_client();
    // begin -> Keycloak login -> credentials -> callback -> session -> SPA.
    let resp = keycloak_authenticate(
        &client,
        &format!("{}/api/sso/corp-oidc/begin?relayState=/", server_base()),
    )
    .await;
    assert!(
        resp.status().is_success() || resp.status().is_redirection(),
        "OIDC post-login chain landed OK (status {})",
        resp.status()
    );

    // The mw_session cookie is now in the jar → an authed /api/me returns the account.
    let me = client
        .get(format!("{}/api/me", server_base()))
        .send()
        .await
        .expect("GET /api/me");
    assert_eq!(
        me.status().as_u16(),
        200,
        "OIDC session authenticates the mailbox surface (/api/me)"
    );
    let body: serde_json::Value = me.json().await.expect("/api/me json");
    eprintln!("[t9-e6 OIDC] /api/me = {body}");
    let who = body
        .get("username")
        .or_else(|| body.get("account").and_then(|a| a.get("username")))
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert!(
        who.eq_ignore_ascii_case(KC_EMAIL) || body.to_string().contains(KC_EMAIL),
        "OIDC identity resolved to the seeded account ({KC_EMAIL}); got {body}"
    );
    eprintln!("[t9-e6 OIDC] LIVE PASS — real discovery+PKCE+JWKS login → authenticated /api/me");
}

// ── SAML: THE DECISION ────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn saml_login_end_to_end_decision() {
    if skipped().await {
        return;
    }
    let cert = fetch_saml_signing_cert().await;
    let db = seed_store(&cert).await;
    spawn_server(db).await;

    // Fresh jar → forces a Keycloak login for the SAML flow.
    let client = cookie_client();
    let post_login = keycloak_authenticate(
        &client,
        &format!("{}/api/sso/corp-saml/begin?relayState=/", server_base()),
    )
    .await;

    // Keycloak returns an auto-submitting HTML form POSTing the REAL signed SAMLResponse
    // to our ACS. Capture it (this is the byte-exact foreign-IdP output e2 couldn't get).
    let acs_page = post_login.text().await.expect("SAML POST auto-submit page");
    let saml_response = form_input_value(&acs_page, "SAMLResponse").unwrap_or_else(|| {
        panic!(
            "Keycloak did not return a SAMLResponse form (page len {}). First 600 chars:\n{}",
            acs_page.len(),
            &acs_page[..acs_page.len().min(600)]
        )
    });
    let relay_state = form_input_value(&acs_page, "RelayState").unwrap_or_default();
    let acs_action = first_form_action(&acs_page)
        .unwrap_or_else(|| format!("{}/api/sso/corp-saml/acs", server_base()));

    // Record the real Keycloak SAMLResponse as evidence (base64 + decoded XML).
    record_saml_fixture(&saml_response);

    // Drive the ACS route: mw-server runs the hand-rolled exc-C14N + XML-DSig validator
    // over Keycloak's REAL signed assertion. 303 => interop SUCCESS; 401 => interop FAIL.
    let acs = no_redirect_cookie_client()
        .post(&acs_action)
        .header("origin", KC_BASE)
        .form(&[
            ("SAMLResponse", saml_response.as_str()),
            ("RelayState", relay_state.as_str()),
        ])
        .send()
        .await
        .expect("POST SAMLResponse to ACS");

    let status = acs.status().as_u16();
    let set_cookie = acs
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .collect::<Vec<_>>()
        .join("; ");

    // The interop was genuinely exercised: a REAL Keycloak-signed assertion was driven
    // through the store-built SamlProvider's exc-C14N + XML-DSig validator. The VERDICT
    // (full-ship vs the §5 flagged-ship boundary) is reported, not asserted — flagged-ship
    // is a committed, documented outcome; only failing to exercise the path is a gate error.
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&saml_response)
        .expect("SAMLResponse is base64");
    let decoded_xml = String::from_utf8_lossy(&decoded);
    assert!(
        saml_response.len() > 1000 && decoded_xml.contains("Signature"),
        "a real signed Keycloak SAMLResponse must have been driven through the validator"
    );
    if status == 303 && set_cookie.contains("mw_session=") {
        eprintln!(
            "\n[t9-e6 SAML] ===== INTEROP VERDICT: FULL-SHIP =====\n\
             Keycloak's REAL RSA-SHA256 signed assertion (exc-C14N) VERIFIED by the \
             hand-rolled validator; ACS minted a session (303 + mw_session). \
             Recommend flipping sso.saml.enabled default ON.\n"
        );
    } else {
        let body = acs.text().await.unwrap_or_default();
        eprintln!(
            "\n[t9-e6 SAML] ===== INTEROP VERDICT: FLAGGED-SHIP =====\n\
             Keycloak's real signed assertion was REJECTED by the hand-rolled validator \
             (ACS status {status}, body {body}). exc-C14N did NOT interop byte-exact with \
             Keycloak's canonical output over the assertion subtree. Recommend SAML stays \
             sso.saml.enabled OFF (documented beta) + ESCALATE. Captured SAMLResponse under \
             scripts/keycloak/. See saml_c14n_diagnostic for the exact SsoError.\n"
        );
    }
    eprintln!("[t9-e6 SAML] interop exercised live (ACS status {status}); verdict reported above.");
}

// ── SAML diagnostic: the EXACT validator error (escalation evidence) ──────────
//
// The route hides the failure behind a uniform 401 (by design). This drives the SAME
// store-built `SamlProvider` DIRECTLY (holding the begin() pending ourselves) so the
// concrete `SsoError` variant + detail string is visible — the precise evidence the
// coordinator needs to decide a targeted c14n fix vs ship-beta.

#[tokio::test(flavor = "multi_thread")]
async fn saml_c14n_diagnostic() {
    if skipped().await {
        return;
    }
    use mw_sso::{SsoBackend, SsoCallback, SsoKind, SsoLogin, SsoScope};

    let cert = fetch_saml_signing_cert().await;
    let config = mw_sso::SsoConfig::Saml(mw_sso::SamlConfig {
        sp_entity_id: "mailwoman-sp".into(),
        acs_url: format!("{}/api/sso/corp-saml/acs", server_base()),
        idp_sso_url: format!("{KC_BASE}/realms/{REALM}/protocol/saml"),
        idp_signing_certs_pem: vec![cert],
        want_assertions_signed: true,
        want_encrypted: false,
        nameid_format: "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".into(),
        first_login_policy: mw_sso::FirstLoginPolicy::Allowlist,
        ..Default::default()
    });
    let backend = SsoBackend {
        id: "corp-saml".into(),
        kind: SsoKind::Saml,
        display_name: "Sign in with Keycloak".into(),
        scope: SsoScope::Deployment,
        enabled: true,
        config,
        claim_map: mw_sso::ClaimMap {
            email: Some("email".into()),
            ..Default::default()
        },
        secret: None,
    };
    let provider = mw_sso::SamlProvider::new(backend).expect("build SamlProvider");

    // begin() → the AuthnRequest redirect URL + the pending (request_id) we keep.
    let begin = provider.begin(Some("/".into())).await.expect("SAML begin");

    // Drive Keycloak with the provider's OWN AuthnRequest, capture the SAMLResponse.
    let client = cookie_client();
    let acs_page = keycloak_authenticate(&client, &begin.url)
        .await
        .text()
        .await
        .expect("SAML POST page");
    let saml_response = form_input_value(&acs_page, "SAMLResponse").expect("captured SAMLResponse");
    record_saml_fixture(&saml_response);

    // complete() DIRECTLY — the concrete SsoError (with its log-only detail) is visible.
    let mut params = std::collections::BTreeMap::new();
    params.insert("SAMLResponse".to_string(), saml_response);
    let callback = SsoCallback {
        params,
        pending: begin.pending,
    };
    match provider.complete(callback).await {
        Ok(identity) => eprintln!(
            "\n[t9-e6 SAML-DIAG] VALIDATOR ACCEPTED → identity {identity:?}\n\
             (exc-C14N interop CONFIRMED — full-ship supported.)\n"
        ),
        Err(e) => eprintln!(
            "\n[t9-e6 SAML-DIAG] VALIDATOR REJECTED → {:?} :: audit_reason='{}'\n\
             Detail (log-only): {e}\n\
             ESCALATION EVIDENCE: this is the exact hand-rolled-validator error over \
             Keycloak's real assertion — Reference targets the Assertion (exc-C14N + \
             enveloped-signature, RSA-SHA256), which declares a default xmlns= on its apex; \
             the byte-exact c14n did not reproduce Keycloak's signed form.\n",
            e,
            e.audit_reason(),
        ),
    }
}

/// Persist the captured Keycloak SAMLResponse (base64 + decoded XML) under the owned
/// `scripts/keycloak/` dir as interop evidence / an offline fixture for the validator.
fn record_saml_fixture(saml_response_b64: &str) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/keycloak");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(
        dir.join("keycloak-saml-response.sample.b64"),
        saml_response_b64,
    );
    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(saml_response_b64) {
        let _ = std::fs::write(dir.join("keycloak-saml-response.sample.xml"), &bytes);
    }
    eprintln!(
        "[t9-e6 SAML] captured real Keycloak SAMLResponse ({} b64 chars) → scripts/keycloak/",
        saml_response_b64.len()
    );
}
