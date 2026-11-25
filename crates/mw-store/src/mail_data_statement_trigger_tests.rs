// t62-n15: 0129 replaces 0113's row-level messages triggers (PostgreSQL) with
// statement-level ones. (1) The validator requires exactly the 0129 set.
// (2) A randomised differential test runs the same statements against a schema
// with 0129 and one with the 0113 triggers restored and compares the derived
// tables after every step. Revisions are random on both sides, so a step is
// compared by which rows exist, their account, whether each revision changed
// in that step, and whether a row's cache/content revision equals its message's.
use super::*;
use crate::ServerKey;
use sqlx::{Connection, PgConnection, Row};
use std::collections::BTreeMap;

const OLD_0113: &str = include_str!("../migrations_pg/0113_mail_data_publication.sql");
const NEW_0129: &str = include_str!("../migrations_pg/0129_mail_data_statement_triggers.sql");
const NEW_0131: &str = include_str!("../migrations_pg/0131_mail_data_seq_lock_order.sql");
const NEW_0133: &str = include_str!("../migrations_pg/0133_mail_data_single_row.sql");
const SEQ_TABLES: [&str; 4] = [
    "changes",
    "pim_changes",
    "crypto_changes",
    "change_log_checkpoint",
];
const REPLACED: [&str; 4] = [
    "mail_data_message_insert",
    "mail_data_message_update",
    "mail_data_message_delete",
    "mail_data_stamp_update",
];
const ADDED: [&str; 4] = [
    "mail_data_messages_inserted",
    "mail_data_message_rekey",
    "mail_data_messages_updated",
    "mail_data_messages_deleted",
];

fn between<'a>(sql: &'a str, start: &str, end: &str) -> &'a str {
    let a = sql
        .find(start)
        .unwrap_or_else(|| panic!("{start} not in migration"));
    let b = sql[a..].find(end).unwrap() + a + end.len();
    &sql[a..b]
}

/// SQL that turns a schema at 0131 back into the 0113 messages triggers, with
/// 0131's one textual change to them (the ensure advances `seq`).
fn restore_0113(schema: &str) -> String {
    let mut sql = String::new();
    for name in ADDED {
        sql += &format!("DROP TRIGGER {name} ON messages;DROP FUNCTION {name}_fn();\n");
    }
    for name in REPLACED {
        sql += &between(OLD_0113, &format!("CREATE FUNCTION {name}_fn()"), "END $$;").replace(
            "ON CONFLICT(account_id) DO NOTHING",
            "ON CONFLICT(account_id) DO UPDATE SET seq=mail_data_accounts.seq+1",
        );
        sql += "\n";
        sql += between(OLD_0113, &format!("CREATE TRIGGER {name} "), ";");
        sql +=
            &format!("\nALTER FUNCTION {name}_fn() SET search_path = pg_catalog, \"{schema}\";\n");
    }
    sql
}

#[test]
fn required_set_is_0113_with_0129_replacements() {
    let pg = triggers(true);
    let names = pg.iter().map(|t| t.name.as_str()).collect::<Vec<_>>();
    for name in REPLACED {
        assert!(!names.contains(&name), "{name} was dropped by 0129");
    }
    let find = |name: &str| {
        pg.iter()
            .find(|t| t.name == name)
            .unwrap_or_else(|| panic!("{name} required"))
    };
    let inserted = find("mail_data_messages_inserted");
    assert_eq!(
        (
            inserted.kind,
            inserted.old_table.as_str(),
            inserted.new_table.as_str()
        ),
        (4, "", "mail_data_new")
    );
    let updated = find("mail_data_messages_updated");
    assert_eq!(
        (
            updated.kind,
            updated.old_table.as_str(),
            updated.new_table.as_str()
        ),
        (16, "mail_data_old", "mail_data_new")
    );
    let deleted = find("mail_data_messages_deleted");
    assert_eq!(
        (
            deleted.kind,
            deleted.old_table.as_str(),
            deleted.new_table.as_str()
        ),
        (8, "mail_data_old", "")
    );
    let rekey = find("mail_data_message_rekey");
    assert_eq!(
        (rekey.kind, rekey.when.as_str()),
        (17, "old.stable_idisdistinctfromnew.stable_id")
    );
    // pg_get_triggerdef prints the condition in its own form.
    assert_eq!(
        when_of(
            "CREATE TRIGGER x AFTER UPDATE ON s.messages FOR EACH ROW WHEN ((old.stable_id IS DISTINCT FROM new.stable_id)) EXECUTE FUNCTION mail_data_message_rekey_fn()"
        ),
        rekey.when
    );
    assert!(
        pg.iter()
            .filter(|t| !ADDED.contains(&t.name.as_str()) && !t.name.starts_with("mail_data_seq_"))
            .all(|t| t.kind & 1 == 1
                && t.when.is_empty()
                && t.old_table.is_empty()
                && t.new_table.is_empty())
    );
    assert!(
        pg.iter().any(|t| t.name == "mail_data_message_meta_insert"),
        "0113's other triggers stay required"
    );
    let fns = functions();
    for name in REPLACED {
        assert!(!fns.iter().any(|f| f.name == format!("{name}_fn")));
    }
    for name in ADDED {
        assert!(fns.iter().any(|f| f.name == format!("{name}_fn")));
    }
    // SQLite keeps its 0113 triggers (its 0129 is a placeholder).
    let lite = triggers(false);
    for name in REPLACED {
        assert!(lite.iter().any(|t| t.name == name));
    }
    // 0131: every ensure advances seq (both dialects); 0133 removed the change-log
    // seq triggers again (above the cost gate).
    for (_pg, set) in [(true, &pg), (false, &lite)] {
        for table in SEQ_TABLES {
            assert!(
                set.iter().all(|t| t.table != table),
                "no seq trigger on {table} after 0133"
            );
        }
        assert!(
            set.iter()
                .all(|t| !t.sql.contains("ON CONFLICT(account_id) DO NOTHING")),
            "no ensure left that locks nothing"
        );
    }
    assert!(
        functions()
            .iter()
            .all(|f| !f.body.contains("ON CONFLICT(account_id) DO NOTHING"))
    );
    // 0132: column lists keep two triggers from firing on a flag change.
    assert_eq!(
        find("mail_data_mailbox_update").columns,
        "role,account_id,id"
    );
    assert_eq!(
        find("verdict_content_revision").columns,
        "account_id,stable_id,blob_ref"
    );
}

