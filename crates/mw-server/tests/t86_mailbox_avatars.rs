//! t86 — mailbox avatars (`layout.avatars` on `GET /api/accounts` and
//! `PUT /api/accounts/layout`): validated on write (icon allowlist, letter
//! length and characters, hex colours), kept beside the nicknames and order,
//! kept when a client sends no avatars, and per owner.
//!
//! Run: cargo test -p mw-server --test t86_mailbox_avatars -- --test-threads=1
mod common;
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{DomainConfigRow, ServerKey, Store};
use serde_json::{Value, json};
const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

async fn value(response: reqwest::Response, expected: u16) -> Value {
    let status = response.status();
    let text = response.text().await.unwrap();
    assert_eq!(status, expected, "{text}");
    serde_json::from_str(&text).unwrap()
}

struct Owner {
    client: reqwest::Client,
    authority: String,
}

async fn sign_in(base: &str, username: &str, password: &str) -> Owner {
    let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let bare = reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(jar.clone())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    let me = value(
        bare.post(format!("{base}/api/login"))
            .json(&json!({"username":username,"password":password}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-csrf-token",
        me["csrfToken"].as_str().unwrap().parse().unwrap(),
    );
    let authority = me["storageAuthority"].as_str().unwrap().to_owned();
    headers.insert("x-mailwoman-storage-authority", authority.parse().unwrap());
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(jar)
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    Owner { client, authority }
}

#[tokio::test]
async fn avatars_are_validated_kept_with_the_layout_and_per_owner() {
    let mail = common::mock_mail::MockPop3::start_accounts(&[
        ("owner@example.org", "owner-pass"),
        ("other@example.org", "other-pass"),
    ])
    .await;
    let db = common::test_db::unique_db_path("t86-avatars");
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    store
        .put_domain_config(&DomainConfigRow {
            name: "example.org".into(),
            upstream_json: json!({"pop3":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    drop(store);
    let app = build_app(AppConfig {
        db_path: db.to_string_lossy().into(),
        server_key_hex: Some(KEY.into()),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: mw_server::HardeningConfig {
            csrf_strict: true,
            ..Default::default()
        },
        security: SecurityConfig {
            upstream_default: Some(mail.url()),
            ..Default::default()
        },
    })
    .await
    .unwrap();
    let socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", socket.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(
            socket,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap()
    });

    let owner = sign_in(&base, "owner@example.org", "owner-pass").await;
    let other = sign_in(&base, "other@example.org", "other-pass").await;
    assert!(!owner.authority.is_empty());
    let list = |who: &Owner| {
        let request = who.client.get(format!("{base}/api/accounts"));
        async move { value(request.send().await.unwrap(), 200).await }
    };
    let put = |who: &Owner, body: Value| {
        let request = who
            .client
            .put(format!("{base}/api/accounts/layout"))
            .json(&body);
        async move { request.send().await.unwrap() }
    };

    // Nothing saved yet: an empty map (the generated look).
    let first = list(&owner).await;
    assert_eq!(first["layout"]["avatars"], json!({}));

    // An icon with colours for the sign-in mailbox; a key that is not a
    // mailbox is dropped; colours are stored lower case.
    let saved = value(
        put(
            &owner,
            json!({"revision":"","order":["main"],"nicknames":{"main":"Work"},
                "avatars":{"main":{"kind":"icon","icon":"briefcase","fg":"#FFFFFF","bg":"#1D4ED8"},
                           "not-a-link":{"kind":"letter","text":"X"}}}),
        )
        .await,
        200,
    )
    .await;
    let revision = saved["layout"]["revision"].as_str().unwrap().to_owned();
    assert_eq!(
        saved["layout"]["avatars"],
        json!({"main":{"kind":"icon","icon":"briefcase","fg":"#ffffff","bg":"#1d4ed8"}})
    );
    assert_eq!(
        list(&owner).await["layout"]["avatars"]["main"]["icon"],
        "briefcase"
    );

    // Refused, and nothing changes: unknown icon, long or hidden letters, colours
    // that are not #rrggbb, unknown fields, both a letter and an icon.
    for (bad, code) in [
        (json!({"kind":"icon","icon":"rocket"}), "avatarIconUnknown"),
        (
            json!({"kind":"icon","icon":"<svg onload=x>"}),
            "avatarIconUnknown",
        ),
        (json!({"kind":"letter","text":"ABC"}), "avatarTextTooLong"),
        (
            json!({"kind":"letter","text":"A\u{202E}"}),
            "avatarTextInvalid",
        ),
        (
            json!({"kind":"letter","text":"\u{200B}"}),
            "avatarTextInvalid",
        ),
        (
            json!({"kind":"letter","text":"A","fg":"red"}),
            "avatarColorInvalid",
        ),
        (
            json!({"kind":"letter","text":"A","bg":"#fff"}),
            "avatarColorInvalid",
        ),
        (
            json!({"kind":"letter","text":"A","bg":"url(x)"}),
            "avatarColorInvalid",
        ),
        (
            json!({"kind":"letter","text":"A","icon":"lock"}),
            "avatarInvalid",
        ),
        (json!({"kind":"image","text":"A"}), "avatarInvalid"),
    ] {
        let response = put(
            &owner,
            json!({"revision":revision,"order":["main"],"nicknames":{},"avatars":{"main":bad}}),
        )
        .await;
        assert_eq!(response.status(), 422, "{bad}");
        let body: Value = response.json().await.unwrap();
        assert_eq!(
            body["error"].as_str().or(body["code"].as_str()),
            Some(code),
            "{bad} {body}"
        );
    }
    assert_eq!(
        put(
            &owner,
            json!({"revision":revision,"order":["main"],"nicknames":{},
                "avatars":{"main":{"kind":"icon","icon":"lock","url":"https://x.test/a.png"}}}),
        )
        .await
        .status()
        .as_u16()
            / 100,
        4,
        "unknown avatar fields are refused"
    );
    assert_eq!(
        list(&owner).await["layout"]["avatars"]["main"]["icon"],
        "briefcase"
    );

    // A client that sends no avatars (older page) keeps them.
    let kept = value(
        put(
            &owner,
            json!({"revision":revision,"order":["main"],"nicknames":{"main":"Office"}}),
        )
        .await,
        200,
    )
    .await;
    let revision = kept["layout"]["revision"].as_str().unwrap().to_owned();
    assert_eq!(kept["layout"]["avatars"]["main"]["icon"], "briefcase");
    assert_eq!(kept["layout"]["nicknames"]["main"], "Office");

    // A letter (two characters, as typed) with Auto foreground.
    let letter = value(
        put(
            &owner,
            json!({"revision":revision,"order":["main"],"nicknames":{"main":"Office"},
                "avatars":{"main":{"kind":"letter","text":" Ab ","bg":"#fde68a"}}}),
        )
        .await,
        200,
    )
    .await;
    let revision = letter["layout"]["revision"].as_str().unwrap().to_owned();
    assert_eq!(
        letter["layout"]["avatars"]["main"],
        json!({"kind":"letter","text":"Ab","bg":"#fde68a"})
    );

    // Per owner: the other account sees none of it and cannot overwrite it.
    let theirs = list(&other).await;
    assert_eq!(theirs["layout"]["avatars"], json!({}));
    assert_eq!(theirs["layout"]["nicknames"], json!({}));
    value(
        put(
            &other,
            json!({"revision":"","order":["main"],"nicknames":{},
                "avatars":{"main":{"kind":"icon","icon":"crown"}}}),
        )
        .await,
        200,
    )
    .await;
    assert_eq!(
        list(&owner).await["layout"]["avatars"]["main"],
        json!({"kind":"letter","text":"Ab","bg":"#fde68a"})
    );
    assert_eq!(
        list(&other).await["layout"]["avatars"]["main"]["icon"],
        "crown"
    );

    // Back to the default look: an empty map clears it.
    let cleared = value(
        put(
            &owner,
            json!({"revision":revision,"order":["main"],"nicknames":{},"avatars":{}}),
        )
        .await,
        200,
    )
    .await;
    assert_eq!(cleared["layout"]["avatars"], json!({}));
    assert_eq!(list(&owner).await["layout"]["avatars"], json!({}));

    // A display preference only: never part of the identity.
    let me = owner
        .client
        .get(format!("{base}/api/me"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(!me.contains("briefcase") && !me.contains("fde68a"), "{me}");
    server.abort();
}
