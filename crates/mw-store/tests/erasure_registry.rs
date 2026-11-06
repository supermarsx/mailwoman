//! Behaviour of the data map on a real migrated store: every deleting rule is
//! seeded with a row for one person (and a row for a second person), the verify
//! pass must see it, the export must carry it, the purge must remove only the
//! first person's rows, and the verify pass must then find nothing. Also covers
//! the audit hash chain across a scrub, and backup, erase, restore with replay.

use mw_store::{
    AuditRow, ServerKey, Store,
    data_map::{self, Bind, Export, How},
    erasure::PersonIds,
    scrub::ScrubOptions,
};
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};
use std::collections::BTreeSet;

#[path = "../src/test_db.rs"]
mod test_db;

fn key() -> ServerKey {
    ServerKey::from_bytes(&[9u8; 32]).unwrap()
}

async fn raw(path: &str) -> SqlitePool {
    SqlitePool::connect_with(SqliteConnectOptions::new().filename(path).foreign_keys(false))
        .await
        .unwrap()
}

/// Insert one row into `table` with `set` overrides; every other NOT NULL column
/// without a default gets a dummy value of its declared type. Returns false when
/// the schema refuses the row (a CHECK the dummy values cannot meet).
async fn seed(pool: &SqlitePool, table: &str, set: &[(&str, &str)], tag: &str) -> Result<(), String> {
    let cols = sqlx::query(&format!(
        "SELECT name,type,\"notnull\",dflt_value,pk FROM pragma_table_info('{table}')"
    ))
    .fetch_all(pool)
    .await
    .unwrap();
    let mut names = Vec::new();
    let mut vals = Vec::new();
    for c in cols {
        let name: String = c.get("name");
        let ty: String = c.get::<String, _>("type").to_ascii_uppercase();
        let notnull: i64 = c.get("notnull");
        let dflt: Option<String> = c.get("dflt_value");
        let pk: i64 = c.get("pk");
        let hint = HINTS.iter().find(|(t, c, _)| *t == table && *c == name).map(|(_, c, v)| (*c, *v));
        let given = set.iter().find(|(k, _)| *k == name).copied().or(hint);
        if given.is_none() && notnull == 0 && pk == 0 {
            continue;
        }
        if given.is_none() && dflt.is_some() && pk == 0 {
            continue;
        }
        names.push(format!("\"{name}\""));
        vals.push(match given {
            Some((_, v)) => format!("'{}'", v.replace('\'', "''")),
            None if ty.contains("INT") => "1".to_string(),
            None if ty.contains("BLOB") => "x'00'".to_string(),
            None => format!("'{tag}-{name}-{}'", SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)),
        });
    }
    let sql = format!("INSERT INTO \"{table}\" ({}) VALUES ({})", names.join(","), vals.join(","));
    sqlx::query(&sql).execute(pool).await.map(|_| ()).map_err(|e| format!("{table}: {e}"))
}

/// Values a CHECK constraint needs that the generic dummy values do not meet.
const HINTS: &[(&str, &str, &str)] = &[
    ("accounts", "kind", "imap"),
    ("signin_event", "result", "success"),
    ("account_draft", "operation_state", "claimed"),
    ("focused_sender", "tab", "focused"),
    ("mdn_sender_rule", "decision", "always"),
    ("receipt_status", "kind", "mdn"),
    ("account_provider", "provider", "ews"),
    ("nextcloud_pim_binding", "kind", "calendar"),
    ("linked_credential", "state", "active"),
    ("delegation_grant", "scope_kind", "account"),
    ("mail_user_account", "mode", "engine"),
    ("mail_user", "origin", "signin"),
    ("brand_asset", "kind", "logo"),
];
static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// `DELETE FROM t WHERE c=?1` -> (t, c).
fn simple_delete(sql: &str) -> Option<(&str, &str)> {
    let rest = sql.strip_prefix("DELETE FROM ")?;
    let (table, rest) = rest.split_once(" WHERE ")?;
    let col = rest.strip_suffix("=?1")?;
    (!col.contains(' ')).then_some((table, col.trim_matches('"')))
}

fn bound_value(bind: Bind, who: u8) -> String {
    match bind {
        Bind::Account | Bind::Any => format!("acct-{who}"),
        Bind::Owner => format!("owner-{who}"),
        Bind::Address => format!("person{who}@example.org"),
        Bind::Principal => format!("prin-{who}"),
        Bind::None => String::new(),
    }
}

