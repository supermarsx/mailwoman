//! t98: feature profiles cover every gateable feature and the server refuses
//! each one when it is off — one representative route per gated family, through
//! the production router — plus presets, import/export and the effective preview.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
mod common;

async fn serve(app: axum::Router) -> String {
    let socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = socket.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            socket,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}
async fn send(req: reqwest::RequestBuilder) -> (u16, Value) {
    let r = req.send().await.unwrap();
    let status = r.status().as_u16();
    let text = r.text().await.unwrap();
    (
        status,
        serde_json::from_str(&text).unwrap_or(Value::String(text)),
    )
}
async fn revision(admin: &reqwest::Client, base: &str) -> String {
    let r = admin
        .get(format!("{base}/admin/features"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    r.headers()[reqwest::header::ETAG]
        .to_str()
        .unwrap()
        .to_string()
}

struct Fixture {
    base: String,
    user: reqwest::Client,
    admin: reqwest::Client,
    account: String,
    authority: String,
    upstream_calls: Arc<AtomicUsize>,
}

async fn fixture() -> Fixture {
    let upstream_calls = Arc::new(AtomicUsize::new(0));
    let observed = upstream_calls.clone();
    let upstream = serve(mw_mock_jmap::router().layer(axum::middleware::from_fn(
        move |request: axum::extract::Request, next: axum::middleware::Next| {
            let calls = observed.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                next.run(request).await
            }
        },
    )))
    .await;
    let root = common::test_db::unique_dir("mw-t98-features");
    let config = AppConfig {
        db_path: root.join("db.sqlite").to_string_lossy().into_owned(),
        server_key_hex: Some(
            "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b".into(),
        ),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            upstream_default: Some(upstream.clone()),
            jmap_upstreams: Some(vec![upstream.clone()]),
            ..Default::default()
        },
    };
    let store = mw_store::Store::open(
        &config.db_path,
        mw_store::ServerKey::from_hex(config.server_key_hex.as_ref().unwrap()).unwrap(),
    )
    .await
    .unwrap();
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "t98-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    store
        .put_domain_config(&mw_store::DomainConfigRow {
            name: "example.org".into(),
            upstream_json: json!({"jmap":{"sessionUrl":upstream}}).to_string(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    let base = serve(
        build_app_full(
            config,
            V6Config {
                admin_enabled: true,
                admin_username: Some("root".into()),
                admin_password: Some("hunter2".into()),
                redis_url: None,
            },
        )
        .await
        .unwrap()
        .0,
    )
    .await;
    let user = client();
    let admin = client();
    let (status, me) = send(
        user.post(format!("{base}/api/login"))
            .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS})),
    )
    .await;
    assert_eq!(status, 200, "{me}");
    let account = me["accountId"].as_str().unwrap().to_string();
    let authority = me["storageAuthority"].as_str().unwrap().to_string();
    assert_eq!(
        send(
            admin
                .post(format!("{base}/admin/login"))
                .json(&json!({"username":"root","password":"hunter2"}))
        )
        .await
        .0,
        200
    );
    Fixture {
        base,
        user,
        admin,
        account,
        authority,
        upstream_calls,
    }
}

impl Fixture {
    async fn set_global(&self, key: &str, value: Value) {
        let (status, body) = send(
            self.admin
                .put(format!("{}/admin/settings/{key}", self.base))
                .json(&json!({ "globalValue": value })),
        )
        .await;
        assert_eq!(status, 200, "{key}: {body}");
    }
    async fn jmap(&self, method: &str, args: Value) -> (u16, Value) {
        let mut args = args;
        args["accountId"] = json!(self.account);
        send(
            self.user
                .post(format!("{}/jmap/api", self.base))
                .json(&json!({
                    "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
                    "methodCalls": [[method, args, "a"]]
                })),
        )
        .await
    }
}

fn refused(status: u16, body: &Value, feature: &str) {
    assert_eq!(status, 403, "{feature}: {body}");
    assert_eq!(body["code"], "feature_disabled", "{body}");
    assert_eq!(body["feature"], feature, "{body}");
}
fn not_refused(status: u16, body: &Value, what: &str) {
    assert!(
        !(status == 403 && body["code"] == "feature_disabled"),
        "{what} must be allowed again: {status} {body}"
    );
}

