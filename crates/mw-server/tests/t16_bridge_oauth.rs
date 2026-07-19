//! t16-e-e2e — bridge OAuth token cache (B1): SEALED 0018 on the REAL store.
//!
//! Scope note (honest): the OAuth *client* (`crates/mw-server/src/oauth_client.rs`) is a
//! private `#[path]` child module of `v7_mount` — its `acquire_access_token` +
//! device/auth-code/refresh flows are unit-tested IN-CRATE against a mock `FormPoster`
//! (an in-process mock IdP; e7 shipped 10/10 including the "cached-expired → refresh →
//! re-cache, retaining the refresh token" path). Those are not reachable from an
//! integration test, and a LIVE M365/Workspace tenant refresh is LPB (loud-skipped
//! everywhere — no tenant credentials in CI). The e7 unit test explicitly defers ONE
//! piece to this lane: "the e2e lane re-verifies non-plaintext bytes on the real
//! filesystem." That is what this leg proves — the 0018 `bridge_oauth_tokens` cache the
//! refresh path writes is SEALED at rest on a real on-disk SQLite store AND round-trips
//! on live Postgres (`MW_E14_PG_DSN`), the second dialect.

use mw_store::{BridgeOauthTokenRow, ServerKey, Store};

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

fn read_all_db_bytes(db_path: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for suffix in ["", "-wal", "-journal", "-shm"] {
        if let Ok(b) = std::fs::read(format!("{db_path}{suffix}")) {
            out.extend_from_slice(&b);
        }
    }
    out
}

#[tokio::test]
async fn bridge_oauth_tokens_are_sealed_on_the_real_store_sqlite() {
    let dir = std::env::temp_dir().join(format!("mw-t16-boauth-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    let store = Store::open(&db, ServerKey::generate()).await.unwrap();

    // Distinctive access + refresh tokens (as the refresh path would re-cache).
    let access = "AT-LIVE-SECRET-abcdef0123456789";
    let refresh = "RT-LONG-LIVED-SECRET-zyxwvu987654";
    store
        .put_bridge_oauth_token(&BridgeOauthTokenRow {
            bridge_account_id: "alice@corp".into(),
            access_token: access.into(),
            refresh_token: Some(refresh.into()),
            expires_at: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
            scope: "Mail.Read".into(),
            updated_at: String::new(),
        })
        .await
        .unwrap();

    // Round-trip works (seal/unseal correct).
    let got = store
        .get_bridge_oauth_token("alice@corp")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.access_token, access);
    assert_eq!(got.refresh_token.as_deref(), Some(refresh));

    // The tokens are SEALED at rest: neither the access nor the (long-lived) refresh
    // token appears in plaintext in the real store file.
    drop(store);
    let bytes = read_all_db_bytes(&db);
    assert!(
        !contains(&bytes, access.as_bytes()),
        "the bridge access token must be sealed at rest, never plaintext in the store"
    );
    assert!(
        !contains(&bytes, refresh.as_bytes()),
        "the long-lived refresh token must be sealed at rest, never plaintext in the store"
    );
    eprintln!(
        "[t16 bridge-oauth] 0018 tokens sealed on the real SQLite store. \
         NOTE: mock-IdP refresh is proven in-crate (e7); live tenant is LPB."
    );
}

#[tokio::test]
async fn bridge_oauth_token_round_trips_on_live_postgres() {
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!(
            "\n[t16 bridge-oauth SKIP] MW_E14_PG_DSN unset — 0018 SQL + seal not exercised on live Postgres.\n"
        );
        return;
    };
    let store = Store::open(&dsn, ServerKey::generate())
        .await
        .expect("open live Postgres store");
    let acct = format!("pg-bridge-{}", unique());
    let access = format!("AT-{}", unique());
    store
        .put_bridge_oauth_token(&BridgeOauthTokenRow {
            bridge_account_id: acct.clone(),
            access_token: access.clone(),
            refresh_token: Some("RT-pg".into()),
            expires_at: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
            scope: "https://graph.microsoft.com/.default".into(),
            updated_at: String::new(),
        })
        .await
        .unwrap();
    // The 0018 SQL + seal/unseal round-trips in the Postgres dialect.
    let got = store.get_bridge_oauth_token(&acct).await.unwrap().unwrap();
    assert_eq!(got.access_token, access);
    assert_eq!(got.refresh_token.as_deref(), Some("RT-pg"));
    // Deleting the cache (re-enrolment) works too.
    store.delete_bridge_oauth_token(&acct).await.unwrap();
    assert!(store.get_bridge_oauth_token(&acct).await.unwrap().is_none());
}
