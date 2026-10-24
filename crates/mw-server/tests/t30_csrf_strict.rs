//! t30-e2 — strict CSRF mode (`MW_CSRF_STRICT=true`) over the real router.
//!
//! `deploy.sh` writes `MW_CSRF_STRICT=true` into every new installation. Until
//! 26.20 that mode refused `POST /admin/login` itself (the route was not exempt and
//! nothing issued the admin panel a token), so a default installation could not be
//! administered. These tests hold what strict mode must do:
//!
//! * a sign-in route is not asked for a token, and its answer issues one;
//! * every other write a cookie authorises is refused without the token and
//!   accepted with it — admin panel and mailbox alike;
//! * a route no cookie authorises (OAuth token endpoint, client registration,
//!   inbound webhook, error report, MCP) answers exactly as it does with strict
//!   mode off;
//! * the Origin check is unchanged: a cross-origin request to an exempt route is
//!   still refused.
//! * an `Authorization: Bearer` header excuses a request from both checks only
//!   when no session cookie comes with it. The admin gates authorise by cookie and
//!   never read that header, so until 26.20 any bearer header beside the admin
//!   cookie switched both checks off.
//!
//! Every refusal is paired with the same request succeeding, so a test cannot pass
//! because a route was unreachable. The client keeps no cookie jar: each request
//! names the cookies it sends, so "has the cookie, lacks the header" is explicit.
//!
//! SQLite only. The guard runs before any handler and reads no store.

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};
use tokio::net::TcpListener;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

mod common;
use common::test_db;

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";
const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "hunter2";

/// The body of the guard's token refusal (`state_change_guard`).
const CSRF_REFUSAL: &str = "missing or invalid CSRF token";
/// The body of the guard's Origin refusal.
const ORIGIN_REFUSAL: &str = "cross-origin request rejected";

// ── servers ────────────────────────────────────────────────────────────────────

async fn spawn_mock() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, mw_mock_jmap::router()).await;
    });
    format!("http://{addr}")
}

/// Boot the real app with the admin panel on. `upstream` is the JMAP mock a
/// proxy-mode server may dial; engine mode (the only mode that mounts `/mcp`)
/// takes none.
async fn spawn_server(mode: ServerMode, strict: bool, upstream: Option<&str>) -> String {
    let dir = test_db::unique_dir("mw-t30-csrf");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode,
        hardening: HardeningConfig {
            csrf_strict: strict,
            ..HardeningConfig::default()
        },
        security: SecurityConfig {
            jmap_upstreams: upstream.map(|u| vec![u.to_string()]),
            ..SecurityConfig::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some(ADMIN_USER.into()),
        admin_password: Some(ADMIN_PASS.into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    format!("http://{addr}")
}

/// No cookie jar and no redirects: a request sends what the test gives it.
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

// ── cookies ────────────────────────────────────────────────────────────────────

/// Every `Set-Cookie` value of a response, whole.
fn set_cookies(resp: &reqwest::Response) -> Vec<String> {
    resp.headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect()
}

/// The whole `Set-Cookie` value for `name`, if the response set that cookie.
fn set_cookie_for<'a>(set: &'a [String], name: &str) -> Option<&'a str> {
    let prefix = format!("{name}=");
    set.iter()
        .map(String::as_str)
        .find(|c| c.starts_with(&prefix))
}

/// The value of the cookie `name` in a `Set-Cookie` list.
fn cookie_value(set: &[String], name: &str) -> Option<String> {
    let whole = set_cookie_for(set, name)?;
    let pair = whole.split(';').next().unwrap();
    Some(pair.split_once('=').unwrap().1.to_string())
}

/// A signed-in client as a browser would hold it: the session cookie and the
/// CSRF token it was issued.
struct Signed {
    /// `name=value` of the session cookie (`mw_admin_session` or `mw_session`).
    session: String,
    /// The value of the `mw_csrf` cookie.
    token: String,
}

impl Signed {
    /// The `Cookie` header a browser sends: session and token cookie.
    fn cookies(&self) -> String {
        format!("{}; mw_csrf={}", self.session, self.token)
    }
}