/// One representative route per gated family: off → `403 feature_disabled`
/// with the feature id, before anything reaches the mail server; on → allowed.
#[tokio::test]
async fn every_gated_family_is_refused_when_off_and_allowed_when_on() {
    let f = fixture().await;
    let b = &f.base;

    // REST prefix (masked addresses).
    f.set_global("cap.masked", json!(false)).await;
    let (s, body) = send(f.user.get(format!("{b}/api/masked"))).await;
    refused(s, &body, "masked");
    f.set_global("cap.masked", json!(true)).await;
    let (s, body) = send(f.user.get(format!("{b}/api/masked"))).await;
    not_refused(s, &body, "masked");

    // A REST write (API keys); a read stays possible.
    f.set_global("cap.api_keys", json!(false)).await;
    let (s, body) = send(
        f.user
            .post(format!("{b}/api/keys"))
            .json(&json!({"label":"x","scope":{}})),
    )
    .await;
    refused(s, &body, "api_keys");
    let (s, body) = send(f.user.get(format!("{b}/api/keys"))).await;
    not_refused(s, &body, "listing keys");
    f.set_global("cap.api_keys", json!(true)).await;
    let (s, body) = send(
        f.user
            .post(format!("{b}/api/keys"))
            .json(&json!({"label":"x","scope":{}})),
    )
    .await;
    not_refused(s, &body, "api_keys");

    // A user setting a feature owns (reader zoom); resetting stays possible.
    f.set_global("cap.zoom", json!(false)).await;
    let put_zoom = || {
        f.user
            .put(format!("{b}/api/settings/reader.zoom"))
            .header("x-mailwoman-storage-authority", &f.authority)
            .json(&json!({"value":120}))
    };
    let (s, body) = send(put_zoom()).await;
    refused(s, &body, "reading.zoom");
    let (s, body) = send(
        f.user
            .delete(format!("{b}/api/settings/reader.zoom"))
            .header("x-mailwoman-storage-authority", &f.authority),
    )
    .await;
    not_refused(s, &body, "resetting zoom");
    f.set_global("cap.zoom", json!(true)).await;
    let (s, body) = send(put_zoom()).await;
    assert_eq!(s, 200, "{body}");

    // JMAP method rules: search operators, labels, snooze, scheduled send, rich text.
    let cases: &[(&str, &str, &str, Value)] = &[
        (
            "cap.search.operators",
            "search.operators",
            "Email/query",
            json!({"filter":{"text":"from:ann budget"}}),
        ),
        (
            "cap.labels",
            "labels",
            "Email/set",
            json!({"update":{"m1":{"keywords/work":true}}}),
        ),
        (
            "cap.snooze",
            "snooze",
            "Email/set",
            json!({"update":{"m1":{"snoozedUntil":"2030-01-01T00:00:00Z"}}}),
        ),
        (
            "cap.compose.scheduled_send",
            "compose.scheduled_send",
            "EmailSubmission/set",
            json!({"create":{"s":{"emailId":"m1","sendAt":"2030-01-01T00:00:00Z"}}}),
        ),
        (
            "cap.compose.rich_text",
            "compose.rich_text",
            "Email/set",
            json!({"create":{"d":{"htmlBody":[{"partId":"b","type":"text/html"}],"bodyValues":{"b":{"value":"<p>x</p>"}}}}}),
        ),
        ("cap.keys", "keys", "CryptoKey/get", json!({})),
        (
            "cap.vacation",
            "vacation",
            "VacationResponse/get",
            json!({}),
        ),
        (
            "cap.views.events",
            "views.events",
            "Email/query",
            json!({"filter":{"inMailbox":"mwViewEvents"}}),
        ),
    ];
    for (key, feature, method, args) in cases {
        f.set_global(key, json!(false)).await;
        let before = f.upstream_calls.load(Ordering::SeqCst);
        let (s, body) = f.jmap(method, args.clone()).await;
        refused(s, &body, feature);
        assert_eq!(
            f.upstream_calls.load(Ordering::SeqCst),
            before,
            "{feature}: a refused batch never reaches the mail server"
        );
        f.set_global(key, json!(true)).await;
        let (s, body) = f.jmap(method, args.clone()).await;
        not_refused(s, &body, feature);
    }
    // Undoing stays possible with the feature off.
    f.set_global("cap.snooze", json!(false)).await;
    let (s, body) = f
        .jmap("Email/set", json!({"update":{"m1":{"snoozedUntil":null}}}))
        .await;
    not_refused(s, &body, "ending a snooze");
    // Plain searches stay possible with operators off.
    f.set_global("cap.search.operators", json!(false)).await;
    let (s, body) = f
        .jmap("Email/query", json!({"filter":{"text":"budget at 10:30"}}))
        .await;
    not_refused(s, &body, "a plain search");

    // A limit: recipients per message.
    f.set_global("limit.recipients_per_message", json!(2)).await;
    let (s, body) = f
        .jmap(
            "Email/set",
            json!({"create":{"d":{"to":[{"email":"a@x.test"},{"email":"b@x.test"},{"email":"c@x.test"}]}}}),
        )
        .await;
    assert_eq!(s, 403, "{body}");
    assert_eq!(body["code"], "limit_exceeded");
    assert_eq!(body["limit"], "limit.recipients_per_message");
    let (s, body) = f
        .jmap(
            "Email/set",
            json!({"create":{"d":{"to":[{"email":"a@x.test"},{"email":"b@x.test"}]}}}),
        )
        .await;
    assert_ne!(body["code"], "limit_exceeded", "{s} {body}");
}

