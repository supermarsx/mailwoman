use super::*;
use crate::settings_policy::SettingSubject;

fn subject() -> SettingSubject<'static> {
    SettingSubject {
        personal_owner: None,
        domain: "",
        account_id: "",
        address: "",
    }
}
fn domain(name: &str) -> crate::DomainConfigRow {
    crate::DomainConfigRow {
        name: name.into(),
        upstream_json: "deliberately unparsed".into(),
        allowlist_json: "invalid but unused".into(),
        blocklist_json: "[]".into(),
        enabled: true,
        preferred: false,
        settings_json: "{}".into(),
        updated_at: 1,
    }
}
async fn exercise(reader: Store, writer: Store) {
    writer
        .put_domain_config(&domain("example.test"))
        .await
        .unwrap();
    writer
        .put_domain_alias("alias.test", "example.test")
        .await
        .unwrap();
    let domains = vec![
        "example.test".into(),
        "alias.test".into(),
        "missing.test".into(),
    ];
    let key = "v6:admin:flags:user@example.test";
    let keys = vec![key.into()];
    let snapshot = reader
        .final_account_gate_snapshot(&domains, &keys)
        .await
        .unwrap();
    assert!(
        !reader
            .try_empty_global_resolution_ready("signin.domainMode", &snapshot.revision)
            .unwrap()
    );
    reader
        .settings_resolution_inputs_for(&subject(), "signin.domainMode")
        .await
        .unwrap();
    let (snapshot, counts) =
        crate::pool_telemetry::measure_queries(reader.final_account_gate_snapshot(&domains, &keys))
            .await;
    let mut snapshot = snapshot.unwrap();
    assert_eq!(counts.read.calls, 1, "all fresh inputs use one SELECT");
    assert!(
        reader
            .try_empty_global_resolution_ready("signin.domainMode", &snapshot.revision)
            .unwrap(),
        "cached absent policy is a valid warm hit"
    );
    assert_eq!(
        snapshot
            .domains
            .remove("alias.test")
            .unwrap()
            .unwrap()
            .canonical_domain,
        "example.test"
    );
    assert_eq!(
        snapshot
            .domains
            .remove("missing.test")
            .unwrap()
            .unwrap()
            .enabled,
        None
    );
    assert!(snapshot.global_policy.unwrap().is_none());

    // A different Store writer needs no invalidation call on this reader.
    let mut disabled = domain("example.test");
    disabled.enabled = false;
    writer.put_domain_config(&disabled).await.unwrap();
    writer
        .set_setting(key, "{\"disabled\":true}")
        .await
        .unwrap();
    q("INSERT INTO mail_user(address,domain,local_part,origin,status,created_at) VALUES('a@example.test','example.test','a','import','disabled',1),('z@bad/path','example.test','z','import','deleted',1)")
        .execute(&writer.backend).await.unwrap();
    let mut fresh = reader
        .final_account_gate_snapshot(&domains, &keys)
        .await
        .unwrap();
    assert_eq!(
        fresh.domains.remove("alias.test").unwrap().unwrap().enabled,
        Some(false)
    );
    assert_eq!(
        fresh.exact_flags.remove(key).unwrap().unwrap(),
        "{\"disabled\":true}"
    );
    assert_eq!(
        fresh
            .held_addresses
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        vec!["a@example.test", "z@bad/path"]
    );
    writer
        .put_setting_policy(&SettingPolicyRow {
            key: "signin.domainMode".into(),
            global_value: Some("\"listed\"".into()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "test".into(),
            updated_at: 2,
        })
        .await
        .unwrap();
    let fresh = reader
        .final_account_gate_snapshot(&domains, &keys)
        .await
        .unwrap();
    assert!(
        !reader
            .try_empty_global_resolution_ready("signin.domainMode", &fresh.revision)
            .unwrap(),
        "fresh policy revision must invalidate warm eligibility"
    );
    assert_eq!(
        fresh
            .global_policy
            .unwrap()
            .unwrap()
            .global_value
            .as_deref(),
        Some("\"listed\"")
    );
    writer
        .put_domain_config(&domain("other.test"))
        .await
        .unwrap();
    q("INSERT INTO domain_alias(alias,domain) VALUES('example.test','other.test')")
        .execute(&writer.backend)
        .await
        .unwrap();
    let mut chained = reader
        .final_account_gate_snapshot(&domains, &keys)
        .await
        .unwrap();
    assert!(
        chained
            .domains
            .remove("alias.test")
            .unwrap()
            .unwrap()
            .second_alias,
        "raw legacy shadow/second hop must force unchanged fallback"
    );
    reader.close().await;
    writer.close().await;
}

#[tokio::test]
async fn sqlite_final_gate_snapshot_is_fresh_bounded_and_single_query() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gate.db");
    let key = crate::ServerKey::generate();
    let reader = Store::open(path.to_str().unwrap(), key.clone())
        .await
        .unwrap();
    let writer = Store::open(path.to_str().unwrap(), key).await.unwrap();
    exercise(reader, writer).await;
}