async fn admin_login(c: &reqwest::Client, base: &str) -> Signed {
    let resp = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": ADMIN_USER, "password": ADMIN_PASS }))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let set = set_cookies(&resp);
    let body = resp.text().await.unwrap();
    assert_eq!(status, 200, "admin login with the right password: {body}");
    Signed {
        session: format!(
            "mw_admin_session={}",
            cookie_value(&set, "mw_admin_session").expect("login sets the admin cookie")
        ),
        token: cookie_value(&set, "mw_csrf").expect("login sets mw_csrf"),
    }
}

/// `_mock` is the server's only listed upstream, which it signs in to; the login
/// does not name it (t32-h9).
async fn mailbox_login(c: &reqwest::Client, base: &str, _mock: &str) -> Signed {
    let resp = c
        .post(format!("{base}/api/login"))
        .json(&json!({
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "mailbox login");
    let set = set_cookies(&resp);
    Signed {
        session: format!(
            "mw_session={}",
            cookie_value(&set, "mw_session").expect("login sets the session cookie")
        ),
        token: cookie_value(&set, "mw_csrf").expect("login sets mw_csrf"),
    }
}

// ── requests ───────────────────────────────────────────────────────────────────

/// One request, described so it can be sent to two servers and with or without
/// the token.
#[derive(Clone)]
struct Call {
    method: reqwest::Method,
    path: &'static str,
    body: Option<Value>,
}

fn call(method: &str, path: &'static str, body: Option<Value>) -> Call {
    Call {
        method: method.parse().unwrap(),
        path,
        body,
    }
}

/// Send `call` with the given `Cookie` header and, when `token` is `Some`, the
/// `X-CSRF-Token` header. Returns status and body text.
async fn send(
    c: &reqwest::Client,
    base: &str,
    call: &Call,
    cookies: Option<&str>,
    token: Option<&str>,
    origin: Option<&str>,
) -> (u16, String) {
    send_with(c, base, call, cookies, token, origin, None).await
}

/// [`send`], plus an `Authorization` header when `authorization` is `Some`.
async fn send_with(
    c: &reqwest::Client,
    base: &str,
    call: &Call,
    cookies: Option<&str>,
    token: Option<&str>,
    origin: Option<&str>,
    authorization: Option<&str>,
) -> (u16, String) {
    let mut req = c.request(call.method.clone(), format!("{base}{}", call.path));
    if let Some(authorization) = authorization {
        req = req.header(reqwest::header::AUTHORIZATION, authorization);
    }
    if let Some(cookies) = cookies {
        req = req.header(reqwest::header::COOKIE, cookies);
    }
    if let Some(token) = token {
        req = req.header("x-csrf-token", token);
    }
    if let Some(origin) = origin {
        req = req.header(reqwest::header::ORIGIN, origin);
    }
    if let Some(body) = &call.body {
        req = req.json(body);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16();
    (status, resp.text().await.unwrap())
}

fn is_token_refusal(answer: &(u16, String)) -> bool {
    answer.0 == 403 && answer.1.contains(CSRF_REFUSAL)
}

/// `call` as another program sends it: no cookie, no token.
async fn bare(c: &reqwest::Client, base: &str, call: &Call) -> (u16, String) {
    send(c, base, call, None, None, None).await
}

/// `call` with `who`'s cookies and no `X-CSRF-Token` header.
async fn without_token(
    c: &reqwest::Client,
    base: &str,
    call: &Call,
    who: &Signed,
) -> (u16, String) {
    send(c, base, call, Some(&who.cookies()), None, None).await
}

/// `call` with `who`'s cookies and the matching `X-CSRF-Token` header.
async fn with_token(c: &reqwest::Client, base: &str, call: &Call, who: &Signed) -> (u16, String) {
    send(c, base, call, Some(&who.cookies()), Some(&who.token), None).await
}

/// `call` from another site: a foreign `Origin`, no cookie.
async fn cross_origin(c: &reqwest::Client, base: &str, call: &Call) -> (u16, String) {
    send(c, base, call, None, None, Some("https://evil.example")).await
}

async fn ban_listed(c: &reqwest::Client, base: &str, admin: &Signed, ip: &str) -> bool {
    let resp = c
        .get(format!("{base}/admin/bans"))
        .header(reqwest::header::COOKIE, admin.cookies())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "GET /admin/bans");
    let bans: Value = resp.json().await.unwrap();
    bans.as_array()
        .expect("ban list is an array")
        .iter()
        .any(|b| b["ip"] == ip)
}

// ── sign-in routes ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn admin_login_is_not_asked_for_a_token_and_issues_one() {
    let base = spawn_server(ServerMode::Proxy, true, None).await;
    let c = client();

    // No cookie, no header: the state of a browser opening /admin for the first time.
    let resp = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": ADMIN_USER, "password": ADMIN_PASS }))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let set = set_cookies(&resp);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        status, 200,
        "the right password signs in under strict mode: {body}"
    );

    // The admin cookie is still the first `Set-Cookie`, HttpOnly.
    assert!(
        set.first()
            .is_some_and(|c| c.starts_with("mw_admin_session=")),
        "admin cookie first: {set:?}"
    );
    assert!(set[0].contains("HttpOnly"));

    // The token cookie is readable by the page and same-site only.
    let csrf = set_cookie_for(&set, "mw_csrf").expect("login sets mw_csrf");
    assert!(!csrf.contains("HttpOnly"), "the page must read it: {csrf}");
    assert!(csrf.contains("SameSite=Strict"), "{csrf}");
    assert!(csrf.contains("Path=/"), "{csrf}");
    let token = cookie_value(&set, "mw_csrf").unwrap();
    assert!(token.len() >= 24, "an unguessable token, got {token:?}");
    assert_eq!(body["csrfToken"], token.as_str(), "body repeats the cookie");
    assert_eq!(body["username"], ADMIN_USER);

    // Each login issues a new token.
    let again = admin_login(&c, &base).await;
    assert_ne!(
        again.token, token,
        "a second login must not reuse the token"
    );

    // A wrong password is answered by the handler (401), not by the guard (403).
    let bad = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": ADMIN_USER, "password": "definitely-wrong" }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 401, "wrong password");
    assert!(
        set_cookies(&bad).is_empty(),
        "a refused login sets no cookie"
    );
}

