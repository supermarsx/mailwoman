//! Transactional domain ownership and preference invariants on both stores.
use mw_store::{DomainConfigRow, DomainMutation as M, ServerKey, Store};
fn row(name: &str) -> DomainConfigRow {
    DomainConfigRow {
        name: name.into(),
        upstream_json: "{}".into(),
        allowlist_json: "[]".into(),
        blocklist_json: "[]".into(),
        enabled: true,
        preferred: false,
        settings_json: "{}".into(),
        updated_at: 1,
    }
}
async fn exercise(store: Store) {
    let a = row("a.test");
    let b = row("b.test");
    store.put_domain_config(&a).await.unwrap();
    store.put_domain_config(&b).await.unwrap();
    assert_eq!(
        store
            .compare_and_prefer_domain(None, Some(&a), true)
            .await
            .unwrap(),
        M::Applied
    );
    assert!(store.preferred_domain().await.unwrap().is_none());
    let (one, two) = tokio::join!(
        store.compare_and_prefer_domain(None, Some(&a), false),
        store.compare_and_prefer_domain(None, Some(&b), false)
    );
    assert!(matches!(
        (one.unwrap(), two.unwrap()),
        (M::Applied, M::Stale) | (M::Stale, M::Applied)
    ));
    assert_eq!(
        store
            .compare_and_prefer_domain(None, None, false)
            .await
            .unwrap(),
        M::Stale
    );
    let preferred = store.preferred_domain_record().await.unwrap().unwrap();
    assert_eq!(
        store
            .compare_and_prefer_domain(Some(&preferred), None, true)
            .await
            .unwrap(),
        M::Applied
    );
    assert_eq!(
        store.preferred_domain().await.unwrap(),
        Some(preferred.name.clone())
    );
    assert_eq!(
        store
            .compare_and_prefer_domain(Some(&preferred), None, false)
            .await
            .unwrap(),
        M::Applied
    );
    let a = store.get_domain_config("a.test").await.unwrap().unwrap();
    let b = store.get_domain_config("b.test").await.unwrap().unwrap();
    assert_eq!(
        store
            .compare_and_map_domain(&a, &[], &[], "alias.test", false, false, true)
            .await
            .unwrap(),
        M::Applied
    );
    assert!(
        store
            .domain_alias_target("alias.test")
            .await
            .unwrap()
            .is_none()
    );
    let (one, two) = tokio::join!(
        store.compare_and_map_domain(&a, &[], &[], "alias.test", false, false, false),
        store.compare_and_map_domain(&b, &[], &[], "alias.test", false, false, false)
    );
    assert!(matches!(
        (one.unwrap(), two.unwrap()),
        (M::Applied, M::Conflict) | (M::Conflict, M::Applied)
    ));
    let owner = store
        .domain_alias_target("alias.test")
        .await
        .unwrap()
        .unwrap();
    let other = if owner == a.name { &b } else { &a };
    assert!(
        store
            .put_domain_alias("alias.test", &other.name)
            .await
            .is_err()
    );
    assert_eq!(
        store.domain_alias_target("alias.test").await.unwrap(),
        Some(owner.clone())
    );
    assert_eq!(
        store
            .compare_and_map_domain(other, &[], &[], &owner, false, false, false)
            .await
            .unwrap(),
        M::Shadow
    );
    assert!(
        !store
            .compare_and_put_domain_config(None, &row("alias.test"))
            .await
            .unwrap()
    );
    assert!(store.put_domain_config(&row("alias.test")).await.is_err());
    let own = store.get_domain_config(&owner).await.unwrap().unwrap();
    assert_eq!(
        store
            .compare_and_map_domain(&own, &[], &[], "new.test", false, false, false)
            .await
            .unwrap(),
        M::Stale
    );
    let aliases = store.domain_aliases(&owner).await.unwrap();
    store.put_domain_host("login.test", &owner).await.unwrap();
    assert!(
        store
            .put_domain_host("login.test", &other.name)
            .await
            .is_err()
    );
    assert_eq!(
        store
            .compare_and_map_domain(&own, &aliases, &[], "alias.test", false, true, false)
            .await
            .unwrap(),
        M::Stale
    );
    let hosts = store.domain_hosts(&owner).await.unwrap();
    assert_eq!(
        store
            .compare_and_map_domain(&own, &aliases, &hosts, "alias.test", false, true, true)
            .await
            .unwrap(),
        M::Applied
    );
    assert!(
        store
            .domain_alias_target("alias.test")
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        store
            .compare_and_map_domain(&own, &aliases, &hosts, "alias.test", false, true, false)
            .await
            .unwrap(),
        M::Applied
    );
    assert!(
        store
            .domain_alias_target("alias.test")
            .await
            .unwrap()
            .is_none()
    );
    let mut disabled = other.clone();
    disabled.enabled = false;
    store.put_domain_config(&disabled).await.unwrap();
    assert_eq!(
        store
            .compare_and_prefer_domain(None, Some(&disabled), false)
            .await
            .unwrap(),
        M::Disabled
    );
    let target = store.get_domain_config(&owner).await.unwrap().unwrap();
    let aliases = store.domain_aliases(&owner).await.unwrap();
    let hosts = store.domain_hosts(&owner).await.unwrap();
    let candidate = row("race.test");
    let (canonical, mapping) = tokio::join!(
        store.compare_and_put_domain_config(None, &candidate),
        store.compare_and_map_domain(&target, &aliases, &hosts, "race.test", false, false, false)
    );
    assert!(
        matches!(
            (canonical.unwrap(), mapping.unwrap()),
            (true, M::Shadow) | (false, M::Applied)
        ),
        "canonical names and aliases have one winner"
    );
}
#[tokio::test]
async fn sqlite_domain_concurrency() {
    exercise(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_domain_concurrency() {
    let Ok(dsn) = std::env::var("MW_DOMAIN_TEST_PG_DSN") else {
        eprintln!("SKIP Postgres: MW_DOMAIN_TEST_PG_DSN unset");
        return;
    };
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    let schema = format!("domain_concurrency_{}", rand::random::<u64>());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&pool)
        .await
        .unwrap();
    let sep = if dsn.contains('?') { '&' } else { '?' };
    let scoped = format!("{dsn}{sep}options=-c%20search_path%3D{schema}");
    let result = tokio::spawn(async move {
        exercise(
            Store::open_postgres(&scoped, ServerKey::generate())
                .await
                .unwrap(),
        )
        .await;
    })
    .await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&pool)
        .await
        .unwrap();
    result.unwrap();
}