#[tokio::test]
async fn postgres_final_gate_snapshot_is_fresh_bounded_and_single_query() {
    use sqlx::Connection;
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        eprintln!("SKIP: MW_TEST_PG absent");
        return;
    };
    let schema = format!("final_gate_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut url = url::Url::parse(&dsn).unwrap();
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let key = crate::ServerKey::generate();
    exercise(
        Store::open(url.as_str(), key.clone()).await.unwrap(),
        Store::open(url.as_str(), key).await.unwrap(),
    )
    .await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}

#[tokio::test]
async fn readiness_is_store_bound_zero_is_valid_and_missing_revision_disables_it() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let other = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    q("UPDATE settings_revision SET n=0 WHERE id=1")
        .execute(&store.backend)
        .await
        .unwrap();
    store
        .settings_resolution_inputs_for(&subject(), "signin.domainMode")
        .await
        .unwrap();
    let snapshot = store.final_account_gate_snapshot(&[], &[]).await.unwrap();
    assert_eq!(snapshot.revision.number(), Some(0));
    let (ready, counts) = crate::pool_telemetry::measure_queries(async {
        store.try_empty_global_resolution_ready("signin.domainMode", &snapshot.revision)
    })
    .await;
    assert!(ready.unwrap());
    assert_eq!(counts.read.calls, 0);
    assert!(
        other
            .try_empty_global_resolution_ready("signin.domainMode", &snapshot.revision)
            .is_err()
    );
    q("DELETE FROM settings_revision WHERE id=1")
        .execute(&store.backend)
        .await
        .unwrap();
    let missing = store.final_account_gate_snapshot(&[], &[]).await.unwrap();
    assert_eq!(missing.revision.number(), None);
    assert!(
        !store
            .try_empty_global_resolution_ready("signin.domainMode", &missing.revision)
            .unwrap()
    );
}

#[tokio::test]
async fn input_limits_reject_before_query_and_hold_limit_is_global() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    for (domains, keys) in [
        (vec!["a".into(); 257], vec![]),
        (vec![], vec!["x".into(); 257]),
        (vec![], vec!["x".repeat(MAX_FINAL_GATE_PARAMETER_BYTES + 1)]),
    ] {
        let (result, counts) = crate::pool_telemetry::measure_queries(
            store.final_account_gate_snapshot(&domains, &keys),
        )
        .await;
        assert!(result.is_err());
        assert_eq!(counts.read.calls, 0);
    }
    q("WITH RECURSIVE n(value) AS (SELECT 1 UNION ALL SELECT value+1 FROM n WHERE value<5001) INSERT INTO mail_user(address,domain,local_part,origin,status,created_at) SELECT 'held' || CAST(value AS TEXT) || '@example.test','example.test','held','import','disabled',1 FROM n")
        .execute(&store.backend).await.unwrap();
    assert_eq!(
        store
            .final_account_gate_snapshot(&[], &[])
            .await
            .unwrap()
            .held_addresses
            .len(),
        5001,
        "do not restrict the global hold scan to requested addresses"
    );
}

#[tokio::test]
async fn policy_section_errors_are_deferred_and_closed_store_fails() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    q("INSERT INTO setting_policy(key,global_value,allow_user_override,global_locked,updated_by,updated_at) VALUES('signin.domainMode','\"open\"',0,0,'test','not a timestamp')")
        .execute(&store.backend).await.unwrap();
    let snapshot = store.final_account_gate_snapshot(&[], &[]).await.unwrap();
    assert!(snapshot.global_policy.is_err());
    for sql in [
        "UPDATE setting_policy SET updated_at=1,allow_user_override='wrong type',global_locked=0 WHERE key='signin.domainMode'",
        "UPDATE setting_policy SET updated_at=1,allow_user_override=0,global_locked='wrong type' WHERE key='signin.domainMode'",
    ] {
        q(sql).execute(&store.backend).await.unwrap();
        let snapshot = store.final_account_gate_snapshot(&[], &[]).await.unwrap();
        assert!(
            snapshot.global_policy.is_err(),
            "do not coerce malformed policy flags into authorization values"
        );
    }
    store.close().await;
    assert!(store.final_account_gate_snapshot(&[], &[]).await.is_err());
}