#[tokio::test]
async fn a_cross_origin_admin_login_is_refused_in_both_modes() {
    let login = call(
        "POST",
        "/admin/login",
        Some(json!({ "username": ADMIN_USER, "password": ADMIN_PASS })),
    );
    for strict in [true, false] {
        let base = spawn_server(ServerMode::Proxy, strict, None).await;
        let c = client();
        // Control: the same request without the foreign Origin signs in.
        let ok = bare(&c, &base, &login).await;
        assert_eq!(ok.0, 200, "strict={strict}: same-origin login: {}", ok.1);
        let own = send(&c, &base, &login, None, None, Some(&base)).await;
        assert_eq!(own.0, 200, "strict={strict}: own Origin: {}", own.1);

        let foreign = cross_origin(&c, &base, &login).await;
        assert_eq!(foreign.0, 403, "strict={strict}: {}", foreign.1);
        assert!(foreign.1.contains(ORIGIN_REFUSAL), "{}", foreign.1);
    }
}

// ── the admin panel ────────────────────────────────────────────────────────────

#[tokio::test]
async fn an_admin_write_needs_the_token() {
    let base = spawn_server(ServerMode::Proxy, true, None).await;
    let c = client();
    let admin = admin_login(&c, &base).await;
    let ip = "203.0.113.77";
    let add = call(
        "POST",
        "/admin/bans",
        Some(json!({ "ip": ip, "reason": "t30-e2" })),
    );
    let remove = call("DELETE", "/admin/bans/203.0.113.77", None);

    // Cookies but no header: refused, and nothing was written.
    let refused = without_token(&c, &base, &add, &admin).await;
    assert!(is_token_refusal(&refused), "no header: {refused:?}");
    // A header that does not match the cookie is refused too.
    let mismatch = send(
        &c,
        &base,
        &add,
        Some(&admin.cookies()),
        Some("not-the-token"),
        None,
    )
    .await;
    assert!(is_token_refusal(&mismatch), "wrong header: {mismatch:?}");
    // The header without the token cookie is refused: both halves are required.
    let no_cookie = send(
        &c,
        &base,
        &add,
        Some(&admin.session),
        Some(&admin.token),
        None,
    )
    .await;
    assert!(
        is_token_refusal(&no_cookie),
        "no mw_csrf cookie: {no_cookie:?}"
    );
    assert!(
        !ban_listed(&c, &base, &admin, ip).await,
        "a refused write wrote nothing"
    );

    // With the token: accepted, and the ban exists.
    let added = with_token(&c, &base, &add, &admin).await;
    assert_eq!(added.0, 204, "with the token: {}", added.1);
    assert!(
        ban_listed(&c, &base, &admin, ip).await,
        "the ban was stored"
    );

    // DELETE is guarded the same way.
    let refused = without_token(&c, &base, &remove, &admin).await;
    assert!(
        is_token_refusal(&refused),
        "DELETE without header: {refused:?}"
    );
    assert!(ban_listed(&c, &base, &admin, ip).await, "still banned");
    let removed = with_token(&c, &base, &remove, &admin).await;
    assert_eq!(removed.0, 204, "DELETE with the token: {}", removed.1);
    assert!(!ban_listed(&c, &base, &admin, ip).await, "ban removed");
}

