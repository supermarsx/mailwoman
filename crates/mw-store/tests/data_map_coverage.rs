//! Coverage gate for the data map. Opens a migrated in-memory SQLite database,
//! lists every table and column, and fails when a table that holds a per-person
//! identifier (a column named for an account, owner, user, subject, address,
//! sender, recipient, principal... or a foreign key to `accounts`) is not
//! mapped in `mw_store::data_map::RULES` to an erasure rule, an export section, a
//! retention control or a documented exemption.

use mw_store::data_map::{self, Export, How, RULES};
use sqlx::{Row, sqlite::SqlitePoolOptions};
use std::collections::{BTreeMap, BTreeSet};

/// Column-name tokens (split on `_`) that mark a per-person identifier.
const PERSON_TOKENS: &[&str] = &[
    "account", "owner", "user", "username", "subject", "email", "recipient", "recipients",
    "sender", "address", "principal", "actor", "login", "delegate", "ip",
];

struct Schema {
    columns: BTreeMap<String, Vec<String>>,
    fk_accounts: BTreeSet<String>,
}

async fn schema() -> Schema {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let tables = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name<>'_sqlx_migrations' ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let mut columns = BTreeMap::new();
    let mut fk_accounts = BTreeSet::new();
    for t in tables {
        let name: String = t.get(0);
        let cols = sqlx::query(&format!("SELECT name FROM pragma_table_info('{name}')"))
            .fetch_all(&pool)
            .await
            .unwrap();
        columns.insert(name.clone(), cols.iter().map(|r| r.get::<String, _>(0)).collect());
        let fks = sqlx::query(&format!("SELECT \"table\" FROM pragma_foreign_key_list('{name}')"))
            .fetch_all(&pool)
            .await
            .unwrap();
        if fks.iter().any(|r| r.get::<String, _>(0) == "accounts") {
            fk_accounts.insert(name);
        }
    }
    Schema { columns, fk_accounts }
}

fn person_columns(cols: &[String]) -> Vec<&String> {
    cols.iter()
        .filter(|c| c.split('_').any(|t| PERSON_TOKENS.contains(&t)))
        .collect()
}

#[tokio::test]
async fn every_table_with_a_person_column_is_mapped() {
    let schema = schema().await;
    let mapped = data_map::mapped_tables();
    let mut missing = Vec::new();
    for (table, cols) in &schema.columns {
        let personal = person_columns(cols);
        let linked = schema.fk_accounts.contains(table);
        if (personal.is_empty() && !linked) || mapped.contains(table.as_str()) {
            continue;
        }
        missing.push(format!("{table} (person columns: {personal:?}, fk to accounts: {linked})"));
    }
    assert!(
        missing.is_empty(),
        "tables with per-person data but no entry in data_map::RULES (map each to an erasure rule, an export section, a retention control or a documented exemption):\n  {}",
        missing.join("\n  ")
    );
}

#[tokio::test]
async fn the_registry_names_only_real_tables_and_columns() {
    let schema = schema().await;
    for rule in RULES {
        let cols = schema
            .columns
            .get(rule.table)
            .unwrap_or_else(|| panic!("data_map names {} which is not in the schema", rule.table));
        // Every identifier the DELETE filters on must exist in the table.
        if let How::Delete(sql) = rule.how {
            assert!(sql.contains("?1"), "{}: DELETE without a bound identifier", rule.table);
            assert!(sql.starts_with("DELETE FROM "), "{}: not a DELETE", rule.table);
        }
        if let Export::Rows { select, cols: out, .. } = rule.export {
            assert!(select.contains("?1"), "{}: export select without an identifier", rule.table);
            assert!(!out.is_empty());
        }
        if let Some(scrub) = rule.scrub {
            for c in scrub.columns.split(',') {
                assert!(
                    cols.iter().any(|x| x == c),
                    "{}: scrub column {c} is not a column",
                    rule.table
                );
            }
        }
        if matches!(rule.how, How::Keep(reason) if reason.trim().is_empty()) {
            panic!("{}: an exemption needs a reason", rule.table);
        }
        if !matches!(rule.how, How::Keep(_)) {
            assert!(rule.count.starts_with("SELECT COUNT(*) FROM "), "{}: needs a count", rule.table);
        }
    }
}