fn person(who: u8) -> PersonIds {
    PersonIds {
        address: format!("person{who}@example.org"),
        account_ids: vec![format!("acct-{who}")],
        owners: vec![format!("owner-{who}")],
        principals: vec![format!("prin-{who}")],
    }
}

#[tokio::test]
async fn erase_removes_exactly_one_persons_rows_and_the_export_carries_them() {
    let path = test_db::unique_file_path("erasure-registry", "store.sqlite");
    let path = path.to_str().unwrap().to_string();
    let store = Store::open(&path, key()).await.unwrap();
    let pool = raw(&path).await;

    // Seed every simply-deletable rule for two people.
    let mut seeded: BTreeSet<&str> = BTreeSet::new();
    let mut refused = Vec::new();
    for rule in data_map::RULES {
        let How::Delete(sql) = rule.how else { continue };
        let Some((table, col)) = simple_delete(sql) else { continue };
        for who in [1u8, 2u8] {
            let v = bound_value(rule.bind, who);
            match seed(&pool, table, &[(col, &v)], &format!("t{who}")).await {
                Ok(()) => {
                    seeded.insert(rule.table);
                }
                Err(e) => refused.push(e),
            }
        }
    }
    // Rows reached through a parent are seeded by hand.
    let hand: &[(&str, &[(&str, &str)])] = &[
        ("calendars", &[("id", "cal-1"), ("account_id", "acct-1")]),
        ("events", &[("id", "ev-1"), ("calendar_id", "cal-1")]),
        ("address_books", &[("id", "ab-1"), ("account_id", "acct-1")]),
        ("contacts", &[("id", "ct-1"), ("address_book_id", "ab-1")]),
        // Rows that reference each other: the delete order must respect the foreign keys.
        ("identity_principal", &[("id", "prin-1")]),
        ("identity_binding", &[("id", "bind-h"), ("principal_id", "prin-1")]),
        ("sessions", &[("id", "sess-h"), ("account_id", "acct-1")]),
        ("session_principal", &[("session_id", "sess-h"), ("principal_id", "prin-1"), ("binding_id", "bind-h")]),
        ("feature_user_subject", &[("subject", "subj-h"), ("account_id", "acct-1"), ("binding_id", "bind-h")]),
    ];
    for (table, set) in hand {
        seed(&pool, table, set, "h").await.unwrap_or_else(|e| refused.push(e));
    }
    eprintln!("seeded {} tables; refused: {refused:#?}", seeded.len());
    assert!(
        seeded.len() > 60,
        "only {} tables could be seeded; refused: {refused:#?}",
        seeded.len()
    );

    let one = person(1);
    let two = person(2);
    let before_one = store.dsr_verify_person(&one).await;
    let before_two = store.dsr_verify_person(&two).await;
    for t in &seeded {
        assert!(before_one.contains_key(*t), "the verify pass does not see seeded rows of {t}");
    }

    // The export carries every table-driven rule that has rows.
    let mut exported: BTreeSet<String> = BTreeSet::new();
    for section in data_map::TABLE_SECTIONS {
        for row in store.dsr_export_table_section(section, &one).await.unwrap() {
            exported.insert(row["table"].as_str().unwrap().to_string());
            assert!(
                row.to_string().find("acct-2").is_none() && !row.to_string().contains("person2@"),
                "the export of person 1 holds person 2's data"
            );
        }
    }
    let mut missing_export = Vec::new();
    for rule in data_map::RULES {
        if matches!(rule.export, Export::Rows { .. } | Export::Marker { .. })
            && seeded.contains(rule.table)
            && !exported.contains(rule.table)
        {
            missing_export.push(rule.table);
        }
    }
    assert!(missing_export.is_empty(), "seeded but not exported: {missing_export:?}");
    // Sealed secrets are markers, never content.
    let security = store.dsr_export_table_section("security", &one).await.unwrap();
    let totp = security.iter().find(|r| r["table"] == "totp_secrets").expect("totp marker");
    assert_eq!(totp["status"], "present, not exported");
    assert!(totp.get("sealedSecret").is_none() && totp.get("sealed_secret").is_none());

    // Erase person 1.
    let (_, errors) = store.dsr_purge_person(&one).await;
    assert!(errors.is_empty(), "{errors:?}");
    let (after_one, check_errors) = store.dsr_verify_checked(&one).await;
    assert!(check_errors.is_empty(), "{check_errors:?}");
    assert!(after_one.is_empty(), "rows remain after erasure: {after_one:?}");
    // Person 2 is untouched.
    assert_eq!(store.dsr_verify_person(&two).await, before_two);
    // The verify pass reads the registry: a table that has only a count (a child
    // reached through a parent) is checked too.
    assert!(data_map::RULES.iter().any(|r| matches!(r.how, How::Cascade) && !r.count.is_empty()));
}