#[tokio::test]
async fn admin_session_issues_a_token_only_when_the_client_has_none() {
    let base = spawn_server(ServerMode::Proxy, true, None).await;
    let c = client();
    let admin = admin_login(&c, &base).await;

    // Not signed in: 401 and no token cookie.
    let anon = c.get(format!("{base}/admin/session")).send().await.unwrap();
    assert_eq!(anon.status(), 401);
    assert!(set_cookies(&anon).is_empty(), "no cookie for a stranger");

    // Signed in and holding a token: it is repeated in the body and not replaced.
    let resp = c
        .get(format!("{base}/admin/session"))
        .header(reqwest::header::COOKIE, admin.cookies())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(
        set_cookie_for(&set_cookies(&resp), "mw_csrf").is_none(),
        "an existing token is kept"
    );
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["username"], ADMIN_USER);
    assert_eq!(body["csrfToken"], admin.token.as_str());

    // Signed in without a token cookie (a session from before the cookie existed,
    // or a browser that dropped it): one is issued, and it works.
    let resp = c
        .get(format!("{base}/admin/session"))
        .header(reqwest::header::COOKIE, &admin.session)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let set = set_cookies(&resp);
    let issued = cookie_value(&set, "mw_csrf").expect("a token is issued");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["csrfToken"], issued.as_str());
    let recovered = Signed {
        session: admin.session.clone(),
        token: issued,
    };
    let add = call(
        "POST",
        "/admin/bans",
        Some(json!({ "ip": "203.0.113.78", "reason": "t30-e2" })),
    );
    let added = with_token(&c, &base, &add, &recovered).await;
    assert_eq!(added.0, 204, "the issued token is accepted: {}", added.1);
}

// ── the mailbox ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_mailbox_write_needs_the_token() {
    let mock = spawn_mock().await;
    let base = spawn_server(ServerMode::Proxy, true, Some(&mock)).await;
    let c = client();
    let user = mailbox_login(&c, &base, &mock).await;
    let jmap = call(
        "POST",
        "/jmap/api",
        Some(json!({ "using": ["urn:ietf:params:jmap:core"], "methodCalls": [] })),
    );

    let refused = without_token(&c, &base, &jmap, &user).await;
    assert!(is_token_refusal(&refused), "no header: {refused:?}");
    let ok = with_token(&c, &base, &jmap, &user).await;
    assert_eq!(ok.0, 200, "with the token: {}", ok.1);

    // `GET /api/me` issues a token to a signed-in client that has none.
    let resp = c
        .get(format!("{base}/api/me"))
        .header(reqwest::header::COOKIE, &user.session)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let issued = cookie_value(&set_cookies(&resp), "mw_csrf").expect("a token is issued");
    let recovered = Signed {
        session: user.session.clone(),
        token: issued,
    };
    let ok = with_token(&c, &base, &jmap, &recovered).await;
    assert_eq!(ok.0, 200, "the issued token is accepted: {}", ok.1);
}