#[tokio::test]
async fn every_registry_statement_runs_on_the_live_schema() {
    // A typo in a column name only shows when the statement is prepared.
    let store = mw_store::Store::open_in_memory(mw_store::ServerKey::from_bytes(&[3u8; 32]).unwrap())
        .await
        .unwrap();
    let p = mw_store::erasure::PersonIds {
        address: "nobody@example.org".into(),
        account_ids: vec!["acct-none".into()],
        owners: vec!["owner-none".into()],
        principals: vec!["principal-none".into()],
    };
    let (_, errors) = store.dsr_purge_person(&p).await;
    assert!(errors.is_empty(), "registry DELETEs failed: {errors:?}");
    for section in data_map::TABLE_SECTIONS {
        store
            .dsr_export_table_section(section, &p)
            .await
            .unwrap_or_else(|e| panic!("export section {section}: {e}"));
    }
    // The count of every rule must also be valid SQL: a failing count must not
    // read as "nothing left".
    let (left, errors) = store.dsr_verify_checked(&p).await;
    assert!(errors.is_empty(), "registry counts failed: {errors:?}");
    assert!(left.is_empty());

}

// ---- the published table ---------------------------------------------------

fn cell(s: &str) -> String {
    s.replace('|', "\\|")
}

fn render_table() -> String {
    let mut out = String::from(
        "| Table | Found by | At erasure | In the export | Limited by or scrubbed |\n|---|---|---|---|---|\n",
    );
    let mut rules: Vec<&data_map::Rule> = RULES.iter().collect();
    rules.sort_by_key(|r| r.table);
    for r in rules {
        let key = match r.count.split_once(" WHERE ") {
            Some((_, w)) => w.replace("?1", "id"),
            None => "-".into(),
        };
        let erase = match r.how {
            How::Delete(_) => "deleted".to_string(),
            How::Cascade => "removed with its parent row".to_string(),
            How::Keep(why) => format!("kept: {why}"),
        };
        let export = match r.export {
            Export::None => "-".to_string(),
            Export::Rows { section, .. } => format!("`{section}`"),
            Export::Marker { section } => format!("`{section}` (marker: present, not exported)"),
            Export::Code { section } => format!("`{section}` (built by code)"),
        };
        let mut limits = Vec::new();
        if !r.retention.is_empty() {
            limits.push(format!("retention: {}", r.retention));
        }
        if let Some(s) = r.scrub {
            limits.push(format!("scrub: {} ({})", s.columns, s.note));
        }
        let limits = if limits.is_empty() { "-".into() } else { limits.join("; ") };
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} |\n",
            r.table,
            if key == "-" { key } else { format!("`{}`", cell(&key)) },
            cell(&erase),
            cell(&export),
            cell(&limits)
        ));
    }
    out
}

#[test]
fn the_published_coverage_table_matches_the_registry() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/admin/erasure-coverage.md");
    let doc = std::fs::read_to_string(&path).unwrap_or_default();
    let (start, end) = ("<!-- registry:start -->", "<!-- registry:end -->");
    let table = render_table();
    let block = format!("{start}\n{table}{end}");
    let current = doc
        .find(start)
        .and_then(|a| doc.find(end).map(|b| doc[a..b + end.len()].replace("\r\n", "\n")));
    if current.as_deref() == Some(block.as_str()) {
        return;
    }
    if std::env::var("BLESS").is_ok_and(|v| v == "1") {
        let new = match (doc.find(start), doc.find(end)) {
            (Some(a), Some(b)) => format!("{}{block}{}", &doc[..a], &doc[b + end.len()..]),
            _ => format!("{doc}\n{block}\n"),
        };
        std::fs::write(&path, new).unwrap();
        return;
    }
    panic!("docs/admin/erasure-coverage.md is out of date with data_map::RULES; run with BLESS=1 to regenerate the table");
}
