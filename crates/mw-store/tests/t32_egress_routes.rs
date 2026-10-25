//! 0033 egress route policy and the internal allowlist on both dialects (t32 e31b).
//!
//! The unit tests in `src/egress_config.rs` run on SQLite. This file runs the same
//! store calls on SQLite and, when `DATABASE_URL_PG` (or `MW_TEST_PG`) is set, on
//! Postgres in a scratch schema — the `ON CONFLICT … CASE` that carries or clears
//! the remote-DNS acceptance is the statement most likely to differ between them.
//! With `MW_REQUIRE_LIVE=1` a missing Postgres fails instead of skipping.
//!
//! Run:
//!   DATABASE_URL_PG=postgres://… MW_REQUIRE_LIVE=1 \
//!     cargo test -p mw-store --test t32_egress_routes -- --test-threads=1

use mw_store::{EgressAllowRow, EgressProxyRow, ServerKey, Store};

fn pg_dsn() -> Option<String> {
    std::env::var("DATABASE_URL_PG")
        .or_else(|_| std::env::var("MW_TEST_PG"))
        .ok()
        .filter(|v| !v.trim().is_empty())
}

fn require_live() -> bool {
    std::env::var("MW_REQUIRE_LIVE").is_ok_and(|v| v == "1")
}

/// A Postgres store in a fresh schema, plus what is needed to drop it.
async fn pg_store(dsn: &str, tag: &str) -> (Store, sqlx::PgPool, String) {
    let schema = format!(
        "t32_f17_{tag}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let admin = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(dsn)
        .await
        .expect("DATABASE_URL_PG is set but Postgres is not reachable");
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let sep = if dsn.contains('?') { '&' } else { '?' };
    let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");
    let store = Store::open_postgres(&scoped, ServerKey::generate())
        .await
        .unwrap();
    (store, admin, schema)
}

/// Run `body` on SQLite, then on Postgres when configured.
macro_rules! both_dialects {
    ($tag:literal, $body:expr) => {{
        let sqlite = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        $body(sqlite).await;
        match pg_dsn() {
            Some(dsn) => {
                let (pg, admin, schema) = pg_store(&dsn, $tag).await;
                $body(pg).await;
                let _ = sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
                    .execute(&admin)
                    .await;
            }
            None if require_live() => {
                panic!("MW_REQUIRE_LIVE=1 but DATABASE_URL_PG and MW_TEST_PG are unset")
            }
            None => eprintln!(
                "[mw-store] t32_egress_routes {}: DATABASE_URL_PG and MW_TEST_PG unset — \
                 Postgres leg skipped",
                $tag
            ),
        }
    }};
}

fn route(id: &str) -> EgressProxyRow {
    EgressProxyRow {
        id: id.into(),
        kind: "socks5h".into(),
        host: "tor.corp.example".into(),
        port: 9050,
        username: String::new(),
        password: None,
        allow_plaintext: false,
        active: false,
        tls_to_proxy: false,
        remote_dns: true,
        remote_dns_risk_accepted_by: None,
        remote_dns_risk_accepted_at: None,
        tunnel_id: None,
        chain_next_id: None,
        scope: "domain:corp.example".into(),
        traffic_classes: vec!["remote_content".into(), "keyservers".into()],
        created_at: String::new(),
        updated_at: String::new(),
    }
}

async fn accepted_by(s: &Store, id: &str) -> Option<String> {
    s.get_egress_proxy(id)
        .await
        .unwrap()
        .unwrap()
        .remote_dns_risk_accepted_by
}

#[tokio::test]
async fn route_policy_round_trips_and_the_acceptance_follows_what_was_accepted() {
    both_dialects!("policy", |s: Store| async move {
        s.put_egress_proxy(&route("tor")).await.unwrap();
        let got = s.get_egress_proxy("tor").await.unwrap().unwrap();
        assert_eq!(got.kind, "socks5h");
        assert_eq!(got.scope, "domain:corp.example");
        assert_eq!(got.traffic_classes, ["remote_content", "keyservers"]);
        assert!(got.remote_dns);
        assert_eq!(got.remote_dns_risk_accepted_by, None);

        assert!(
            s.accept_egress_remote_dns_risk("tor", "root", 1_700_000_000)
                .await
                .unwrap()
        );
        let got = s.get_egress_proxy("tor").await.unwrap().unwrap();
        assert_eq!(got.remote_dns_risk_accepted_by.as_deref(), Some("root"));
        assert_eq!(got.remote_dns_risk_accepted_at, Some(1_700_000_000));

        // An edit that keeps kind, host, port and remote DNS keeps the acceptance.
        let mut edit = route("tor");
        edit.traffic_classes = vec!["remote_content".into()];
        s.put_egress_proxy(&edit).await.unwrap();
        assert_eq!(accepted_by(&s, "tor").await.as_deref(), Some("root"));

        // Moving the route to another proxy clears it.
        let mut moved = route("tor");
        moved.host = "elsewhere.example".into();
        s.put_egress_proxy(&moved).await.unwrap();
        assert_eq!(accepted_by(&s, "tor").await, None);

        // A route without remote DNS cannot be given an acceptance.
        let mut plain = route("plain");
        plain.kind = "socks5".into();
        plain.remote_dns = false;
        s.put_egress_proxy(&plain).await.unwrap();
        assert!(
            !s.accept_egress_remote_dns_risk("plain", "root", 1)
                .await
                .unwrap()
        );
        assert_eq!(accepted_by(&s, "plain").await, None);

        // Live routes are the active rows.
        assert!(s.live_egress_routes().await.unwrap().is_empty());
        s.set_active_egress_proxy(Some("plain")).await.unwrap();
        let live: Vec<_> = s
            .live_egress_routes()
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.id)
            .collect();
        assert_eq!(live, ["plain"]);
    });
}

#[tokio::test]
async fn the_internal_allowlist_keeps_other_classes_out_on_both_dialects() {
    both_dialects!("allow", |s: Store| async move {
        let row = |id: &str, class: &str| EgressAllowRow {
            id: id.into(),
            traffic_class: class.into(),
            destination: "10.20.0.0/16".into(),
            note: String::new(),
            created_by: "root".into(),
            created_at: 1_700_000_000,
        };
        s.add_egress_internal_allow(&row("s", "siem"))
            .await
            .unwrap();
        s.add_egress_internal_allow(&row("w", "webhooks"))
            .await
            .unwrap();
        assert!(
            s.add_egress_internal_allow(&row("r", "remote_content"))
                .await
                .is_err()
        );
        let ids: Vec<_> = s
            .list_egress_internal_allow()
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.id)
            .collect();
        assert_eq!(ids, ["s", "w"]);
        assert!(s.delete_egress_internal_allow("s").await.unwrap());
        assert!(!s.delete_egress_internal_allow("s").await.unwrap());
    });
}
