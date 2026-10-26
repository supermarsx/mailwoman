use mw_store::{
    Credentials, OAuthClientRow, ServerKey, Store, identity_principal::VerifiedPrimary,
    oidc_provider::*,
};
use serde_json::json;
#[path = "../src/test_db.rs"]
mod test_db;
fn hash(n: u8) -> String {
    format!("{n:064x}")
}
async fn grant_fixture(store: &Store, suffix: u8) -> (OidcGrant, String, String) {
    let now = 1000;
    let session = store
        .create_session(
            "verified-account",
            "address@example.test",
            "engine",
            "engine",
            &Credentials {
                username: "verified-account".into(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap();
    store
        .set_session_authentication(&session, "password+factor")
        .await
        .unwrap();
    let principal = store
        .bind_verified_primary(
            &session,
            VerifiedPrimary {
                account_id: "verified-account",
                kind: "mailbox",
                authority: "verified-backend",
                subject: "immutable-account",
            },
            now,
        )
        .await
        .unwrap()
        .unwrap();
    let client = store.oidc_client("rp").await.unwrap().unwrap();
    let provider = store.oidc_provider_config().await.unwrap().unwrap();
    let request_hash = hash(suffix);
    assert!(
        store
            .begin_oidc_authorization(
                &OidcAuthorization {
                    request_hash: request_hash.clone(),
                    session_id: session.clone(),
                    client_id: "rp".into(),
                    client_revision: client.revision.clone(),
                    provider_revision: provider.revision.clone(),
                    request: json!({"scope":"openid offline_access","nonce":"bound-nonce"}),
                    expires_at: now + 300
                },
                now
            )
            .await
            .unwrap()
    );
    let pending = store
        .oidc_authorization(&request_hash, &session, now)
        .await
        .unwrap()
        .unwrap();
    let (a, b) = tokio::join!(
        store.begin_oidc_authorization(&pending, now + 1),
        store.begin_oidc_authorization(&pending, now + 1)
    );
    assert!(a.unwrap());
    assert!(b.unwrap());
    let grant = OidcGrant {
        id: format!("grant-{suffix}"),
        client_id: "rp".into(),
        client_revision: client.revision,
        provider_revision: provider.revision,
        principal_id: principal.principal_id,
        session_id: session,
        claims: json!({"email":"address@example.test"}),
        scopes: vec!["openid".into(), "offline_access".into()],
        nonce: Some("bound-nonce".into()),
        auth_time: now,
        created_at: now,
        expires_at: now + 3600,
    };
    let code_hash = hash(suffix + 1);
    assert!(
        store
            .issue_oidc_code(OidcIssueCode {
                request_hash: &request_hash,
                grant: &grant,
                code_hash: &code_hash,
                redirect_uri: "https://rp.example/callback",
                pkce_challenge: "bound-s256",
                now
            })
            .await
            .unwrap()
    );
    assert!(
        !store
            .issue_oidc_code(OidcIssueCode {
                request_hash: &request_hash,
                grant: &grant,
                code_hash: &hash(suffix + 2),
                redirect_uri: "https://rp.example/callback",
                pkce_challenge: "bound-s256",
                now
            })
            .await
            .unwrap()
    );
    (grant, code_hash, request_hash)
}
async fn exercise(store: Store) {
    let revision = store
        .put_oidc_provider_config("https://mail.example/oidc", true, None, 1000)
        .await
        .unwrap()
        .unwrap();
    assert!(
        store
            .put_oidc_provider_config("https://evil.example", true, None, 1000)
            .await
            .unwrap()
            .is_none()
    );
    store
        .put_oauth_client(&OAuthClientRow {
            client_id: "rp".into(),
            name: "Independent RP".into(),
            redirect_uris_json: "[\"https://rp.example/callback\"]".into(),
            approved_by: "root".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        })
        .await
        .unwrap();
    let row = OidcClientRow {
        client_id: "rp".into(),
        config: json!({"authMethod":"none"}),
        enabled: true,
        revision: String::new(),
        updated_at: 1000,
        secret_hash: Some("never-serialize-this".into()),
    };
    assert!(
        !serde_json::to_string(&row)
            .unwrap()
            .contains("never-serialize")
    );
    store.put_oidc_client(&row, None).await.unwrap().unwrap();
    assert!(store.put_oidc_client(&row, None).await.unwrap().is_none());
    let (grant, code, _) = grant_fixture(&store, 10).await;
    let a = hash(12);
    let b = hash(13);
    // Wrong binding does not consume the code.
    assert!(matches!(
        store
            .redeem_oidc_token(OidcRedeem {
                scope: None,
                presented_hash: &code,
                client_id: "rp",
                client_revision: &grant.client_revision,
                kind: OidcRedeemKind::Code {
                    redirect_uri: "https://evil.example",
                    pkce_challenge: "bound-s256"
                },
                access_hash: &a,
                refresh_hash: None,
                now: 1001
            })
            .await
            .unwrap(),
        OidcRedemption::Invalid
    ));
    assert!(matches!(
        store
            .redeem_oidc_token(OidcRedeem {
                scope: None,
                presented_hash: &code,
                client_id: "rp",
                client_revision: &grant.client_revision,
                kind: OidcRedeemKind::Code {
                    redirect_uri: "https://rp.example/callback",
                    pkce_challenge: "wrong-s256"
                },
                access_hash: &a,
                refresh_hash: None,
                now: 1001
            })
            .await
            .unwrap(),
        OidcRedemption::Invalid
    ));
    let redeem = |access| OidcRedeem {
        scope: None,
        presented_hash: code.as_str(),
        client_id: "rp",
        client_revision: grant.client_revision.as_str(),
        kind: OidcRedeemKind::Code {
            redirect_uri: "https://rp.example/callback",
            pkce_challenge: "bound-s256",
        },
        access_hash: access,
        refresh_hash: None,
        now: 1001,
    };
    let (first, second) = tokio::join!(
        store.redeem_oidc_token(redeem(&a)),
        store.redeem_oidc_token(redeem(&b))
    );
    let results = [first.unwrap(), second.unwrap()];
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, OidcRedemption::Issued(_)))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, OidcRedemption::Replay))
            .count(),
        1
    );
    assert!(store.oidc_access_grant(&a, 1002).await.unwrap().is_none());
    assert!(store.oidc_access_grant(&b, 1002).await.unwrap().is_none());
    // Refresh replay revokes the entire family, including the winning access.
    let (grant, code, _) = grant_fixture(&store, 30).await;
    let access = hash(32);
    let refresh = hash(33);
    assert!(matches!(
        store
            .redeem_oidc_token(OidcRedeem {
                scope: None,
                presented_hash: &code,
                client_id: "rp",
                client_revision: &grant.client_revision,
                kind: OidcRedeemKind::Code {
                    redirect_uri: "https://rp.example/callback",
                    pkce_challenge: "bound-s256"
                },
                access_hash: &access,
                refresh_hash: Some(&refresh),
                now: 1001
            })
            .await
            .unwrap(),
        OidcRedemption::Issued(_)
    ));
    assert_eq!(
        store
            .oidc_access_grant(&access, 1002)
            .await
            .unwrap()
            .unwrap()
            .nonce
            .as_deref(),
        Some("bound-nonce")
    );
    let a = hash(34);
    let b = hash(35);
    let ra = hash(36);
    let rb = hash(37);
    let redeem = |a, r| OidcRedeem {
        scope: None,
        presented_hash: refresh.as_str(),
        client_id: "rp",
        client_revision: grant.client_revision.as_str(),
        kind: OidcRedeemKind::Refresh,
        access_hash: a,
        refresh_hash: Some(r),
        now: 1003,
    };
    let (first, second) = tokio::join!(
        store.redeem_oidc_token(redeem(&a, &ra)),
        store.redeem_oidc_token(redeem(&b, &rb))
    );
    let results = [first.unwrap(), second.unwrap()];
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, OidcRedemption::Issued(_)))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, OidcRedemption::Replay))
            .count(),
        1
    );
    assert!(
        store
            .oidc_access_grant(&access, 1004)
            .await
            .unwrap()
            .is_none()
    );
    assert!(store.oidc_access_grant(&a, 1004).await.unwrap().is_none());
    assert!(store.oidc_access_grant(&b, 1004).await.unwrap().is_none());
    let (narrow, code, _) = grant_fixture(&store, 60).await;
    assert!(matches!(
        store
            .redeem_oidc_token(OidcRedeem {
                scope: None,
                presented_hash: &code,
                client_id: "rp",
                client_revision: &narrow.client_revision,
                kind: OidcRedeemKind::Code {
                    redirect_uri: "https://rp.example/callback",
                    pkce_challenge: "bound-s256"
                },
                access_hash: &hash(62),
                refresh_hash: Some(&hash(63)),
                now: 1001
            })
            .await
            .unwrap(),
        OidcRedemption::Issued(_)
    ));
    assert!(matches!(
        store
            .redeem_oidc_token(OidcRedeem {
                scope: Some(&["mail.read".into()]),
                presented_hash: &hash(63),
                client_id: "rp",
                client_revision: &narrow.client_revision,
                kind: OidcRedeemKind::Refresh,
                access_hash: &hash(64),
                refresh_hash: Some(&hash(65)),
                now: 1002
            })
            .await
            .unwrap(),
        OidcRedemption::InvalidScope
    ));
    let result = store
        .redeem_oidc_token(OidcRedeem {
            scope: Some(&["openid".into()]),
            presented_hash: &hash(63),
            client_id: "rp",
            client_revision: &narrow.client_revision,
            kind: OidcRedeemKind::Refresh,
            access_hash: &hash(66),
            refresh_hash: Some(&hash(67)),
            now: 1003,
        })
        .await
        .unwrap();
    let OidcRedemption::Issued(narrowed) = result else {
        panic!("valid narrowing refused")
    };
    assert_eq!(narrowed.scopes, vec!["openid"]);
    assert_eq!(
        store
            .oidc_access_grant(&hash(62), 1004)
            .await
            .unwrap()
            .unwrap()
            .scopes,
        vec!["openid"]
    );
    assert!(matches!(
        store
            .redeem_oidc_token(OidcRedeem {
                scope: None,
                presented_hash: &hash(67),
                client_id: "rp",
                client_revision: &narrow.client_revision,
                kind: OidcRedeemKind::Refresh,
                access_hash: &hash(68),
                refresh_hash: None,
                now: 1004
            })
            .await
            .unwrap(),
        OidcRedemption::Invalid
    ));
    // Provider changes invalidate outstanding codes without accepting old authority.
    let (grant, code, _) = grant_fixture(&store, 50).await;
    store
        .put_oidc_provider_config("https://mail.example/oidc", false, Some(&revision), 1005)
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        store
            .redeem_oidc_token(OidcRedeem {
                scope: None,
                presented_hash: &code,
                client_id: "rp",
                client_revision: &grant.client_revision,
                kind: OidcRedeemKind::Code {
                    redirect_uri: "https://rp.example/callback",
                    pkce_challenge: "bound-s256"
                },
                access_hash: &hash(52),
                refresh_hash: None,
                now: 1006
            })
            .await
            .unwrap(),
        OidcRedemption::Invalid
    ));
    store
        .rotate_oidc_signing_key(
            "old",
            "https://mail.example/oidc",
            b"sealed-private-fixture",
            &json!({"kid":"old"}),
            1000,
        )
        .await
        .unwrap();
    store
        .rotate_oidc_signing_key(
            "new",
            "https://mail.example/oidc",
            b"new-private-fixture",
            &json!({"kid":"new"}),
            1010,
        )
        .await
        .unwrap();
    let keys = store
        .oidc_signing_keys("https://mail.example/oidc", 1011)
        .await
        .unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys.iter().filter(|key| key.active).count(), 1);
    assert_eq!(
        store
            .oidc_signing_keys("https://mail.example/oidc", 1910)
            .await
            .unwrap()
            .len(),
        1
    );

    let issuer = "https://mail.example/oidc";
    assert!(
        !store
            .rotate_oidc_signing_key_if_current(OidcKeyRotation {
                kid: "stale",
                issuer,
                private_pem: b"private",
                public_jwk: &json!({"kid":"stale"}),
                expected_kid: Some("old"),
                now: 1020
            })
            .await
            .unwrap()
    );
    assert!(
        store
            .rotate_oidc_signing_key_if_current(OidcKeyRotation {
                kid: "current",
                issuer,
                private_pem: b"private",
                public_jwk: &json!({"kid":"current"}),
                expected_kid: Some("new"),
                now: 1020
            })
            .await
            .unwrap()
    );
    assert_eq!(
        store
            .oidc_signing_keys(issuer, 1021)
            .await
            .unwrap()
            .into_iter()
            .find(|k| k.active)
            .unwrap()
            .kid,
        "current"
    );
    let flow = OidcBrowserFlow {
        flow_hash: hash(100),
        browser_hash: hash(101),
        source_hash: hash(102),
        request: json!({"purpose":"test"}),
        initial_session_id: None,
        created_at: 2000,
        expires_at: 2300,
    };
    assert!(store.begin_oidc_browser_flow(&flow, 2000).await.unwrap());
    assert!(
        store
            .oidc_browser_flow(&flow.flow_hash, &hash(103), 2001)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .oidc_browser_flow(&flow.flow_hash, &flow.browser_hash, 2300)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        !store
            .consume_oidc_browser_flow(&flow.flow_hash, &hash(103), 2001)
            .await
            .unwrap()
    );
    let (a, b) = tokio::join!(
        store.consume_oidc_browser_flow(&flow.flow_hash, &flow.browser_hash, 2001),
        store.consume_oidc_browser_flow(&flow.flow_hash, &flow.browser_hash, 2001)
    );
    assert_ne!(a.unwrap(), b.unwrap());
    assert!(
        store
            .oidc_browser_flow(&flow.flow_hash, &flow.browser_hash, 2002)
            .await
            .unwrap()
            .is_none()
    );
    for i in 110..126 {
        assert!(
            store
                .begin_oidc_browser_flow(
                    &OidcBrowserFlow {
                        flow_hash: hash(i),
                        browser_hash: hash(105),
                        source_hash: hash(106),
                        request: json!({}),
                        initial_session_id: None,
                        created_at: 3000,
                        expires_at: 3300
                    },
                    3000
                )
                .await
                .unwrap()
        );
    }
    assert!(
        !store
            .begin_oidc_browser_flow(
                &OidcBrowserFlow {
                    flow_hash: hash(126),
                    browser_hash: hash(105),
                    source_hash: hash(106),
                    request: json!({}),
                    initial_session_id: None,
                    created_at: 3000,
                    expires_at: 3300
                },
                3000
            )
            .await
            .unwrap()
    );
    let new = OidcClientRow {
        client_id: "managed".into(),
        config: json!({"redirectUris":["https://rp.example/callback"]}),
        enabled: true,
        revision: String::new(),
        updated_at: 4000,
        secret_hash: None,
    };
    let revision = store
        .create_oidc_client(&new, "Original", "admin")
        .await
        .unwrap()
        .unwrap();
    assert!(
        store
            .put_oidc_client_named(&new, Some("stale"), Some("Wrong"))
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .get_oauth_client("managed")
            .await
            .unwrap()
            .unwrap()
            .name,
        "Original"
    );
    let revision = store
        .put_oidc_client_named(&new, Some(&revision), Some("Renamed"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        store
            .get_oauth_client("managed")
            .await
            .unwrap()
            .unwrap()
            .name,
        "Renamed"
    );
    assert!(
        store
            .delete_oidc_client("managed", &revision)
            .await
            .unwrap()
    );
    assert!(store.get_oauth_client("managed").await.unwrap().is_none());
}
#[tokio::test]
async fn oidc_atomic_authority_sqlite() {
    exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn oidc_atomic_authority_postgres() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        return;
    };
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let name = format!("mw_oidc_{:032x}", rand::random::<u128>());
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(&admin)
        .await
        .unwrap();
    let mut url = url::Url::parse(&dsn).unwrap();
    url.set_path(&format!("/{name}"));
    exercise(
        Store::open(url.as_str(), ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
    sqlx::query(&format!("DROP DATABASE {name} WITH (FORCE)"))
        .execute(&admin)
        .await
        .unwrap();
}

#[tokio::test]
async fn token_history_exhaustion_revokes_family_without_forgetting_replay() {
    let path = test_db::unique_db_path("oidc-retention");
    let store = Store::open(path.to_str().unwrap(), ServerKey::generate())
        .await
        .unwrap();
    exercise(store.clone()).await;
    let config = store.oidc_provider_config().await.unwrap().unwrap();
    store
        .put_oidc_provider_config(&config.issuer, true, Some(&config.revision), 1000)
        .await
        .unwrap()
        .unwrap();
    let (grant, code, _) = grant_fixture(&store, 80).await;
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&path))
        .await
        .unwrap();
    sqlx::query("WITH RECURSIVE n(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM n WHERE x<8192) INSERT INTO oidc_token(token_hash,grant_id,kind,expires_at,consumed_at) SELECT 'retained-'||x,?1,'refresh',4600,1000 FROM n")
        .bind(&grant.id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO oidc_token(token_hash,grant_id,kind,expires_at) VALUES('expired-access',?1,'access',999)").bind(&grant.id).execute(&pool).await.unwrap();
    assert!(matches!(
        store
            .redeem_oidc_token(OidcRedeem {
                scope: None,
                presented_hash: &code,
                client_id: "rp",
                client_revision: &grant.client_revision,
                kind: OidcRedeemKind::Code {
                    redirect_uri: "https://rp.example/callback",
                    pkce_challenge: "bound-s256"
                },
                access_hash: &hash(82),
                refresh_hash: Some(&hash(83)),
                now: 1001
            })
            .await
            .unwrap(),
        OidcRedemption::Invalid
    ));
    let revoked: Option<i64> = sqlx::query_scalar("SELECT revoked_at FROM oidc_grant WHERE id=?1")
        .bind(&grant.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(revoked, Some(1001));
    let retained: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM oidc_token WHERE grant_id=?1 AND kind='refresh'")
            .bind(&grant.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(retained, 8192);
    let expired: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM oidc_token WHERE token_hash='expired-access'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(expired, 0);
    assert!(
        store
            .oidc_access_grant(&hash(82), 1002)
            .await
            .unwrap()
            .is_none()
    );
    pool.close().await;
}
