use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{ServerKey, Store};
use serde_json::{Value, json};
use std::net::SocketAddr;
mod common;

#[tokio::test]
async fn local_photos_require_opt_in_owned_contacts_and_reencode_to_private_png() {
    // This integration binary has one test and owns its environment.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        std::env::set_var(
            "MW_UPLOAD_DIR",
            common::test_db::unique_dir("photo-uploads"),
        );
    }
    let upstream = common::mock_imap::MockImap::start("owner@example.org", "testpass").await;
    let db = common::test_db::unique_db_path("contact-photos");
    let key = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    let store = Store::open(&db.to_string_lossy(), ServerKey::from_hex(key).unwrap())
        .await
        .unwrap();
    let (router, _) = mw_server::build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(key.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Engine,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.url()),
                ..SecurityConfig::default()
            },
        },
        V6Config::default(),
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let endpoint = format!("{base}/api/account/contact-photo");
    assert_eq!(
        client
            .post(&endpoint)
            .json(&json!({"address":"ann@example.org"}))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let login = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"owner@example.org","password":"testpass"}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    let account = login.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_owned();
    let photo = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGNQKHjwHwAElAJwfN7sMwAAAABJRU5ErkJggg==";
    let book = "contact-photo-owned";
    store
        .upsert_address_book(&mw_store::AddressBookRow {
            id: book.into(),
            account_id: account,
            name: "Contacts".into(),
            is_default: true,
            carddav_url: None,
            sync_token: None,
            ctag: None,
        })
        .await
        .unwrap();
    let mut card = mw_store::ContactRow {
        id: "contact-photo-ann".into(),
        address_book_id: book.into(),
        uid: "ann".into(),
        etag: None,
        vcard_raw: format!(
            "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Ann\r\nEMAIL:ann@example.org\r\nNOTE:other@example.org\r\nPHOTO;ENCODING=b;TYPE=PNG:{photo}\r\nEND:VCARD\r\n"
        ),
        json: None,
        full_name: "Ann".into(),
        is_favorite: false,
        photo_blob_id: None,
        pgp_key: None,
        smime_cert: None,
    };
    store.upsert_contact(&card).await.unwrap();
    assert_eq!(
        client
            .post(&endpoint)
            .json(&json!({"address":"ann@example.org"}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let setting = format!("{base}/api/settings/contacts.avatarMode");
    let enabled = client
        .put(&setting)
        .json(&json!({"value":"contacts"}))
        .send()
        .await
        .unwrap();
    assert_eq!(enabled.status(), 200, "{}", enabled.text().await.unwrap());
    let response = client
        .post(&endpoint)
        .json(&json!({"address":"ann@example.org"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "image/png");
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    assert!(
        response
            .bytes()
            .await
            .unwrap()
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );
    assert_eq!(
        client
            .post(&endpoint)
            .json(&json!({"address":"other@example.org"}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(
        client
            .post(&endpoint)
            .json(&json!({"address":"ann@example.org","accountId":"foreign"}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    card.vcard_raw = card.vcard_raw.replace(
        &format!("PHOTO;ENCODING=b;TYPE=PNG:{photo}"),
        "PHOTO:https://127.0.0.1:1/private",
    );
    store.upsert_contact(&card).await.unwrap();
    assert_eq!(
        client
            .post(&endpoint)
            .json(&json!({"address":"ann@example.org"}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "contacts.avatarMode".into(),
            global_value: Some("\"off\"".into()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    assert_eq!(
        client
            .put(&setting)
            .json(&json!({"value":"contacts"}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client
            .post(&endpoint)
            .json(&json!({"address":"ann@example.org"}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
}
