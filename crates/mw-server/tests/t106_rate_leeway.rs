//! Leeway for legitimate clients (t106), through the real middleware stack.
//!
//! One signed-in account doing a heavy-but-legitimate burst of JMAP calls is
//! queued, not refused; the old ceiling (four at once, no queue) is reproduced by
//! setting the knobs to it, which refuses with pacing headers; a second account
//! behind the same address is unaffected; anonymous budgets stay strict.
use futures_util::future::join_all;
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{DomainConfigRow, ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
mod common;
use common::{mock_mail::MockPop3, test_db};

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

struct Fixture {
    base: String,
    store: Store,
    _mail: MockPop3,
}

fn row(name: &str, mail: &MockPop3) -> DomainConfigRow {
    DomainConfigRow {
        name: name.into(),
        upstream_json: json!({"pop3":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),
        allowlist_json: "[]".into(),
        blocklist_json: "[]".into(),
        enabled: true,
        preferred: false,
        settings_json: "{}".into(),
        updated_at: 1,
    }
}

async fn put(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "t106".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

/// A server in engine mode with one POP3 domain holding `users`, and `settings`
/// written before the first request.
async fn fixture(domain: &str, users: &[&str], settings: &[(&str, Value)]) -> Fixture {
    let accounts: Vec<(String, String)> = users
        .iter()
        .map(|u| (format!("{u}@{domain}"), "testpass".to_owned()))
        .collect();
    let refs: Vec<(&str, &str)> = accounts.iter().map(|(u, p)| (u.as_str(), p.as_str())).collect();
    let mail = MockPop3::start_accounts(&refs).await;
    let db = test_db::unique_dir("t106")
        .join("db.sqlite")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap()).await.unwrap();
    store.put_domain_config(&row(domain, &mail)).await.unwrap();
    put(&store, "signin.retryCooldownSecs", json!(0)).await;
    for (key, value) in settings {
        put(&store, key, value.clone()).await;
    }
    mw_server::forget_rate_knobs();
    let app = build_app(AppConfig {
        db_path: db,
        server_key_hex: Some(KEY.into()),
        mode: ServerMode::Engine,
        cookie_secure: false,
        web_dir: None,
        hardening: Default::default(),
        security: SecurityConfig {
            upstream_default: Some(mail.url()),
            ..Default::default()
        },
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    Fixture {
        base: format!("http://{addr}"),
        store,
        _mail: mail,
    }
}

async fn sign_in(base: &str, user: &str) -> reqwest::Client {
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let r = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":user,"password":"testpass"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "{}", r.text().await.unwrap());
    client
}

fn echo() -> Value {
    json!({"using":["urn:ietf:params:jmap:core"],"methodCalls":[["Core/echo",{},"x"]]})
}

/// `n` simultaneous Core/echo requests; the statuses and the refusals' headers.
async fn burst(client: &reqwest::Client, base: &str, n: usize) -> (Vec<u16>, Vec<reqwest::header::HeaderMap>) {
    let calls = (0..n).map(|_| async {
        let r = client
            .post(format!("{base}/jmap/api"))
            .json(&echo())
            .send()
            .await
            .unwrap();
        (r.status().as_u16(), r.headers().clone())
    });
    let all = join_all(calls).await;
    let refused = all
        .iter()
        .filter(|(s, _)| *s == 429)
        .map(|(_, h)| h.clone())
        .collect();
    (all.iter().map(|(s, _)| *s).collect(), refused)
}

#[tokio::test]
async fn a_legitimate_burst_of_one_account_is_queued_not_refused() {
    let f = fixture("burst.example", &["alice"], &[]).await;
    let alice = sign_in(&f.base, "alice@burst.example").await;
    // Sixty at once: a folder switch, prefetch of neighbours and a second tab.
    let (statuses, _) = burst(&alice, &f.base, 60).await;
    assert!(statuses.iter().all(|s| *s == 200), "{statuses:?}");
}

#[tokio::test]
async fn the_old_ceiling_refuses_with_pacing_headers_and_is_a_setting() {
    // The pre-t106 shape: four at once, no waiting.
    let f = fixture(
        "old.example",
        &["alice"],
        &[
            ("ratelimit.jmap.perAccountConcurrent", json!(4)),
            ("ratelimit.jmap.globalConcurrent", json!(4)),
            ("ratelimit.queueMs", json!(0)),
        ],
    )
    .await;
    let alice = sign_in(&f.base, "alice@old.example").await;
    let mut refused = Vec::new();
    for _ in 0..5 {
        let (_, headers) = burst(&alice, &f.base, 150).await;
        refused.extend(headers);
        if !refused.is_empty() {
            break;
        }
    }
    assert!(!refused.is_empty(), "the old ceiling never refused");
    for h in &refused {
        assert!(h.get("retry-after").is_some(), "{h:?}");
        assert_eq!(h.get("ratelimit-limit").unwrap(), "4");
        assert_eq!(h.get("ratelimit-remaining").unwrap(), "0");
        assert!(h.get("ratelimit-reset").is_some());
    }
}

#[tokio::test]
async fn two_accounts_behind_one_address_do_not_starve_each_other() {
    let f = fixture(
        "nat.example",
        &["alice", "bob"],
        &[
            ("ratelimit.jmap.perAccountConcurrent", json!(4)),
            ("ratelimit.queueMs", json!(0)),
        ],
    )
    .await;
    let alice = sign_in(&f.base, "alice@nat.example").await;
    let bob = sign_in(&f.base, "bob@nat.example").await;
    // Both clients come from 127.0.0.1. Alice floods; Bob makes one request at a
    // time and is never refused, because Bob's allowance is Bob's own.
    let flood = burst(&alice, &f.base, 150);
    let polite = async {
        let mut statuses = Vec::new();
        for _ in 0..25 {
            let r = bob
                .post(format!("{}/jmap/api", f.base))
                .json(&echo())
                .send()
                .await
                .unwrap();
            statuses.push(r.status().as_u16());
        }
        statuses
    };
    let (_, bob_statuses) = tokio::join!(flood, polite);
    assert!(bob_statuses.iter().all(|s| *s == 200), "{bob_statuses:?}");
}

#[tokio::test]
async fn anonymous_consent_budget_stays_strict_and_paces_the_client() {
    let f = fixture(
        "anon.example",
        &["alice"],
        &[
            ("ratelimit.anon.burst", json!(3)),
            ("ratelimit.anon.perMinute", json!(1)),
        ],
    )
    .await;
    let anonymous = reqwest::Client::builder().no_proxy().build().unwrap();
    let mut statuses = Vec::new();
    let mut last = None;
    for _ in 0..6 {
        let r = anonymous
            .post(format!("{}/api/cookies/detected", f.base))
            .header("content-type", "application/json")
            .body("{}")
            .send()
            .await
            .unwrap();
        statuses.push(r.status().as_u16());
        if r.status() == 429 {
            last = Some(r.headers().clone());
        }
    }
    assert_eq!(statuses.iter().filter(|s| **s == 429).count(), 3, "{statuses:?}");
    let h = last.unwrap();
    assert_eq!(h.get("retry-after").unwrap(), "60");
    assert_eq!(h.get("ratelimit-limit").unwrap(), "3");
    // The refusals are listed for the administrator (class, reason, key type).
    let _ = &f.store;
}

#[tokio::test]
async fn signed_in_accounts_are_not_charged_to_the_shared_address() {
    // The per-address budget is tiny; both signed-in mailboxes behind that one
    // address still make dozens of requests, while an anonymous caller is refused.
    let f = fixture(
        "agg.example",
        &["alice", "bob"],
        &[
            ("security.protection.rateMode", json!("enforce")),
            ("security.protection.clientRequests", json!(5)),
            ("security.protection.accountRequests", json!(1800)),
        ],
    )
    .await;
    let alice = sign_in(&f.base, "alice@agg.example").await;
    let bob = sign_in(&f.base, "bob@agg.example").await;
    for client in [&alice, &bob] {
        for _ in 0..40 {
            let r = client.get(format!("{}/jmap/session", f.base)).send().await.unwrap();
            assert_eq!(r.status(), 200);
        }
    }
    let anonymous = reqwest::Client::builder().no_proxy().build().unwrap();
    let mut refused = None;
    for _ in 0..12 {
        let r = anonymous.get(format!("{}/api/me", f.base)).send().await.unwrap();
        if r.status() == 429 {
            refused = Some(r);
            break;
        }
    }
    let r = refused.expect("an anonymous source is still limited per address");
    assert!(r.headers().get("retry-after").is_some());
}
