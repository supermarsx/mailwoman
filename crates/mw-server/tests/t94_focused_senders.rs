//! t94: `/api/account/focused-senders`, the per-user Focused inbox overrides.
//! Two signed-in users share the same upstream account id ("1") in different
//! realms; each reads and changes only their own rows. Writes need the
//! storage-authority header; a request without a session is refused.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{Credentials, ServerKey, Store, identity_principal::VerifiedPrimary};
use serde_json::{Value, json};
mod common;
const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

struct User {
    client: reqwest::Client,
    authority: String,
}

#[tokio::test]
async fn overrides_are_owner_scoped_validated_and_imported_once() {
    let db = common::test_db::unique_db_path("t94-focused-senders");
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(KEY.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        },
        V6Config::default(),
    )
    .await
    .unwrap();
    let socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", socket.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            socket,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let mut users = Vec::new();
    for path in ["first", "second"] {
        let service = format!("https://mail.example.test/{path}/session");
        let username = format!("alice@{path}.example.test");
        store
            .put_domain_config(&mw_store::DomainConfigRow {
                name: format!("{path}.example.test"),
                upstream_json: json!({"jmap":{"sessionUrl":service}}).to_string(),
                allowlist_json: "[]".into(),
                blocklist_json: "[]".into(),
                enabled: true,
                preferred: false,
                settings_json: "{}".into(),
                updated_at: 0,
            })
            .await
            .unwrap();
        let id = store
            .create_session(
                "1",
                &username,
                &service,
                &format!("https://mail.example.test/{path}/api"),
                &Credentials {
                    username: username.clone(),
                    password: "synthetic".into(),
                },
            )
            .await
            .unwrap();
        store
            .set_session_authentication(&id, "header-auth")
            .await
            .unwrap();
        store
            .bind_verified_primary(
                &id,
                VerifiedPrimary {
                    account_id: "1",
                    kind: "header",
                    authority: "deployment:trusted-header",
                    subject: "alice",
                },
                100,
            )
            .await
            .unwrap()
            .unwrap();
        let authority = store.session_storage_authority(&id).await.unwrap().unwrap();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("cookie", format!("mw_session={id}").parse().unwrap());
        users.push(User {
            client: reqwest::Client::builder()
                .no_proxy()
                .default_headers(headers)
                .build()
                .unwrap(),
            authority,
        });
    }
    let url = format!("{base}/api/account/focused-senders");
    let list = |u: &User| {
        let req = u.client.get(&url);
        async move {
            let resp = req.send().await.unwrap();
            assert_eq!(resp.status(), 200);
            assert_eq!(resp.headers()["cache-control"], "no-store");
            resp.json::<Value>().await.unwrap()["senders"].clone()
        }
    };
    let put = |u: &User, body: Value, authority: bool| {
        let mut req = u.client.put(&url).json(&body);
        if authority {
            req = req.header("x-mailwoman-storage-authority", &u.authority);
        }
        async move { req.send().await.unwrap().status().as_u16() }
    };

    // No session: refused.
    let anon = reqwest::Client::builder().no_proxy().build().unwrap();
    assert_eq!(anon.get(&url).send().await.unwrap().status(), 401);
    assert_eq!(
        anon.put(&url)
            .json(&json!({"email":"a@x.example","tab":"other"}))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );

    let (a, b) = (&users[0], &users[1]);
    assert_eq!(list(a).await, json!([]));
    // A write without the storage authority is refused (428), with the wrong
    // authority too (409).
    assert_eq!(
        put(a, json!({"email":"news@shop.example","tab":"other"}), false).await,
        428
    );
    let wrong = a
        .client
        .put(&url)
        .header("x-mailwoman-storage-authority", &b.authority)
        .json(&json!({"email":"news@shop.example","tab":"other"}))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(wrong, 409);

    assert_eq!(
        put(a, json!({"email":"News@Shop.Example","tab":"other"}), true).await,
        200
    );
    assert_eq!(
        put(
            a,
            json!({"email":"boss@work.example","tab":"focused"}),
            true
        )
        .await,
        200
    );
    assert_eq!(
        put(
            b,
            json!({"email":"news@shop.example","tab":"focused"}),
            true
        )
        .await,
        200
    );
    let rows = list(a).await;
    assert_eq!(rows.as_array().unwrap().len(), 2);
    assert_eq!(rows[0]["email"], "boss@work.example");
    assert_eq!(rows[0]["tab"], "focused");
    assert_eq!(rows[1]["email"], "news@shop.example");
    assert_eq!(rows[1]["tab"], "other");
    assert!(rows[1]["updatedAt"].as_i64().unwrap() > 0);

    // Isolation: b sees only b's row, and b removing the shared sender leaves a's.
    let theirs = list(b).await;
    assert_eq!(theirs.as_array().unwrap().len(), 1);
    assert_eq!(theirs[0]["tab"], "focused");
    assert_eq!(
        put(b, json!({"email":"news@shop.example","tab":null}), true).await,
        200
    );
    assert_eq!(
        put(b, json!({"email":"boss@work.example","tab":null}), true).await,
        200
    );
    assert_eq!(list(b).await, json!([]));
    assert_eq!(list(a).await.as_array().unwrap().len(), 2);

    // Validation.
    for bad in [
        json!({"email":"Name <a@x.example>","tab":"other"}),
        json!({"email":"a@x.example,b@x.example","tab":"other"}),
        json!({"email":"a@x.example","tab":"spam"}),
    ] {
        assert_eq!(put(a, bad.clone(), true).await, 400, "{bad}");
    }
    assert_eq!(
        put(
            a,
            json!({"email":"a@x.example","tab":"other","owner":"x"}),
            true
        )
        .await,
        422,
        "unknown fields are refused"
    );

    // Import: rows already on the server win; invalid entries are skipped.
    let import = a
        .client
        .post(format!("{url}/import"))
        .header("x-mailwoman-storage-authority", &a.authority)
        .json(&json!({"senders":{
            "news@shop.example":"focused",
            "old@legacy.example":"other",
            "not an address":"other",
            "bad@tab.example":"bogus"
        }}))
        .send()
        .await
        .unwrap();
    assert_eq!(import.status(), 200);
    let import = import.json::<Value>().await.unwrap();
    assert_eq!(import["added"], 1);
    let emails: Vec<(&str, &str)> = import["senders"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["email"].as_str().unwrap(), r["tab"].as_str().unwrap()))
        .collect();
    assert_eq!(
        emails,
        [
            ("boss@work.example", "focused"),
            ("news@shop.example", "other"),
            ("old@legacy.example", "other")
        ]
    );
    assert_eq!(
        list(b).await,
        json!([]),
        "an import never reaches another user"
    );
    let no_authority = a
        .client
        .post(format!("{url}/import"))
        .json(&json!({"senders":{}}))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(no_authority, 428);
}