/// The live acceptance path at the API level: the "Mail only" preset assigned
/// to the user's domain hides calendar, tasks, notes and contacts (and their API
/// refuses them); switching the profile to "Full" brings them back at once.
#[tokio::test]
async fn mail_only_preset_refuses_pim_and_full_restores_it() {
    let f = fixture().await;
    let b = &f.base;
    let (_, cat) = send(f.admin.get(format!("{b}/admin/features"))).await;
    let preset = |id: &str| {
        cat["presets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .unwrap()["values"]
            .clone()
    };
    assert!(cat["profileKeys"].as_array().unwrap().len() > 60);
    assert_eq!(cat["defaults"]["cap.pim.calendar"], true);
    assert!(
        cat["categories"]
            .as_array()
            .unwrap()
            .contains(&json!("pim"))
    );
    let put = |values: Value| {
        let admin = f.admin.clone();
        let base = b.clone();
        async move {
            let rev = revision(&admin, &base).await;
            send(
                admin
                    .put(format!("{base}/admin/features/profiles/mail-only"))
                    .header("if-match", rev)
                    .json(&json!({"name":"Mail only","values":values})),
            )
            .await
        }
    };
    let (s, body) = put(preset("mail-only")).await;
    assert_eq!(s, 200, "{body}");
    let rev = revision(&f.admin, b).await;
    let (s, body) = send(
        f.admin
            .put(format!("{b}/admin/features/assignments"))
            .header("if-match", rev)
            .json(&json!({"scopeKind":"domain","scopeId":"example.org","profileId":"mail-only","priority":1})),
    )
    .await;
    assert_eq!(s, 200, "{body}");

    let (_, features) = send(f.user.get(format!("{b}/api/features"))).await;
    for id in ["calendar", "tasks", "notes", "contacts", "keys"] {
        assert_eq!(
            features["features"][id]["enabled"], false,
            "{id}: {features}"
        );
    }
    assert_eq!(features["features"]["compose.rich_text"]["enabled"], true);
    let pim = [
        ("CalendarEvent/get", "calendar"),
        ("Task/get", "tasks"),
        ("Note/get", "notes"),
        ("ContactCard/get", "contacts"),
    ];
    for (method, feature) in pim {
        let (s, body) = f.jmap(method, json!({})).await;
        refused(s, &body, feature);
    }
    // The preview says the same, and names the profile.
    let (s, preview) = send(
        f.admin
            .get(format!("{b}/admin/features/effective?domain=example.org")),
    )
    .await;
    assert_eq!(s, 200, "{preview}");
    let cal = preview["features"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == "calendar")
        .unwrap();
    assert_eq!(cal["enabled"], false);
    assert_eq!(cal["keys"]["cap.pim.calendar"]["source"], "profile");
    assert_eq!(cal["keys"]["cap.pim.calendar"]["profile"], "mail-only");

    // Export → import round trip, and a bad document is refused.
    let (s, doc) = send(
        f.admin
            .get(format!("{b}/admin/features/profiles/mail-only/export")),
    )
    .await;
    assert_eq!(s, 200, "{doc}");
    assert_eq!(doc["format"], "mailwoman.featureProfile");
    let (s, back) = send(
        f.admin
            .post(format!("{b}/admin/features/profiles/import"))
            .json(&doc),
    )
    .await;
    assert_eq!(s, 200, "{back}");
    assert_eq!(back["values"], doc["values"]);
    let mut bad = doc.clone();
    bad["values"]["cap.pim.calendar"] = json!({"value":"no"});
    let (s, err) = send(
        f.admin
            .post(format!("{b}/admin/features/profiles/import"))
            .json(&bad),
    )
    .await;
    assert_eq!(s, 422, "{err}");
    let (s, _) = send(
        f.user
            .post(format!("{b}/admin/features/profiles/import"))
            .json(&doc),
    )
    .await;
    assert_eq!(s, 401, "users cannot reach the admin import");

    // Full: everything back, without a new sign-in.
    let (s, body) = put(preset("full")).await;
    assert_eq!(s, 200, "{body}");
    let (_, features) = send(f.user.get(format!("{b}/api/features"))).await;
    for id in ["calendar", "tasks", "notes", "contacts", "keys"] {
        assert_eq!(features["features"][id]["enabled"], true, "{id}");
    }
    for (method, feature) in pim {
        let (s, body) = f.jmap(method, json!({})).await;
        not_refused(s, &body, feature);
    }
}