#[tokio::test]
async fn audit_chain_survives_a_scrub_and_the_anchor_maps_old_head_to_new() {
    let path = test_db::unique_file_path("scrub-chain", "store.sqlite");
    let store = Store::open(path.to_str().unwrap(), key()).await.unwrap();
    for (i, (actor, detail)) in [
        ("admin", r#"{"note":"unrelated"}"#),
        ("acct-1", r#"{"who":"person1@example.org"}"#),
        ("admin", r#"{"user":"Person1@Example.org","other":"person2@example.org"}"#),
        ("admin", r#"{"x":1}"#),
    ]
    .iter()
    .enumerate()
    {
        store
            .append_audit(&AuditRow {
                id: format!("a{i}"),
                ts: format!("2026-01-01T00:00:0{i}Z"),
                actor: (*actor).into(),
                actor_kind: if *actor == "admin" { "admin".into() } else { "user".into() },
                action: "test".into(),
                target: Some(format!("account:{}", if i == 1 { "acct-1" } else { "other" })),
                detail_json: (*detail).into(),
                ip: Some("192.0.2.7".into()),
            })
            .await
            .unwrap();
    }
    assert_eq!(store.audit_chain_seal().await.unwrap(), 4);
    let before = store.audit_chain_verify().await.unwrap();
    assert!(before.ok && before.rows == 4 && before.unsealed == 0, "{before:?}");

    let p = person(1);
    let dry = store
        .scrub_person(&p, &[], &[], &ScrubOptions { dry_run: true, ..Default::default() }, None, 100)
        .await
        .unwrap();
    assert_eq!(dry.counts.get("audit_log"), Some(&2));
    assert!(store.audit_chain_verify().await.unwrap().ok, "a dry run changed the chain");
    assert!(store.audit_rechains(10).await.unwrap().is_empty());

    let report = store
        .scrub_person(&p, &[], &[], &ScrubOptions { tombstone: true, ..Default::default() }, Some("req-1"), 200)
        .await
        .unwrap();
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(report.counts.get("audit_log"), Some(&2));
    let after = store.audit_chain_verify().await.unwrap();
    assert!(after.ok, "verification fails after the scrub: {after:?}");
    assert_eq!(after.rows, 4);
    assert_eq!(after.rechains, 1);

    let rows = store.list_audit(10).await.unwrap();
    let text = serde_json::to_string(&rows.iter().map(|r| (&r.actor, &r.target, &r.detail_json, &r.ip)).collect::<Vec<_>>()).unwrap();
    assert!(!text.to_lowercase().contains("person1@example.org"), "{text}");
    assert!(!text.contains("acct-1"), "{text}");
    assert!(text.contains(&report.pseudonym));
    assert!(text.contains("person2@example.org"), "another person's address was changed");
    // The pseudonym is stable: scrubbing again changes nothing.
    let again = store.scrub_person(&p, &[], &[], &ScrubOptions::default(), None, 300).await.unwrap();
    assert_eq!(again.pseudonym, report.pseudonym);
    assert!(again.counts.get("audit_log").is_none());

    let anchors = store.audit_rechains(10).await.unwrap();
    assert_eq!(anchors.len(), 1);
    assert_ne!(anchors[0]["oldHeadHash"], anchors[0]["newHeadHash"]);
    assert_eq!(anchors[0]["rowsChanged"], 2);
    assert!(report.tombstone_seq.is_some());

    // Tampering with a row is caught.
    let pool = raw(path.to_str().unwrap()).await;
    sqlx::query("UPDATE audit_log SET action='tampered' WHERE id='a3'").execute(&pool).await.unwrap();
    assert!(!store.audit_chain_verify().await.unwrap().ok);
}

#[tokio::test]
async fn rotating_the_key_changes_new_pseudonyms_and_a_destroyed_key_cannot_be_used() {
    let path = test_db::unique_file_path("scrub-keys", "store.sqlite");
    let store = Store::open(path.to_str().unwrap(), key()).await.unwrap();
    let (g1, k1) = store.scrub_key_current(10).await.unwrap();
    let g2 = store.scrub_rotate(20, Some(1)).await.unwrap();
    assert_eq!(g2, g1 + 1);
    let (cur, k2) = store.scrub_key_current(21).await.unwrap();
    assert_eq!(cur, g2);
    assert_ne!(k1, k2);
    assert!(store.scrub_key_get(g1).await.unwrap().is_some(), "retired key kept until destroy_after");
    assert_eq!(store.scrub_destroy_expired_keys(20 + 86_400 + 1).await.unwrap(), 1);
    assert!(store.scrub_key_get(g1).await.unwrap().is_none());
    assert!(store.scrub_key_get(g2).await.unwrap().is_some());
}

#[tokio::test]
async fn restore_replays_erasures_made_after_the_backup() {
    let live_path = test_db::unique_file_path("restore-live", "live.sqlite");
    let live_path = live_path.to_str().unwrap().to_string();
    let live = Store::open(&live_path, key()).await.unwrap();
    let pool = raw(&live_path).await;
    let addr = "ana@example.org";
    for (table, set) in [
        ("accounts", vec![("id", "acct-ana")]),
        ("mail_user", vec![("address", addr), ("domain", "example.org"), ("local_part", "ana"), ("status", "active")]),
        ("mail_user_account", vec![("account_id", "acct-ana"), ("address", addr)]),
        ("signin_identity", vec![("account_id", "acct-ana"), ("address", addr)]),
        ("notes", vec![("id", "n1"), ("account_id", "acct-ana")]),
        ("accounts", vec![("id", "acct-bob")]),
        ("mail_user", vec![("address", "bob@example.org"), ("domain", "example.org"), ("local_part", "bob"), ("status", "active")]),
        ("mail_user_account", vec![("account_id", "acct-bob"), ("address", "bob@example.org")]),
        ("notes", vec![("id", "n2"), ("account_id", "acct-bob")]),
    ] {
        seed(&pool, table, &set, "r").await.unwrap_or_else(|e| panic!("{e}"));
    }

    // Back up while Ana is present.
    let backup = test_db::unique_file_path("restore-backup", "backup.sqlite");
    let manifest = live.backup_to_file(&backup, "nightly", 1_000).await.unwrap();
    assert_eq!(manifest["tombstoneWatermark"], 0);
    assert!(std::fs::metadata(format!("{}.manifest.json", backup.display())).is_ok());
    let registered = live.backup_register_list().await.unwrap();
    assert_eq!(registered.len(), 1);
    assert_eq!(registered[0].erased_since, 0);

    // Erase Ana on the live store and record the tombstone.
    let ana = PersonIds {
        address: addr.into(),
        account_ids: vec!["acct-ana".into()],
        owners: vec![],
        principals: vec![],
    };
    let (_, errors) = live.dsr_purge_person(&ana).await;
    assert!(errors.is_empty(), "{errors:?}");
    live.erasure_record(addr, "erasure", Some("req-9"), 2_000).await.unwrap();
    assert!(live.dsr_verify_person(&ana).await.is_empty());
    // The backup is now marked as holding a person who was erased since.
    let registered = live.backup_register_list().await.unwrap();
    assert_eq!(registered[0].erased_since, 1);
    // The tombstone holds no raw identifier.
    let tombs = live.erasure_tombstones_since(0).await.unwrap();
    assert_eq!(tombs.len(), 1);
    assert!(!serde_json::to_string(&tombs).unwrap().contains("ana"));

    // Restore the backup to a new file: Ana is back until the replay runs.
    let restored_path = test_db::unique_file_path("restore-target", "restored.sqlite");
    std::fs::copy(&backup, &restored_path).unwrap();
    let restored = Store::open(restored_path.to_str().unwrap(), key()).await.unwrap();
    assert!(!restored.dsr_verify_person(&ana).await.is_empty(), "the backup should still hold Ana");
    let report = restored
        .erasure_replay(&tombs, &live.scrub_key_rows().await.unwrap(), 3_000)
        .await;
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!((report.tombstones, report.applied, report.absent), (1, 1, 0));
    assert!(restored.dsr_verify_person(&ana).await.is_empty(), "Ana is back after the restore");
    // Bob is untouched, and the tombstone is carried into the restored store.
    let bob = PersonIds {
        address: "bob@example.org".into(),
        account_ids: vec!["acct-bob".into()],
        ..Default::default()
    };
    assert!(!restored.dsr_verify_person(&bob).await.is_empty());
    assert_eq!(restored.erasure_watermark().await.unwrap(), 1);
    // Replaying again is harmless (the closed directory row still matches, so it
    // is applied to nothing).
    let again = restored.erasure_replay(&tombs, &[], 3_100).await;
    assert!(again.errors.is_empty());
    assert_eq!(again.applied + again.absent, 1);
    assert!(restored.dsr_verify_person(&ana).await.is_empty());
}

#[tokio::test]
async fn history_tables_are_pseudonymised_and_other_people_are_left_alone() {
    let path = test_db::unique_file_path("scrub-history", "store.sqlite");
    let path = path.to_str().unwrap().to_string();
    let store = Store::open(&path, key()).await.unwrap();
    let pool = raw(&path).await;
    let ana = "ana@example.org";
    for (table, set) in [
        ("signin_event", vec![("id", "se-ana"), ("username_attempted", ana), ("ip", "192.0.2.9"), ("result", "failure")]),
        ("signin_event", vec![("id", "se-bob"), ("username_attempted", "bob@example.org"), ("ip", "192.0.2.10"), ("result", "failure")]),
        ("assist_audit", vec![("id", "as-ana"), ("actor", ana)]),
        ("assist_audit", vec![("id", "as-bob"), ("actor", "bob@example.org")]),
        ("alert_event", vec![("id", "al-ana"), ("summary", "Sign-in locked for Ana@Example.org after 5 attempts")]),
        ("sso_login_audit", vec![("id", "so-ana"), ("subject_hash", "hash-of-ana-sub")]),
        ("sso_login_audit", vec![("id", "so-bob"), ("subject_hash", "hash-of-bob-sub")]),
        ("breach_delivery", vec![("id", "bd-ana"), ("recipient", ana)]),
    ] {
        seed(&pool, table, &set, "h").await.unwrap_or_else(|e| panic!("{e}"));
    }
    let p = PersonIds { address: ana.into(), ..Default::default() };
    let dry = store
        .scrub_person(&p, &[], &["hash-of-ana-sub".to_string()], &ScrubOptions { dry_run: true, ..Default::default() }, None, 10)
        .await
        .unwrap();
    for t in ["signin_event", "assist_audit", "alert_event", "sso_login_audit", "breach_delivery"] {
        assert_eq!(dry.counts.get(t), Some(&1), "{t}: {:?}", dry.counts);
    }
    let one = |sql: &'static str| {
        let pool = pool.clone();
        async move { sqlx::query_scalar::<_, String>(sql).fetch_one(&pool).await.unwrap() }
    };
    assert_eq!(one("SELECT username_attempted FROM signin_event WHERE id='se-ana'").await, ana, "a dry run changed a row");

    let done = store
        .scrub_person(&p, &[], &["hash-of-ana-sub".to_string()], &ScrubOptions::default(), None, 20)
        .await
        .unwrap();
    assert!(done.errors.is_empty(), "{:?}", done.errors);
    let pseudo = done.pseudonym.clone();
    assert_eq!(one("SELECT username_attempted FROM signin_event WHERE id='se-ana'").await, pseudo);
    assert!(one("SELECT ip FROM signin_event WHERE id='se-ana'").await.starts_with("scrubbed-"));
    assert_eq!(one("SELECT actor FROM assist_audit WHERE id='as-ana'").await, pseudo);
    assert_eq!(one("SELECT summary FROM alert_event WHERE id='al-ana'").await, format!("Sign-in locked for {pseudo} after 5 attempts"));
    assert_eq!(one("SELECT subject_hash FROM sso_login_audit WHERE id='so-ana'").await, pseudo);
    assert_eq!(one("SELECT recipient FROM breach_delivery WHERE id='bd-ana'").await, pseudo);
    // Bob is untouched.
    assert_eq!(one("SELECT username_attempted FROM signin_event WHERE id='se-bob'").await, "bob@example.org");
    assert_eq!(one("SELECT ip FROM signin_event WHERE id='se-bob'").await, "192.0.2.10");
    assert_eq!(one("SELECT actor FROM assist_audit WHERE id='as-bob'").await, "bob@example.org");
    assert_eq!(one("SELECT subject_hash FROM sso_login_audit WHERE id='so-bob'").await, "hash-of-bob-sub");
}