/// Routes from each merged router that a cookie authorises. None may be on the
/// exempt list: without the token each is refused by the guard, and with it the
/// guard lets it through to the handler (whatever the handler then answers).
///
/// The push-subscription routes are here on purpose. They accept a native bearer
/// token *or* the browser's session cookie (`crate::authed`), so they are not
/// machine-only routes.
#[tokio::test]
async fn routes_a_cookie_authorises_are_not_exempt() {
    let mock = spawn_mock().await;
    let base = spawn_server(ServerMode::Proxy, true, Some(&mock)).await;
    let c = client();
    let user = mailbox_login(&c, &base, &mock).await;
    let admin = admin_login(&c, &base).await;

    let mailbox = [
        call("POST", "/api/sanitize", Some(json!({ "html": "<p>x</p>" }))),
        call("POST", "/api/push/subscribe", Some(json!({}))),
        call("POST", "/api/push/unsubscribe", Some(json!({}))),
        call("POST", "/oauth/consent", Some(json!({}))),
        call("POST", "/oauth/decision", Some(json!({}))),
        call("POST", "/api/keys", Some(json!({}))),
        call("POST", "/api/zeroaccess/disable", None),
        call("POST", "/api/account/sessions/revoke", Some(json!({}))),
        call("PUT", "/api/account/notifications", Some(json!({}))),
        call("DELETE", "/api/account/signatures/none", None),
        call("POST", "/api/masked", Some(json!({}))),
        call("POST", "/api/remote-images/grant", Some(json!({}))),
        call("POST", "/api/calendar/refresh", Some(json!({}))),
        call("POST", "/api/account/sieve/sync", Some(json!({}))),
        call("POST", "/jmap/upload/acct-1", Some(json!({}))),
        // Last: with the token these end the session.
        call("POST", "/api/sso/logout", None),
        call("POST", "/api/logout", None),
    ];
    let panel = [
        call("PUT", "/admin/oauth-dcr", Some(json!({ "enabled": false }))),
        call("POST", "/admin/2fa/policy", Some(json!({}))),
        call("POST", "/admin/sso", Some(json!({}))),
        call("DELETE", "/admin/sso/none", None),
        call("POST", "/admin/plugins", Some(json!({}))),
        call("POST", "/admin/ui-plugins", Some(json!({}))),
        call("POST", "/admin/egress/proxies/deactivate", None),
        call("PUT", "/admin/assist", Some(json!({}))),
        call("POST", "/admin/maintenance/rethread", Some(json!({}))),
        call(
            "PUT",
            "/admin/api-keys/none/unattended-send",
            Some(json!({})),
        ),
        call("PUT", "/admin/security-policy", Some(json!({}))),
        // Last: with the token this ends the session.
        call("POST", "/admin/logout", None),
    ];

    for (who, calls) in [(&user, &mailbox[..]), (&admin, &panel[..])] {
        for call in calls {
            let refused = without_token(&c, &base, call, who).await;
            assert!(
                is_token_refusal(&refused),
                "{} {} without the token must be refused by the guard: {refused:?}",
                call.method,
                call.path
            );
            let passed = with_token(&c, &base, call, who).await;
            assert!(
                !is_token_refusal(&passed),
                "{} {} with the token must reach its handler: {passed:?}",
                call.method,
                call.path
            );
        }
    }
}

// ── a bearer header beside a session cookie ────────────────────────────────────

const FOREIGN: &str = "https://evil.example";