struct Pg {
    dsn: String,
    admin: PgConnection,
    schemas: Vec<String>,
}
impl Pg {
    async fn new() -> Option<Self> {
        let Ok(dsn) = std::env::var("MW_TEST_PG") else {
            mw_test_gate::skip(
                "MW_TEST_PG absent; 0129 statement-trigger tests not executed".to_owned(),
            );
            return None;
        };
        let admin = PgConnection::connect(&dsn).await.unwrap();
        Some(Self {
            dsn,
            admin,
            schemas: Vec::new(),
        })
    }
    fn url(&self, schema: &str) -> String {
        let mut url = url::Url::parse(&self.dsn).unwrap();
        url.query_pairs_mut()
            .append_pair("options", &format!("-csearch_path={schema}"));
        url.to_string()
    }
    /// A fully migrated schema; the store is returned open.
    async fn schema(&mut self, prefix: &str) -> (String, Store) {
        let schema = format!("{prefix}_{}", uuid::Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&mut self.admin)
            .await
            .unwrap();
        self.schemas.push(schema.clone());
        let store = Store::open(&self.url(&schema), ServerKey::generate())
            .await
            .unwrap();
        (schema, store)
    }
    async fn connect(&self, schema: &str) -> PgConnection {
        PgConnection::connect(&self.url(schema)).await.unwrap()
    }
    async fn cleanup(mut self) {
        for schema in std::mem::take(&mut self.schemas) {
            sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
                .execute(&mut self.admin)
                .await
                .unwrap();
        }
    }
}

async fn raw(conn: &mut PgConnection, sql: &str) -> Result<(), String> {
    sqlx::raw_sql(sql)
        .execute(&mut *conn)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tokio::test]
async fn postgres_validator_requires_0129_and_restamps_once_at_upgrade() {
    let Some(mut pg) = Pg::new().await else {
        return;
    };
    let (schema, store) = pg.schema("n15_validator").await;
    let mut conn = pg.connect(&schema).await;
    raw(&mut conn, "INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES('va','imap','h',993,'tls','va','\\x00')").await.unwrap();
    let descriptor = store.mail_data_descriptor("va", &[]).unwrap();
    let snapshot = async || {
        store
            .mail_data_snapshot(&descriptor)
            .await
            .unwrap()
            .unwrap()
    };
    let fresh = snapshot().await;
    assert!(
        store.validate_mail_data_schema(&fresh).await.unwrap(),
        "a fresh install at 0129 qualifies"
    );

    // The schema as it was before the upgrade (0113 triggers): this binary's
    // validator refuses it, and its stamp is the one older fences carry.
    raw(&mut conn, &restore_0113(&schema)).await.unwrap();
    let before_upgrade = snapshot().await;
    assert!(
        !store
            .validate_mail_data_schema(&before_upgrade)
            .await
            .unwrap(),
        "0113's row triggers are not the required set any more"
    );
    // The upgrade itself: 0129, then the function definitions of 0131 and 0133
    // (their table and trigger parts are already in place in this schema).
    raw(
        &mut conn,
        &format!(
            "BEGIN;{NEW_0129};{};{};COMMIT",
            replay_functions(NEW_0131, &schema),
            replay_functions(NEW_0133, &schema)
        ),
    )
    .await
    .unwrap();
    assert!(
        !store
            .validate_mail_data_schema(&before_upgrade)
            .await
            .unwrap(),
        "a fence from before the upgrade fails once (stamp moved)"
    );
    let after_upgrade = snapshot().await;
    assert!(!before_upgrade.same_schema_generation(&after_upgrade));
    assert!(
        store
            .validate_mail_data_schema(&after_upgrade)
            .await
            .unwrap(),
        "the re-stamped fence qualifies after the upgrade"
    );
    assert!(
        store
            .validate_mail_data_schema(&snapshot().await)
            .await
            .unwrap()
    );

    // Each property of the new set is required.
    let rekey = between(NEW_0129, "CREATE TRIGGER mail_data_message_rekey ", ";").to_owned();
    let updated = between(NEW_0129, "CREATE TRIGGER mail_data_messages_updated ", ";").to_owned();
    let inserted = between(NEW_0129, "CREATE TRIGGER mail_data_messages_inserted ", ";").to_owned();
    let cases: Vec<(&str, String, String)> = vec![
        ("statement trigger disabled", "ALTER TABLE messages DISABLE TRIGGER mail_data_messages_inserted".into(), "ALTER TABLE messages ENABLE TRIGGER mail_data_messages_inserted".into()),
        ("statement trigger missing", "DROP TRIGGER mail_data_messages_deleted ON messages".into(), between(NEW_0129, "CREATE TRIGGER mail_data_messages_deleted ", ";").into()),
        ("rekey trigger without WHEN", format!("DROP TRIGGER mail_data_message_rekey ON messages;{}", rekey.replace(" WHEN (OLD.stable_id IS DISTINCT FROM NEW.stable_id)", "")), format!("DROP TRIGGER mail_data_message_rekey ON messages;{rekey}")),
        ("rekey trigger with another WHEN", format!("DROP TRIGGER mail_data_message_rekey ON messages;{}", rekey.replace("OLD.stable_id IS DISTINCT FROM NEW.stable_id", "OLD.uid IS DISTINCT FROM NEW.uid")), format!("DROP TRIGGER mail_data_message_rekey ON messages;{rekey}")),
        ("update trigger per row", format!("DROP TRIGGER mail_data_messages_updated ON messages;{}", updated.replace("REFERENCING OLD TABLE AS mail_data_old NEW TABLE AS mail_data_new FOR EACH STATEMENT", "FOR EACH ROW")), format!("DROP TRIGGER mail_data_messages_updated ON messages;{updated}")),
        ("insert trigger with other transition name", format!("DROP TRIGGER mail_data_messages_inserted ON messages;{}", inserted.replace("AS mail_data_new", "AS other_new")), format!("DROP TRIGGER mail_data_messages_inserted ON messages;{inserted}")),
        ("statement function body changed", "CREATE OR REPLACE FUNCTION mail_data_messages_deleted_fn() RETURNS TRIGGER LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END $$".into(), format!("CREATE OR REPLACE {};ALTER FUNCTION mail_data_messages_deleted_fn() SET search_path = pg_catalog, \"{schema}\"", between(NEW_0133, "FUNCTION mail_data_messages_deleted_fn()", "END $$;"))),
        ("0113 function without the seq bump", format!("CREATE OR REPLACE {};ALTER FUNCTION mail_data_mailbox_insert_fn() SET search_path = pg_catalog, \"{schema}\"", between(OLD_0113, "FUNCTION mail_data_mailbox_insert_fn()", "END $$;")), format!("CREATE OR REPLACE {};ALTER FUNCTION mail_data_mailbox_insert_fn() SET search_path = pg_catalog, \"{schema}\"", between(NEW_0131, "FUNCTION mail_data_mailbox_insert_fn()", "END $$;"))),
    ];
    for (what, break_sql, repair_sql) in cases {
        raw(&mut conn, &break_sql)
            .await
            .unwrap_or_else(|e| panic!("{what}: {e}"));
        assert!(
            !store
                .validate_mail_data_schema(&snapshot().await)
                .await
                .unwrap(),
            "{what} must not qualify"
        );
        raw(&mut conn, &repair_sql)
            .await
            .unwrap_or_else(|e| panic!("{what} repair: {e}"));
        assert!(
            store
                .validate_mail_data_schema(&snapshot().await)
                .await
                .unwrap(),
            "{what} repaired qualifies again"
        );
    }
    store.close().await;
    drop(conn);
    pg.cleanup().await;
}