#[tokio::test]
async fn sqlite_unused_domain_storage_classes_require_legacy_reader_without_parsing_json() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    store
        .put_domain_config(&domain("example.test"))
        .await
        .unwrap();
    let domains = vec!["example.test".into()];
    let mut valid = store
        .final_account_gate_snapshot(&domains, &[])
        .await
        .unwrap();
    assert!(
        !valid
            .domains
            .remove("example.test")
            .unwrap()
            .unwrap()
            .legacy_read_required,
        "invalid JSON text is deliberately unparsed"
    );
    q("UPDATE domains SET upstream_json=X'6162' WHERE name='example.test'")
        .execute(&store.backend)
        .await
        .unwrap();
    let mut invalid = store
        .final_account_gate_snapshot(&domains, &[])
        .await
        .unwrap();
    assert!(
        invalid
            .domains
            .remove("example.test")
            .unwrap()
            .unwrap()
            .legacy_read_required
    );
    store
        .put_domain_config(&domain("example.test"))
        .await
        .unwrap();
    q("UPDATE domains SET preferred='wrong type' WHERE name='example.test'")
        .execute(&store.backend)
        .await
        .unwrap();
    let mut invalid = store
        .final_account_gate_snapshot(&domains, &[])
        .await
        .unwrap();
    assert!(
        invalid
            .domains
            .remove("example.test")
            .unwrap()
            .unwrap()
            .legacy_read_required
    );
    store
        .put_domain_config(&domain("example.test"))
        .await
        .unwrap();
    q("UPDATE domains SET preferred=0,updated_at=NULL WHERE name='example.test'")
        .execute(&store.backend)
        .await
        .unwrap();
    let mut nullable = store
        .final_account_gate_snapshot(&domains, &[])
        .await
        .unwrap();
    assert!(
        !nullable
            .domains
            .remove("example.test")
            .unwrap()
            .unwrap()
            .legacy_read_required,
        "legacy timestamp decoding permits NULL"
    );
    for sql in [
        "UPDATE domains SET upstream_json=CAST(X'ff' AS TEXT) WHERE name='example.test'",
        "UPDATE domains SET upstream_json=CAST(X'6100ff' AS TEXT) WHERE name='example.test'",
        "UPDATE domains SET upstream_json=CAST(X'c1a0' AS TEXT) WHERE name='example.test'",
    ] {
        store
            .put_domain_config(&domain("example.test"))
            .await
            .unwrap();
        q(sql).execute(&store.backend).await.unwrap();
        let mut invalid = store
            .final_account_gate_snapshot(&domains, &[])
            .await
            .unwrap();
        assert!(
            invalid
                .domains
                .remove("example.test")
                .unwrap()
                .unwrap()
                .legacy_read_required,
            "invalid UTF8/NUL must use original decoding"
        );
    }
    for upstream in ["{\"label\":\"câmara\"}".into(), "x".repeat(65537)] {
        let mut row = domain("example.test");
        row.upstream_json = upstream;
        store.put_domain_config(&row).await.unwrap();
        let mut compatible = store
            .final_account_gate_snapshot(&domains, &[])
            .await
            .unwrap();
        assert!(
            compatible
                .domains
                .remove("example.test")
                .unwrap()
                .unwrap()
                .legacy_read_required,
            "valid international/large config falls back instead of changing admission"
        );
    }
}

#[tokio::test]
async fn sqlite_size_guard_uses_column_metadata_and_skips_payload_inspection_when_large() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let root = q("SELECT rootpage FROM sqlite_schema WHERE type='table' AND name='domains'")
        .fetch_scalar_i64(&store.backend)
        .await
        .unwrap();
    let sql: &'static str = Box::leak(format!("EXPLAIN {SQLITE}").into_boxed_str());
    let plan = q(sql)
        .bind("[\"example.test\"]")
        .bind("[]")
        .fetch_all(&store.backend)
        .await
        .unwrap();
    let cursors = plan
        .iter()
        .filter(|row| row.get_string("opcode") == "OpenRead" && row.get_i64("p2") == root)
        .map(|row| row.get_i64("p1"))
        .collect::<Vec<_>>();
    assert!(!cursors.is_empty());
    // Domain config storage columns are upstream_json, allowlist, blocklist,
    // settings_json. SQLCipher's direct-column octet_length sets both flags;
    // wrapping a column in CAST would force payload before the size decision.
    let columns = vec![1, 2, 3, 6];
    let config_columns = plan
        .iter()
        .filter(|row| {
            row.get_string("opcode") == "Column"
                && cursors.contains(&row.get_i64("p1"))
                && columns.contains(&row.get_i64("p2"))
        })
        .collect::<Vec<_>>();
    assert!(
        config_columns
            .iter()
            .filter(|row| row.get_i64("p5") & 0xc0 == 0xc0)
            .count()
            >= 4,
        "byte lengths must use metadata-only Column flags"
    );
    let payload = config_columns
        .iter()
        .filter(|row| row.get_i64("p5") & 0xc0 == 0)
        .map(|row| row.get_i64("addr"))
        .collect::<Vec<_>>();
    let first = *payload.iter().min().unwrap();
    let last = *payload.iter().max().unwrap();
    assert!(
        plan.iter().any(|row| row.get_string("opcode") == "Goto"
            && row.get_i64("addr") < first
            && row.get_i64("p2") > last),
        "sequential CASE must jump over every config payload inspection on the oversized branch"
    );
    let control = q("EXPLAIN SELECT length(CAST(upstream_json AS BLOB)) FROM domains")
        .fetch_all(&store.backend)
        .await
        .unwrap();
    assert!(
        control
            .iter()
            .any(|row| row.get_string("opcode") == "Column"
                && row.get_i64("p2") == 1
                && row.get_i64("p5") & 0xc0 == 0),
        "control must distinguish the unsafe pre-guard materialization form"
    );
}