/// A bearer header does not switch the guard off for a request a cookie can
/// authorise. The admin gates read only the `mw_admin_session` cookie, so before
/// 26.20 `Authorization: Bearer anything` beside that cookie got a write past both
/// the Origin check and the token check.
#[tokio::test]
async fn a_bearer_header_does_not_excuse_a_request_with_a_session_cookie() {
    for strict in [true, false] {
        let mock = spawn_mock().await;
        let base = spawn_server(ServerMode::Proxy, strict, Some(&mock)).await;
        let c = client();
        let admin = admin_login(&c, &base).await;
        let user = mailbox_login(&c, &base, &mock).await;
        let bearer = Some("Bearer anything");
        let add = call(
            "POST",
            "/admin/bans",
            Some(json!({ "ip": "203.0.113.80", "reason": "t30-e2" })),
        );
        let jmap = call(
            "POST",
            "/jmap/api",
            Some(json!({ "using": ["urn:ietf:params:jmap:core"], "methodCalls": [] })),
        );

        // Origin check, in both modes: admin cookie, mailbox cookie.
        for (who, call) in [(&admin, &add), (&user, &jmap)] {
            let cookies = who.cookies();
            let foreign = send_with(
                &c,
                &base,
                call,
                Some(&cookies),
                Some(&who.token),
                Some(FOREIGN),
                bearer,
            )
            .await;
            assert_eq!(
                foreign.0, 403,
                "strict={strict}: {} from another origin, cookie + bearer: {foreign:?}",
                call.path
            );
            assert!(foreign.1.contains(ORIGIN_REFUSAL), "{foreign:?}");
        }
        assert!(
            !ban_listed(&c, &base, &admin, "203.0.113.80").await,
            "strict={strict}: the cross-origin write stored nothing"
        );

        // Token check, strict mode only.
        let cookies = admin.cookies();
        let no_token = send_with(&c, &base, &add, Some(&cookies), None, None, bearer).await;
        if strict {
            assert!(
                is_token_refusal(&no_token),
                "admin cookie + bearer, no token: {no_token:?}"
            );
            assert!(!ban_listed(&c, &base, &admin, "203.0.113.80").await);
            let cookies = user.cookies();
            let no_token = send_with(&c, &base, &jmap, Some(&cookies), None, None, bearer).await;
            assert!(
                is_token_refusal(&no_token),
                "mailbox cookie + bearer, no token: {no_token:?}"
            );
        } else {
            assert_eq!(no_token.0, 204, "not strict: no token needed: {no_token:?}");
        }

        // Control: the same admin write, same origin, with its token, is accepted
        // with the bearer header still on it — the header itself is not refused.
        let cookies = admin.cookies();
        let ok = send_with(
            &c,
            &base,
            &add,
            Some(&cookies),
            Some(&admin.token),
            None,
            bearer,
        )
        .await;
        assert_eq!(ok.0, 204, "strict={strict}: {ok:?}");
        assert!(ban_listed(&c, &base, &admin, "203.0.113.80").await);
    }
}