/// The CREATE OR REPLACE FUNCTION blocks of a migration, each followed by its
/// search_path pin (the migration's own DO block also names functions a later
/// migration drops).
fn replay_functions(migration: &str, schema: &str) -> String {
    let mut sql = String::new();
    let mut rest = migration;
    while let Some(at) = rest.find("CREATE OR REPLACE FUNCTION ") {
        let block = between(&rest[at..], "CREATE OR REPLACE FUNCTION ", "END $$;");
        let name = block["CREATE OR REPLACE FUNCTION ".len()..]
            .split('(')
            .next()
            .unwrap();
        sql += block;
        sql += &format!(
            "
ALTER FUNCTION {name}() SET search_path = pg_catalog, \"{schema}\";
"
        );
        rest = &rest[at + block.len()..];
    }
    sql
}

// ---- differential test ----

#[derive(Default, Clone, PartialEq, Debug)]
struct Snap {
    messages: BTreeMap<String, (String, String, String, String, String)>,
    accounts: BTreeMap<String, (String, String, String, i64)>,
    rows: BTreeMap<String, (String, String, String, String)>,
    outbox: BTreeMap<String, String>,
}
async fn snap(conn: &mut PgConnection) -> Snap {
    let mut s = Snap::default();
    for r in sqlx::query("SELECT stable_id,account_id,mailbox_id,flags_json,cache_revision,content_revision FROM messages").fetch_all(&mut *conn).await.unwrap() {
        s.messages.insert(r.get(0), (r.get(1), r.get(2), r.get(3), r.get(4), r.get(5)));
    }
    for r in sqlx::query(
        "SELECT account_id,incarnation,focus_revision,policy_revision,seq FROM mail_data_accounts",
    )
    .fetch_all(&mut *conn)
    .await
    .unwrap()
    {
        s.accounts
            .insert(r.get(0), (r.get(1), r.get(2), r.get(3), r.get(4)));
    }
    for r in sqlx::query("SELECT stable_id,account_id,data_revision,cache_revision,content_revision FROM mail_data_rows").fetch_all(&mut *conn).await.unwrap() {
        s.rows.insert(r.get(0), (r.get(1), r.get(2), r.get(3), r.get(4)));
    }
    for r in sqlx::query("SELECT account_id,revision FROM mail_data_outbox")
        .fetch_all(&mut *conn)
        .await
        .unwrap()
    {
        s.outbox.insert(r.get(0), r.get(1));
    }
    s
}
/// What one step did, without the random values themselves.
#[derive(PartialEq, Debug)]
struct Seen {
    messages: BTreeMap<String, (String, String, String)>,
    accounts: BTreeMap<String, (bool, bool, bool, bool, bool)>,
    rows: BTreeMap<String, (String, bool, bool, bool, bool)>,
    outbox: BTreeMap<String, (bool, bool)>,
}
fn seen(before: &Snap, after: &Snap) -> Seen {
    Seen {
        messages: after
            .messages
            .iter()
            .map(|(k, v)| (k.clone(), (v.0.clone(), v.1.clone(), v.2.clone())))
            .collect(),
        accounts: after
            .accounts
            .iter()
            .map(|(k, v)| {
                let old = before.accounts.get(k);
                (
                    k.clone(),
                    (
                        old.is_none(),
                        old.is_some_and(|o| o.0 != v.0),
                        old.is_some_and(|o| o.1 != v.1),
                        old.is_some_and(|o| o.2 != v.2),
                        old.is_some_and(|o| o.3 != v.3),
                    ),
                )
            })
            .collect(),
        rows: after
            .rows
            .iter()
            .map(|(k, v)| {
                let old = before.rows.get(k);
                let message = after.messages.get(k);
                (
                    k.clone(),
                    (
                        v.0.clone(),
                        old.is_none(),
                        old.is_some_and(|o| o.1 != v.1),
                        message.is_some_and(|m| m.3 == v.2),
                        message.is_some_and(|m| m.4 == v.3),
                    ),
                )
            })
            .collect(),
        outbox: after
            .outbox
            .iter()
            .map(|(k, v)| {
                let old = before.outbox.get(k);
                (k.clone(), (old.is_none(), old.is_some_and(|o| o != v)))
            })
            .collect(),
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}

const ACCOUNTS: [&str; 3] = ["a0", "a1", "a2"];
const BOXES: [&str; 4] = ["inbox", "sent", "trash", "arch"];
const FLAGS: [&str; 4] = [
    "[]",
    "[\"Seen\"]",
    "[\"Flagged\"]",
    "[\"Seen\",{\"Keyword\":\"x\"}]",
];

fn account_sql(account: &str) -> String {
    let mut sql = format!(
        "INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES('{account}','imap','h',993,'tls','{account}','\\x00');"
    );
    for (i, b) in BOXES.iter().enumerate() {
        let role = match *b {
            "arch" => "NULL".to_owned(),
            r => format!("'{r}'"),
        };
        sql += &format!(
            "INSERT INTO mailboxes(id,account_id,name,role,uidvalidity) VALUES('{account}_{b}','{account}','{b}',{role},{i});"
        );
    }
    sql
}

enum Op {
    Sql(String),
    Copy(String),
    Tx {
        ops: Vec<Op>,
        commit: bool,
        deferred: bool,
    },
}

struct Gen {
    rng: Rng,
    counter: u64,
}
impl Gen {
    fn fresh(&mut self, prefix: &str) -> String {
        self.counter += 1;
        format!("{prefix}{}", self.counter)
    }
    fn mailbox(&mut self, account: &str) -> String {
        let owner = if self.rng.chance(80) {
            account.to_owned()
        } else {
            (*self.rng.pick(&ACCOUNTS)).to_owned()
        };
        format!("{owner}_{}", self.rng.pick(&BOXES))
    }
    fn subset(&mut self, ids: &[String]) -> Vec<String> {
        if ids.is_empty() {
            return vec!["none".into()];
        }
        let size = *self.rng.pick(&[1usize, 1, 1, 2, 3, 5, 9, 12, 40, 1000]);
        let mut out = Vec::new();
        for _ in 0..size.min(ids.len()) {
            out.push(self.rng.pick(ids).clone());
        }
        out.sort();
        out.dedup();
        out
    }
    fn list(ids: &[String]) -> String {
        ids.iter()
            .map(|i| format!("'{i}'"))
            .collect::<Vec<_>>()
            .join(",")
    }
    fn values(&mut self, n: usize) -> Vec<String> {
        (0..n)
            .map(|_| {
                let id = self.fresh("m");
                let account = *self.rng.pick(&ACCOUNTS);
                let mailbox = self.mailbox(account);
                let flags = *self.rng.pick(&FLAGS);
                format!(
                    "('{id}','{account}','{mailbox}',{},1,'{flags}')",
                    self.counter
                )
            })
            .collect()
    }
    fn op(&mut self, ids: &[String], nested: bool) -> Op {
        let r = &mut self.rng;
        let kind = r.below(if nested { 20 } else { 23 });
        let set = self.subset(ids);
        let set = Self::list(&set);
        let sql = match kind {
            0 | 1 => {
                let n = *self.rng.pick(&[1usize, 1, 2, 3, 9, 30]);
                format!(
                    "INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,flags_json) VALUES {}",
                    self.values(n).join(",")
                )
            }
            2 => {
                let fresh = self.rng.below(3);
                let mut rows = self.values(fresh);
                for id in self.subset(ids) {
                    let account = *self.rng.pick(&ACCOUNTS);
                    let mailbox = self.mailbox(account);
                    self.counter += 1;
                    rows.push(format!(
                        "('{id}','{account}','{mailbox}',{},1,'{}')",
                        self.counter,
                        self.rng.pick(&FLAGS)
                    ));
                }
                if rows.is_empty() {
                    return Op::Sql("SELECT 1".into());
                }
                let action = *self.rng.pick(&[
                    "DO UPDATE SET flags_json=excluded.flags_json",
                    "DO UPDATE SET flags_json=excluded.flags_json,mailbox_id=excluded.mailbox_id",
                    "DO UPDATE SET account_id=excluded.account_id",
                    "DO UPDATE SET flags_json=messages.flags_json",
                    "DO NOTHING",
                ]);
                format!(
                    "INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,flags_json) VALUES {} ON CONFLICT(stable_id) {action}",
                    rows.join(",")
                )
            }
            3 | 4 => format!(
                "UPDATE messages SET flags_json='{}' WHERE stable_id IN({set})",
                self.rng.pick(&FLAGS)
            ),
            5 => format!(
                "UPDATE messages SET flags_json='{}' WHERE account_id='{}'",
                self.rng.pick(&FLAGS),
                self.rng.pick(&ACCOUNTS)
            ),
            6 | 7 => {
                let account = *self.rng.pick(&ACCOUNTS);
                let target = self.mailbox(account);
                format!("UPDATE messages SET mailbox_id='{target}' WHERE stable_id IN({set})")
            }
            8 => format!(
                "UPDATE messages SET account_id='{}' WHERE stable_id IN({set})",
                self.rng.pick(&ACCOUNTS)
            ),
            9 => {
                let c = self.fresh("rev");
                let column = *self.rng.pick(&["content_revision", "cache_revision"]);
                format!("UPDATE messages SET {column}='{c}' WHERE stable_id IN({set})")
            }
            10 => format!(
                "UPDATE messages SET {} WHERE stable_id IN({set})",
                self.rng.pick(&[
                    "flags_json=flags_json",
                    "uid=uid",
                    "size=size+1",
                    "thread_id='t1'",
                    "internaldate='2030-01-01T00:00:00Z'",
                    "envelope_json='\\x7b7d'",
                    "blob_ref=NULL",
                    "blob_ref='b1'",
                    "message_id='<x@y>'",
                    "uidvalidity=uidvalidity+1"
                ])
            ),
            11 => {
                let Some(id) = ids.first().map(|_| self.rng.pick(ids).clone()) else {
                    return Op::Sql("SELECT 1".into());
                };
                let fresh = self.fresh("r");
                format!("UPDATE messages SET stable_id='{fresh}' WHERE stable_id='{id}'")
            }
            12 => {
                // A->B, B->C in one statement (fails when A is processed first),
                // or a rename together with ordinary changes of other rows.
                if ids.len() < 2 {
                    return Op::Sql("SELECT 1".into());
                }
                let a = self.rng.pick(ids).clone();
                let b = self.rng.pick(ids).clone();
                let c = self.fresh("r");
                if self.rng.chance(50) && a != b {
                    // B is scanned first (primary-key order), so B->C precedes A->B
                    // and the chain succeeds on both sides; plan forced so the
                    // order does not depend on each schema's heap layout.
                    let (a, b) = if b < a { (a, b) } else { (b, a) };
                    format!(
                        "SET LOCAL enable_seqscan=off;SET LOCAL enable_bitmapscan=off;UPDATE messages SET stable_id=CASE stable_id WHEN '{b}' THEN '{c}' ELSE '{b}' END WHERE stable_id IN('{a}','{b}')"
                    )
                } else {
                    let flags = *self.rng.pick(&FLAGS);
                    let sent = format!("{}_sent", self.rng.pick(&ACCOUNTS));
                    format!(
                        "UPDATE messages SET stable_id=CASE WHEN stable_id='{a}' THEN '{c}' ELSE stable_id END,flags_json='{flags}',mailbox_id=CASE WHEN stable_id='{b}' THEN '{sent}' ELSE mailbox_id END WHERE stable_id IN({set},'{a}')"
                    )
                }
            }
            13 | 14 => format!("DELETE FROM messages WHERE stable_id IN({set})"),
            15 => {
                let account = *self.rng.pick(&ACCOUNTS);
                format!(
                    "DELETE FROM messages WHERE mailbox_id='{}'",
                    self.mailbox(account)
                )
            }
            16 => {
                let account = *self.rng.pick(&ACCOUNTS);
                let role = if self.rng.chance(50) {
                    "'sent'"
                } else {
                    "NULL"
                };
                format!("UPDATE mailboxes SET role={role} WHERE id='{account}_arch'")
            }
            17 => {
                let fresh = self.fresh("w");
                format!(
                    "WITH d AS (DELETE FROM messages WHERE stable_id IN({set}) RETURNING *) INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,flags_json) SELECT stable_id||'_{fresh}',account_id,mailbox_id,uid+1000000,uidvalidity,flags_json FROM d"
                )
            }
            18 => {
                let other = Self::list(&self.subset(ids));
                format!(
                    "WITH u AS (UPDATE messages SET flags_json='[\"Seen\"]' WHERE stable_id IN({set}) RETURNING 1) UPDATE messages SET mailbox_id='a0_sent' WHERE stable_id IN({other}) AND stable_id NOT IN({set})"
                )
            }
            19 => {
                let n = *self.rng.pick(&[1usize, 3, 12]);
                let lines = (0..n)
                    .map(|_| {
                        let id = self.fresh("c");
                        let account = *self.rng.pick(&ACCOUNTS);
                        let mailbox = self.mailbox(account);
                        format!("{id}\t{account}\t{mailbox}\t{}\t1\t[]\n", self.counter)
                    })
                    .collect::<String>();
                return Op::Copy(lines);
            }
            20 => {
                let account = *self.rng.pick(&ACCOUNTS);
                let b = *self.rng.pick(&BOXES);
                let role = match b {
                    "arch" => "NULL".to_owned(),
                    r => format!("'{r}'"),
                };
                format!(
                    "DELETE FROM mailboxes WHERE id='{account}_{b}';INSERT INTO mailboxes(id,account_id,name,role,uidvalidity) VALUES('{account}_{b}','{account}','{b}',{role},7)"
                )
            }
            21 => {
                let account = *self.rng.pick(&ACCOUNTS);
                format!(
                    "DELETE FROM accounts WHERE id='{account}';{}",
                    account_sql(account)
                )
            }
            _ => {
                let n = 2 + self.rng.below(4);
                let ops = (0..n).map(|_| self.op(ids, true)).collect();
                return Op::Tx {
                    ops,
                    commit: self.rng.chance(80),
                    deferred: self.rng.chance(30),
                };
            }
        };
        Op::Sql(sql)
    }
}

/// Runs `op` on one side; returns the outcome of every statement.
async fn run(conn: &mut PgConnection, op: &Op, out: &mut Vec<bool>) {
    match op {
        Op::Sql(sql) => {
            let result = raw(conn, sql).await;
            if let Err(e) = &result {
                eprintln!("statement failed: {e}: {}", &sql[..sql.len().min(160)]);
            }
            out.push(result.is_ok())
        }
        Op::Copy(lines) => {
            let result = async {
                let mut copy = conn.copy_in_raw("COPY messages(stable_id,account_id,mailbox_id,uid,uidvalidity,flags_json) FROM STDIN").await?;
                copy.send(lines.as_bytes()).await?;
                copy.finish().await
            }.await;
            out.push(result.is_ok());
        }
        Op::Tx {
            ops,
            commit,
            deferred,
        } => {
            raw(conn, "BEGIN").await.unwrap();
            if *deferred {
                raw(conn, "SET CONSTRAINTS ALL DEFERRED").await.unwrap();
            }
            for op in ops {
                raw(conn, "SAVEPOINT step").await.unwrap();
                let at = out.len();
                Box::pin(run(conn, op, out)).await;
                if out[at..].iter().all(|ok| *ok) {
                    raw(conn, "RELEASE SAVEPOINT step").await.unwrap();
                } else {
                    raw(conn, "ROLLBACK TO SAVEPOINT step").await.unwrap();
                }
            }
            out.push(
                raw(conn, if *commit { "COMMIT" } else { "ROLLBACK" })
                    .await
                    .is_ok(),
            );
        }
    }
}

fn describe(op: &Op) -> String {
    match op {
        Op::Sql(sql) => sql.clone(),
        Op::Copy(lines) => format!("COPY {} rows", lines.lines().count()),
        Op::Tx {
            ops,
            commit,
            deferred,
        } => format!(
            "BEGIN{};{};{}",
            if *deferred {
                ";SET CONSTRAINTS ALL DEFERRED"
            } else {
                ""
            },
            ops.iter().map(describe).collect::<Vec<_>>().join(";"),
            if *commit { "COMMIT" } else { "ROLLBACK" }
        ),
    }
}

#[tokio::test]
async fn postgres_0129_statement_triggers_equal_0113_row_triggers() {
    let Some(mut pg) = Pg::new().await else {
        return;
    };
    let steps: usize = std::env::var("MW_N15_STEPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(160);
    for seed in [
        0x9e37_79b9_7f4a_7c15u64,
        0x2545_f491_4f6c_dd1d,
        0x0123_4567_89ab_cdef,
    ] {
        let (new_schema, new_store) = pg.schema("n15_new").await;
        let (old_schema, old_store) = pg.schema("n15_old").await;
        new_store.close().await;
        old_store.close().await;
        let mut new = pg.connect(&new_schema).await;
        let mut old = pg.connect(&old_schema).await;
        raw(&mut old, &restore_0113(&old_schema)).await.unwrap();
        let setup = ACCOUNTS.iter().map(|a| account_sql(a)).collect::<String>();
        raw(&mut new, &setup).await.unwrap();
        raw(&mut old, &setup).await.unwrap();
        let mut generator = Gen {
            rng: Rng(seed),
            counter: 0,
        };
        let (mut new_before, mut old_before) = (snap(&mut new).await, snap(&mut old).await);
        let mut renamed_once = false;
        let mut peak = 0usize;
        for step in 0..steps {
            let ids = new_before.messages.keys().cloned().collect::<Vec<_>>();
            let op = generator.op(&ids, false);
            let (mut new_ok, mut old_ok) = (Vec::new(), Vec::new());
            run(&mut new, &op, &mut new_ok).await;
            run(&mut old, &op, &mut old_ok).await;
            let text = describe(&op);
            assert_eq!(
                new_ok, old_ok,
                "seed {seed:x} step {step}: outcomes differ for {text}"
            );
            renamed_once |= text.contains("SET stable_id") && new_ok.iter().all(|ok| *ok);
            let (new_after, old_after) = (snap(&mut new).await, snap(&mut old).await);
            assert_eq!(
                seen(&new_before, &new_after),
                seen(&old_before, &old_after),
                "seed {seed:x} step {step}: derived tables differ after {text}"
            );
            let leftover: Option<String> = sqlx::query_scalar(
                "SELECT NULLIF(current_setting('mw_mail_data.rekeyed_1',true),'')",
            )
            .fetch_one(&mut new)
            .await
            .unwrap();
            assert_eq!(
                leftover, None,
                "seed {seed:x} step {step}: rekey list not cleared after {text}"
            );
            peak = peak.max(new_after.messages.len());
            (new_before, old_before) = (new_after, old_after);
        }
        // Every column of 0113's update gate, and columns outside it, once on
        // one row (plain-join form) and once on all rows (sorted form).
        let sets = [
            "content_revision=content_revision||'x'",
            "envelope_json='\x01'",
            "account_id='a1'",
            "blob_ref='b2'",
            "mailbox_id='a2_sent'",
            "uid=uid+5000000",
            "uidvalidity=uidvalidity+1",
            "message_id='<z@y>'",
            "thread_id='t2'",
            "internaldate='2031-01-01T00:00:00Z'",
            "size=size+1",
            "flags_json='[\"Flagged\"]'",
            "cache_revision='sweep'",
            "flags_json=flags_json",
        ];
        for set in sets {
            for scope in ["stable_id=(SELECT min(stable_id) FROM messages)", "true"] {
                let sql = format!("UPDATE messages SET {set} WHERE {scope}");
                let (mut new_ok, mut old_ok) = (Vec::new(), Vec::new());
                run(&mut new, &Op::Sql(sql.clone()), &mut new_ok).await;
                run(&mut old, &Op::Sql(sql.clone()), &mut old_ok).await;
                assert_eq!((new_ok.clone(), old_ok), (vec![true], vec![true]), "{sql}");
                let (new_after, old_after) = (snap(&mut new).await, snap(&mut old).await);
                assert_eq!(
                    seen(&new_before, &new_after),
                    seen(&old_before, &old_after),
                    "seed {seed:x}: derived tables differ after {sql}"
                );
                (new_before, old_before) = (new_after, old_after);
            }
        }
        assert!(
            renamed_once,
            "the sequence must contain a successful stable_id rename"
        );
        assert!(
            peak > 20,
            "the sequence must hold enough messages to compare (peak {peak})"
        );
    }
    pg.cleanup().await;
}

// ---- 0131: seq and lock order ----

async fn seq(store: &Store, account: &str) -> i64 {
    q("SELECT COALESCE((SELECT seq FROM mail_data_accounts WHERE account_id=?1),-1)")
        .bind(account)
        .fetch_scalar_i64(&store.backend)
        .await
        .unwrap()
}

async fn seq_contract(store: Store) {
    let leak = |sql: String| -> &'static str { Box::leak(sql.into_boxed_str()) };
    for sql in account_sql("sq")
        .split(';')
        .filter(|s| !s.trim().is_empty())
    {
        q(leak(sql.to_owned()))
            .execute(&store.backend)
            .await
            .unwrap();
    }
    let steps: [(&str, bool); 14] = [
        (
            "INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,flags_json) VALUES('sq1','sq','sq_inbox',1,1,'[]')",
            true,
        ),
        (
            "UPDATE messages SET flags_json='[\"Seen\"]' WHERE stable_id='sq1'",
            true,
        ),
        (
            "UPDATE messages SET flags_json=flags_json WHERE stable_id='sq1'",
            false,
        ),
        (
            "UPDATE messages SET mailbox_id='sq_trash' WHERE stable_id='sq1'",
            true,
        ),
        (
            "INSERT INTO message_meta(stable_id,pinned) VALUES('sq1',1)",
            true,
        ),
        (
            "INSERT INTO changes(account_id,type,state,stable_id,op,at) VALUES('sq','Email',1,'sq1','updated','2026-01-01T00:00:00Z')",
            false,
        ),
        (
            "UPDATE changes SET at='2026-01-02T00:00:00Z' WHERE account_id='sq'",
            false,
        ),
        ("DELETE FROM changes WHERE account_id='sq'", false),
        (
            "INSERT INTO pim_changes(account_id,type,state,object_id,op,at) VALUES('sq','Contact',1,'c1','created','2026-01-01T00:00:00Z')",
            false,
        ),
        (
            "INSERT INTO crypto_changes(account_id,type,state,object_id,op,at) VALUES('sq','Key',1,'k1','created','2026-01-01T00:00:00Z')",
            false,
        ),
        (
            "INSERT INTO change_log_checkpoint(account_id,type,head_state,pruned_through) VALUES('sq','Email',1,0)",
            false,
        ),
        (
            "UPDATE change_log_checkpoint SET pruned_through=1 WHERE account_id='sq'",
            false,
        ),
        ("UPDATE mailboxes SET role='sent' WHERE id='sq_arch'", true),
        ("DELETE FROM messages WHERE stable_id='sq1'", true),
    ];
    let mut last = seq(&store, "sq").await;
    for (sql, advances) in steps {
        q(sql).execute(&store.backend).await.unwrap();
        let now = seq(&store, "sq").await;
        if advances {
            assert!(now > last, "{sql}: seq {last} -> {now}");
        } else {
            assert_eq!(now, last, "{sql}: no covered change, seq must stay");
        }
        last = now;
    }
    store.close().await;
}

#[tokio::test]
async fn sqlite_seq_advances_for_every_covered_write() {
    seq_contract(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}

#[tokio::test]
async fn postgres_seq_advances_for_every_covered_write() {
    let Some(mut pg) = Pg::new().await else {
        return;
    };
    let (_, store) = pg.schema("n15_seq").await;
    seq_contract(store).await;
    pg.cleanup().await;
}

/// Two connections write the same account with the statement mixes that
/// deadlocked before 0131 (mailbox upsert against message insert/update/delete).
/// Single statements run alone; a transaction of several statements follows the
/// store's rule (t54 advisory lock, then the mailbox and message rows it will
/// write, locked before its first write; publication_mutations::lock_mailboxes,
/// set_flags_batch). No 40P01 may occur. Before 0131 single statements alone
/// deadlocked (mailbox upsert against message writes).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn postgres_two_writers_of_one_account_do_not_deadlock() {
    let Some(mut pg) = Pg::new().await else {
        return;
    };
    let iterations: usize = std::env::var("MW_N15_LOCK_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(400);
    let (schema, store) = pg.schema("n15_lock").await;
    store.close().await;
    let mut setup = pg.connect(&schema).await;
    raw(&mut setup, &account_sql("lk")).await.unwrap();
    let ids = (0..40).map(|i| format!("lk{i}")).collect::<Vec<_>>();
    let rows = ids
        .iter()
        .enumerate()
        .map(|(i, id)| format!("('{id}','lk','lk_inbox',{i},1,'[]')"))
        .collect::<Vec<_>>()
        .join(",");
    raw(&mut setup, &format!("INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,flags_json) VALUES {rows}")).await.unwrap();
    let worker = move |seed: u64, conn: PgConnection| {
        let ids = ids.clone();
        async move {
            let mut conn = conn;
            let mut rng = Rng(seed);
            let mut fresh = 0u64;
            let mut deadlocks = 0usize;
            for _ in 0..iterations {
                let statements = 1 + rng.below(3);
                let mut tx = String::new();
                let mut touched = Vec::new();
                for _ in 0..statements {
                    let id = rng.pick(&ids).clone();
                    fresh += 1;
                    // A move changes a foreign key and takes the mailbox row's
                    // key-share lock: the store only moves inside a transaction
                    // that locked its mailboxes first (lock_mailboxes).
                    let op = match rng.below(8) {
                        3 if statements == 1 => 2,
                        op => op,
                    };
                    tx += &match op {
                        0 | 1 => format!(
                            "INSERT INTO mailboxes(id,account_id,name,role,uidvalidity) VALUES('lk_x{seed}','lk','x{seed}',NULL,1) ON CONFLICT(account_id,name,uidvalidity) DO UPDATE SET role=CASE WHEN mailboxes.role IS NULL THEN 'sent' ELSE NULL END;"
                        ),
                        2 => format!(
                            "UPDATE messages SET flags_json='[\"Seen\",\"{fresh}\"]' WHERE stable_id='{id}';"
                        ),
                        3 => format!(
                            "UPDATE messages SET mailbox_id=CASE WHEN mailbox_id='lk_inbox' THEN 'lk_sent' ELSE 'lk_inbox' END WHERE stable_id='{id}';"
                        ),
                        4 => format!(
                            "INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,flags_json) VALUES('lk_n{seed}_{fresh}','lk','lk_inbox',{},1,'[]');",
                            1_000_000 * seed + fresh
                        ),
                        5 => format!(
                            "DELETE FROM messages WHERE stable_id LIKE 'lk_n{seed}_%' AND stable_id<>'lk_n{seed}_{fresh}';"
                        ),
                        6 => format!(
                            "UPDATE messages SET thread_id='t{fresh}' WHERE stable_id='{id}';"
                        ),
                        _ => format!(
                            "INSERT INTO changes(account_id,type,state,stable_id,op,at) VALUES('lk','Email',{fresh},'{id}','updated','2026-01-01T00:00:00Z');"
                        ),
                    };
                    touched.push(id);
                }
                if statements > 1 {
                    touched.sort();
                    let list = touched
                        .iter()
                        .map(|i| format!("'{i}'"))
                        .collect::<Vec<_>>()
                        .join(",");
                    tx = format!(
                        "BEGIN;SELECT pg_advisory_xact_lock(hashtextextended('lk',719304));SELECT id FROM mailboxes WHERE account_id='lk' ORDER BY id FOR UPDATE;SELECT stable_id FROM messages WHERE stable_id IN({list}) OR stable_id LIKE 'lk_n{seed}_%' ORDER BY stable_id FOR UPDATE;{tx}"
                    );
                } else {
                    tx = format!("BEGIN;{tx}");
                }
                tx += "COMMIT;";
                if let Err(e) = raw(&mut conn, &tx).await {
                    let _ = raw(&mut conn, "ROLLBACK").await;
                    assert!(e.contains("deadlock"), "unexpected error: {e}");
                    deadlocks += 1;
                }
            }
            deadlocks
        }
    };
    let (c1, c2) = (pg.connect(&schema).await, pg.connect(&schema).await);
    let (a, b) = tokio::join!(worker(1, c1), worker(2, c2));
    let deadlocks = a + b;
    assert_eq!(
        deadlocks, 0,
        "{deadlocks} transactions of {iterations} x 2 deadlocked"
    );
    drop(setup);
    pg.cleanup().await;
}

// ---- 0133: single-message statements no slower than before 0129 ----

/// The schema as it was before 0129: 0113's messages triggers exactly (no seq)
/// and 0110's verdict trigger without 0132's column list.
fn restore_pre_0129(schema: &str) -> String {
    let mut sql = String::new();
    for name in ADDED {
        sql += &format!("DROP TRIGGER {name} ON messages;DROP FUNCTION {name}_fn();\n");
    }
    for name in REPLACED {
        sql += between(OLD_0113, &format!("CREATE FUNCTION {name}_fn()"), "END $$;");
        sql += "\n";
        sql += between(OLD_0113, &format!("CREATE TRIGGER {name} "), ";");
        sql +=
            &format!("\nALTER FUNCTION {name}_fn() SET search_path = pg_catalog, \"{schema}\";\n");
    }
    sql + "DROP TRIGGER verdict_content_revision ON messages;\
           CREATE TRIGGER verdict_content_revision BEFORE INSERT OR UPDATE ON messages FOR EACH ROW EXECUTE FUNCTION mw_verdict_content_revision();"
}

async fn execution_us(conn: &mut PgConnection, sql: &str) -> f64 {
    let rows = sqlx::query(&format!("EXPLAIN (ANALYZE) {sql}"))
        .fetch_all(&mut *conn)
        .await
        .unwrap_or_else(|e| panic!("{sql}: {e}"));
    rows.iter()
        .filter_map(|r| {
            r.get::<String, _>(0)
                .trim()
                .strip_prefix("Execution Time: ")
                .and_then(|t| t.trim_end_matches(" ms").parse::<f64>().ok())
        })
        .next()
        .expect("EXPLAIN ANALYZE prints the execution time")
        * 1000.0
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// One-row INSERT, flag UPDATE, move UPDATE and DELETE (the shape of every
/// Email/set the goal mix sends) through all messages triggers: the current
/// schema against the pre-0129 schema in the same server, statements
/// interleaved one by one so host load affects both alike. Median statement
/// time must not exceed the pre-0129 median by more than 5% (noise margin;
/// measured at 0133 in psql, outbox present/absent: INSERT -8/-8%, flag -13/-9%,
/// move -17/-17%, DELETE -1/-3%; DELETE is the one within noise).
/// Both with the account's outbox row present and absent (an acknowledged
/// hint deletes it, so the next write inserts it again).
#[tokio::test]
async fn postgres_one_row_writes_no_slower_than_before_0129() {
    let Some(mut pg) = Pg::new().await else {
        return;
    };
    let rounds: usize = std::env::var("MW_N15_COST_ROUNDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    let (pre_schema, pre_store) = pg.schema("n15_pre").await;
    let (cur_schema, cur_store) = pg.schema("n15_cur").await;
    pre_store.close().await;
    cur_store.close().await;
    let mut pre = pg.connect(&pre_schema).await;
    let mut cur = pg.connect(&cur_schema).await;
    raw(&mut pre, &restore_pre_0129(&pre_schema)).await.unwrap();
    let envelope = "convert_to(repeat('envelope-subject-and-addresses ',20),'UTF8')";
    for conn in [&mut pre, &mut cur] {
        raw(conn, &account_sql("ct")).await.unwrap();
        raw(conn, &format!("INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,internaldate,size,flags_json,envelope_json) SELECT 'bg'||g,'ct','ct_inbox',100000+g,1,'2026-01-01T00:00:00Z',100,'[]',{envelope} FROM generate_series(1,10000) g")).await.unwrap();
        raw(conn, "VACUUM ANALYZE").await.unwrap();
    }
    let ops = [
        (
            "insert",
            format!(
                "INSERT INTO messages(stable_id,account_id,mailbox_id,uid,uidvalidity,internaldate,size,flags_json,envelope_json) VALUES('c{{i}}','ct','ct_inbox',{{i}},1,'2026-01-01T00:00:00Z',100,'[]',{envelope})"
            ),
        ),
        (
            "flag",
            "UPDATE messages SET flags_json='[\"Seen\"]' WHERE stable_id='c{i}'".to_owned(),
        ),
        (
            "move",
            "UPDATE messages SET mailbox_id='ct_trash' WHERE stable_id='c{i}'".to_owned(),
        ),
        (
            "delete",
            "DELETE FROM messages WHERE stable_id='c{i}'".to_owned(),
        ),
    ];
    // A loaded host can still tilt one batch; a regression shows in every attempt.
    let mut report = Vec::new();
    let mut slower = Vec::new();
    for attempt in 0..3 {
        report.clear();
        slower.clear();
        for (mode, absent) in [("outbox row present", false), ("outbox row absent", true)] {
            let mut times: BTreeMap<(&str, bool), Vec<f64>> = BTreeMap::new();
            let warmup = 20;
            for round in 0..warmup + rounds {
                let i = round + if absent { 1_000_000 } else { 0 } + attempt * 100_000;
                for (op, template) in &ops {
                    let sql = template.replace("{i}", &i.to_string());
                    for current in if round % 2 == 0 {
                        [false, true]
                    } else {
                        [true, false]
                    } {
                        let conn = if current { &mut cur } else { &mut pre };
                        if absent {
                            raw(conn, "DELETE FROM mail_data_outbox WHERE account_id='ct'")
                                .await
                                .unwrap();
                        }
                        let us = execution_us(conn, &sql).await;
                        if round >= warmup {
                            times.entry((op, current)).or_default().push(us);
                        }
                    }
                }
            }
            for (op, _) in &ops {
                let before = median(times[&(*op, false)].clone());
                let now = median(times[&(*op, true)].clone());
                report.push(format!(
                    "{mode:18} {op:7} pre-0129 {before:7.1} us  now {now:7.1} us  ({:+5.1}%)",
                    (now / before - 1.0) * 100.0
                ));
                if now > before * 1.05 {
                    slower.push(format!("{mode} {op}"));
                }
            }
        }
        eprintln!("attempt {attempt}:\n{}", report.join("\n"));
        if slower.is_empty() {
            break;
        }
    }
    assert!(
        slower.is_empty(),
        "one-row writes more than 5% slower than pre-0129 in three attempts: {slower:?}\n{}",
        report.join("\n")
    );
    drop((pre, cur));
    pg.cleanup().await;
}