/// The other direction: a request whose only authority is its bearer token is
/// still excused from both checks — the native shells' path. They send
/// `Authorization: Bearer <session token>` with no cookie, from another origin,
/// and hold no CSRF token.
#[tokio::test]
async fn a_bearer_request_without_a_session_cookie_is_not_checked() {
    let mock = spawn_mock().await;
    let base = spawn_server(ServerMode::Proxy, true, Some(&mock)).await;
    let c = client();

    // A native login: the token comes back in the body and no cookie is set.
    let resp = c
        .post(format!("{base}/api/login"))
        .json(&json!({
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
            "clientType": "native",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "native login");
    assert!(
        set_cookies(&resp).is_empty(),
        "a native login sets no cookie"
    );
    let body: Value = resp.json().await.unwrap();
    let bearer = format!("Bearer {}", body["token"].as_str().expect("token in body"));
    let jmap = call(
        "POST",
        "/jmap/api",
        Some(json!({ "using": ["urn:ietf:params:jmap:core"], "methodCalls": [] })),
    );

    // No cookie at all, another origin, no CSRF token: accepted.
    let ok = send_with(&c, &base, &jmap, None, None, Some(FOREIGN), Some(&bearer)).await;
    assert_eq!(ok.0, 200, "bearer only: {ok:?}");

    // A cookie that authorises nothing (a stray `mw_csrf`) changes nothing.
    let ok = send_with(
        &c,
        &base,
        &jmap,
        Some("mw_csrf=stray; theme=dark"),
        None,
        Some(FOREIGN),
        Some(&bearer),
    )
    .await;
    assert_eq!(ok.0, 200, "bearer + a non-session cookie: {ok:?}");

    // Control for the two above: the same request with a session cookie beside the
    // bearer is checked, and refused for its origin.
    let user = mailbox_login(&c, &base, &mock).await;
    let checked = send_with(
        &c,
        &base,
        &jmap,
        Some(&user.session),
        None,
        Some(FOREIGN),
        Some(&bearer),
    )
    .await;
    assert_eq!(checked.0, 403, "{checked:?}");
    assert!(checked.1.contains(ORIGIN_REFUSAL), "{checked:?}");
}

// ── routes no cookie authorises ────────────────────────────────────────────────

/// The exempt machine routes, each with a request that carries no cookie, no
/// bearer token and no CSRF token — what a program calling them sends. A strict
/// server must answer each exactly as a server with strict mode off does: the
/// answer comes from the handler in both.
///
/// Engine mode, because `/mcp` is mounted only there.
#[tokio::test]
async fn routes_no_cookie_authorises_answer_as_with_strict_off() {
    let strict = spawn_server(ServerMode::Engine, true, None).await;
    let plain = spawn_server(ServerMode::Engine, false, None).await;
    let c = client();

    let machine = [
        call(
            "POST",
            "/oauth/token",
            Some(json!({
                "grant_type": "authorization_code",
                "code": "no-such-code",
                "redirect_uri": "https://app.example/cb",
                "client_id": "no-such-client",
                "code_verifier": "v",
            })),
        ),
        call(
            "POST",
            "/oauth/introspect",
            Some(json!({ "token": "no-such-token" })),
        ),
        call(
            "POST",
            "/oauth/revoke",
            Some(json!({ "token": "no-such-token" })),
        ),
        call(
            "POST",
            "/oauth/register",
            Some(json!({ "redirect_uris": ["https://app.example/cb"] })),
        ),
        call(
            "PUT",
            "/oauth/register/no-such-client",
            Some(json!({ "redirect_uris": ["https://app.example/cb"] })),
        ),
        call("DELETE", "/oauth/register/no-such-client", None),
        call("POST", "/api/webhooks/inbound", Some(json!({}))),
        call("POST", "/errors", Some(json!({ "message": "boom" }))),
        call(
            "POST",
            "/mcp",
            Some(json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" })),
        ),
        // Sign-in routes that were exempt before 26.20, pinned here as well.
        call(
            "POST",
            "/api/discover",
            Some(json!({ "email": "not-an-address" })),
        ),
        call(
            "POST",
            "/api/login/2fa",
            Some(json!({ "pendingToken": "none", "method": "totp", "code": "000000" })),
        ),
    ];

    for call in &machine {
        let off = bare(&c, &plain, call).await;
        let on = bare(&c, &strict, call).await;
        assert!(
            !is_token_refusal(&on),
            "{} {} must not be asked for a CSRF token: {on:?}",
            call.method,
            call.path
        );
        assert_eq!(
            on, off,
            "{} {}: strict mode must not change the answer",
            call.method, call.path
        );

        // The Origin check still covers it, in strict mode and out of it.
        for base in [&strict, &plain] {
            let foreign = cross_origin(&c, base, call).await;
            assert_eq!(foreign.0, 403, "{} {}: {foreign:?}", call.method, call.path);
            assert!(
                foreign.1.contains(ORIGIN_REFUSAL),
                "{} {}: {foreign:?}",
                call.method,
                call.path
            );
        }
    }

    // The MCP answer above is the MCP server's, not the SPA fallback's.
    let tools = bare(&c, &strict, &machine[8]).await;
    assert_eq!(tools.0, 200, "{tools:?}");
    assert!(tools.1.contains("\"jsonrpc\""), "{tools:?}");

    // Client registration, switched on: a registration with no cookie and no
    // token succeeds on the strict server as it does on the other.
    for (base, is_strict) in [(&strict, true), (&plain, false)] {
        let admin = admin_login(&c, base).await;
        let enable = call(
            "PUT",
            "/admin/oauth-dcr",
            Some(json!({
                "enabled": true,
                "allowedRedirectHostSuffixes": ["example.org"],
                "defaultScope": { "read": true },
            })),
        );
        let enabled = with_token(&c, base, &enable, &admin).await;
        assert_eq!(
            enabled.0, 200,
            "strict={is_strict}: enable DCR: {}",
            enabled.1
        );
        let register = call(
            "POST",
            "/oauth/register",
            Some(json!({
                "redirect_uris": ["https://client.example.org/cb"],
                "client_name": "t30-e2",
            })),
        );
        let created = bare(&c, base, &register).await;
        assert_eq!(
            created.0, 201,
            "strict={is_strict}: register: {}",
            created.1
        );
        let body: Value = serde_json::from_str(&created.1).unwrap();
        assert!(body["client_id"].is_string(), "strict={is_strict}: {body}");
    }
}
