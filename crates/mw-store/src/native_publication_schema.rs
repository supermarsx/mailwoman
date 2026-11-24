//! Schema/required-trigger eligibility for published DATA, not malicious-DBA
//! protection. Normal DDL changes the cheap stamp; cold validation checks exact
//! installed definitions before any prepared inputs can be reused under it.
use super::*;
const SQLITE_SQL: &str = concat!(
    // t34 (review N1): the settings-revision triggers. The bounded guard
    // snapshot reuses settings-derived values (session-watch preferences) while
    // the revision is unchanged, so a dropped one must disqualify published
    // inputs instead of leaving those values silently stale. PG_SQL requires
    // the PostgreSQL 0078 triggers as well (t49pg).
    include_str!("../migrations/0078_settings_revision.sql"),
    "\n",
    include_str!("../migrations/0111_authority_publication.sql"),
    "\n",
    include_str!("../migrations/0112_authority_subject_anchor.sql"),
    "\n",
    include_str!("../migrations/0115_authority_change_log.sql")
);
const PG_SQL: &str = concat!(
    // t49pg (review N1, decision D12): the 0078 statement-level triggers. 0120
    // pins the search_path of their function, so they fit the validator.
    include_str!("../migrations_pg/0078_settings_revision.sql"),
    "\n",
    include_str!("../migrations_pg/0111_authority_publication.sql"),
    "\n",
    include_str!("../migrations_pg/0112_authority_subject_anchor.sql"),
    "\n",
    // No 0115 here: 0118 drops its triggers, including the per-insert prune
    // (t34 S2). 0120 re-creates the two change-log triggers without a prune.
    include_str!("../migrations_pg/0120_authority_change_log_xid.sql")
);
/// Largest required-trigger set the validator accepts, per dialect.
const MAX_REQUIRED: usize = 192;
const TABLES: &[&str] = &[
    "accounts",
    "sessions",
    "settings",
    "domains",
    "domain_alias",
    "setting_policy",
    "setting_value",
    "personal_setting_value",
    "user_group",
    "user_group_member",
    "feature_profile",
    "feature_profile_assignment",
    "feature_user_subject",
    "session_principal",
    "identity_principal",
    "identity_binding",
    "linked_account",
    "linked_context",
    "mail_user",
    "mail_user_account",
    "zeroaccess_accounts",
    "account_restriction",
    "consent_record",
    "authority_epoch",
    "authority_publication_outbox",
    "authority_subject_anchor",
    // 0115: the bounded guard snapshot reads the change log and depends on the
    // settings_revision row; both join the shadowing/identity stamp.
    "settings_revision",
    "authority_change_log",
];
pub(super) fn stamp_expression(dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[String; 2]> = OnceLock::new();
    &SQL.get_or_init(||{
        let names=TABLES.iter().map(|n|format!("'{n}'")).collect::<Vec<_>>().join(",");
        let sqlite=format!("(SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM sqlite_temp_master WHERE name IN({names})) THEN 'sqlite:'||CAST(schema_version AS TEXT) ELSE NULL END FROM pragma_schema_version)");
        let pg=format!("(WITH r AS (SELECT c.oid,'r:'||c.oid::text||':'||c.xmin::text||':'||c.relfilenode::text AS value FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=current_schema() AND c.relname IN({names}) AND c.relkind='r' AND pg_catalog.to_regclass(pg_catalog.quote_ident(c.relname))=c.oid LIMIT 33),t AS (SELECT t.tgfoid,'t:'||t.oid::text||':'||t.xmin::text||':'||t.tgenabled::text AS value FROM pg_catalog.pg_trigger t WHERE t.tgrelid IN(SELECT oid FROM r) AND NOT t.tgisinternal ORDER BY t.oid LIMIT 513),p AS (SELECT 'p:'||p.oid::text||':'||p.xmin::text AS value FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE p.oid IN(SELECT tgfoid FROM t) OR (n.nspname=current_schema() AND p.proname='mw_authority_bump') ORDER BY p.oid LIMIT 129),v AS (SELECT value FROM r UNION ALL SELECT value FROM t UNION ALL SELECT value FROM p) SELECT CASE WHEN (SELECT COUNT(*) FROM r)={} AND (SELECT COUNT(*) FROM t)<=512 AND (SELECT COUNT(*) FROM p)<=128 THEN 'pg:'||md5(string_agg(value,',' ORDER BY value)) ELSE NULL END FROM v)",TABLES.len());
        [sqlite,pg]
    })[usize::from(dialect==Dialect::Postgres)]
}
pub(super) fn stamp_sql(dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[String; 2]> = OnceLock::new();
    &SQL.get_or_init(|| {
        std::array::from_fn(|pg| {
            format!(
                "SELECT {} AS publication_schema",
                stamp_expression(if pg == 1 {
                    Dialect::Postgres
                } else {
                    Dialect::Sqlite
                })
            )
        })
    })[usize::from(dialect == Dialect::Postgres)]
}
fn canonical_sql(sql: &str) -> String {
    sql.replace("\r\n", "\n")
        .trim()
        .trim_end_matches(';')
        .trim()
        .to_owned()
}
struct Trigger {
    name: String,
    table: String,
    function: String,
    kind: i64,
    sql: String,
}
struct Function {
    name: String,
    args: i64,
    body: String,
}
fn triggers(pg: bool) -> Vec<Trigger> {
    let source = if pg { PG_SQL } else { SQLITE_SQL };
    triggers_from(source, pg)
}
fn triggers_from(source: &str, pg: bool) -> Vec<Trigger> {
    let mut out = Vec::new();
    for chunk in source.split("CREATE TRIGGER ").skip(1) {
        let end = if pg {
            chunk.find(';')
        } else {
            chunk.find("END;").map(|i| i + 3)
        };
        let Some(end) = end else {
            continue;
        };
        let sql = format!("CREATE TRIGGER {}", &chunk[..=end]);
        let words = chunk[..=end].split_whitespace().collect::<Vec<_>>();
        let Some(on) = words.iter().position(|w| *w == "ON") else {
            continue;
        };
        // pg_trigger.tgtype bits: ROW 1, BEFORE 2, INSERT 4, DELETE 8, UPDATE 16,
        // TRUNCATE 32. `FOR EACH STATEMENT` (0078 on PostgreSQL) leaves ROW unset.
        let statement = words[on..]
            .windows(3)
            .any(|w| w == ["FOR", "EACH", "STATEMENT"]);
        let mut kind = i64::from(!statement);
        for token in &words[1..on] {
            kind |= match *token {
                "BEFORE" => 2,
                "INSERT" => 4,
                "DELETE" => 8,
                "UPDATE" => 16,
                "TRUNCATE" => 32,
                _ => 0,
            };
        }
        let function = chunk
            .split("EXECUTE FUNCTION ")
            .nth(1)
            .and_then(|s| s.split('(').next())
            .unwrap_or("")
            .to_owned();
        out.push(Trigger {
            name: words[0].to_owned(),
            table: words[on + 1].to_owned(),
            function,
            kind,
            sql: canonical_sql(&sql),
        });
    }
    out
}
fn functions() -> Vec<Function> {
    functions_from(PG_SQL)
}
fn functions_from(source: &str) -> Vec<Function> {
    source
        .split("CREATE FUNCTION ")
        .skip(1)
        .filter_map(|chunk| {
            let (name, tail) = chunk.split_once('(')?;
            let args = tail.split_once(')')?.0;
            // The body ends at the closing `$$`, whether `;` follows (0111) or
            // `LANGUAGE plpgsql;` does (0078).
            let body = chunk
                .split_once("AS $$")?
                .1
                .split_once("$$")?
                .0
                .trim()
                .replace("\r\n", "\n");
            Some(Function {
                name: name.trim().into(),
                args: if args.trim().is_empty() {
                    0
                } else {
                    args.split(',').count() as i64
                },
                body,
            })
        })
        .collect()
}
impl Store {
    /// Cold-only exact required-definition check, always bounded before driver
    /// transfer. Warm requests compare the same cheap schema stamp inside their
    /// existing ONE q() fence. Every component key also includes that stamp.
    pub async fn validate_native_publication_schema(
        &self,
        fence: &NativeAuthorityFence,
    ) -> Result<bool, StoreError> {
        self.validate_native_publication_schema_with_extra(fence, "", "").await
    }
    pub(super) async fn validate_native_publication_schema_with_extra(
        &self,
        fence: &NativeAuthorityFence,
        sqlite_extra: &'static str,
        postgres_extra: &'static str,
    ) -> Result<bool, StoreError> {
        if fence.versions().store != self.cache_scope() {
            return Err(invalid("foreign publication schema source"));
        }
        let Some(expected) = fence.publication_schema() else {
            return Ok(false);
        };
        self.validate_publication_schema_source(fence.versions(), expected, sqlite_extra, postgres_extra).await
    }
    pub(super) async fn validate_publication_schema_source(
        &self,
        versions: &AuthorityVersions,
        expected: &str,
        sqlite_extra: &'static str,
        postgres_extra: &'static str,
    ) -> Result<bool, StoreError> {
        if versions.store != self.cache_scope() {
            return Err(invalid("foreign publication schema source"));
        }
        let valid = match self.backend.dialect() {
            Dialect::Sqlite => {
                let mut required = triggers(false);
                required.extend(triggers_from(sqlite_extra, false));
                if required.is_empty() || required.len() > MAX_REQUIRED {
                    return Ok(false);
                }
                let names =
                    serde_json::to_string(&required.iter().map(|r| &r.name).collect::<Vec<_>>())
                        .map_err(|_| invalid("schema names encoding"))?;
                let sql = "SELECT w.value AS name,CASE WHEN (SELECT COALESCE(SUM(octet_length(x.sql)),0) FROM sqlite_master x JOIN json_each(?1) wanted ON wanted.value=x.name WHERE x.type='trigger')<=262144 AND typeof(s.sql)='text' AND octet_length(s.sql)<=16384 THEN s.sql ELSE NULL END AS definition FROM json_each(?1) w LEFT JOIN sqlite_master s ON s.name=w.value AND s.type='trigger' ORDER BY w.value";
                let rows = q(sql).bind(names).fetch_all(&self.backend).await?;
                rows.len() == required.len()
                    && rows.iter().all(|r| {
                        let Ok(Some(name)) = r.try_get_opt_string("name") else {
                            return false;
                        };
                        let Ok(Some(definition)) = r.try_get_opt_string("definition") else {
                            return false;
                        };
                        required
                            .iter()
                            .any(|v| v.name == name && canonical_sql(&definition) == v.sql)
                    })
            }
            Dialect::Postgres => {
                let mut required = triggers(true);
                required.extend(triggers_from(postgres_extra, true));
                let mut functions = functions();
                functions.extend(functions_from(postgres_extra));
                if required.is_empty() || required.len() > MAX_REQUIRED || functions.len() > 64 {
                    return Ok(false);
                }
                let names =
                    serde_json::to_string(&required.iter().map(|r| &r.name).collect::<Vec<_>>())
                        .map_err(|_| invalid("schema names encoding"))?;
                let rows=q("SELECT w.value AS name,c.relname AS table_name,p.proname AS function_name,CAST(t.tgtype AS BIGINT) AS kind,CAST(CASE WHEN t.tgenabled='O' AND NOT t.tgisinternal AND t.tgqual IS NULL AND t.tgnargs=0 AND t.tgattr::text='' AND t.tgconstraint=0 AND NOT t.tgdeferrable AND NOT t.tginitdeferred AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL AND pn.nspname=current_schema() THEN 1 ELSE 0 END AS BIGINT) AS enabled FROM json_array_elements_text(?1::json) w LEFT JOIN pg_catalog.pg_trigger t ON t.tgname=w.value AND t.tgrelid IN(SELECT c.oid FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=current_schema()) LEFT JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid LEFT JOIN pg_catalog.pg_proc p ON p.oid=t.tgfoid LEFT JOIN pg_catalog.pg_namespace pn ON pn.oid=p.pronamespace ORDER BY w.value LIMIT 193").bind(names).fetch_all(&self.backend).await?;
                if rows.len() != required.len()
                    || rows.iter().any(|r| {
                        let (Ok(Some(name)), Ok(Some(table)), Ok(Some(function))) = (
                            r.try_get_opt_string("name"),
                            r.try_get_opt_string("table_name"),
                            r.try_get_opt_string("function_name"),
                        ) else {
                            return true;
                        };
                        !required.iter().any(|t| {
                            t.name == name
                                && t.table == table
                                && t.function == function
                                && t.kind == r.get_i64("kind")
                                && r.get_i64("enabled") == 1
                        })
                    })
                {
                    return Ok(false);
                }
                let names = serde_json::to_string(
                    &functions
                        .iter()
                        .map(|f| serde_json::json!({"name":f.name,"args":f.args}))
                        .collect::<Vec<_>>(),
                )
                .map_err(|_| invalid("function names encoding"))?;
                let rows=q("SELECT w.value->>'name' AS name,CAST(p.pronargs AS BIGINT) AS args,CASE WHEN (SELECT COALESCE(SUM(octet_length(x.prosrc)),0) FROM pg_catalog.pg_proc x JOIN pg_catalog.pg_namespace ns ON ns.oid=x.pronamespace JOIN json_array_elements(?1::json) wanted ON wanted.value->>'name'=x.proname AND (wanted.value->>'args')::bigint=x.pronargs WHERE ns.nspname=current_schema())<=262144 AND octet_length(p.prosrc)<=16384 THEN p.prosrc ELSE NULL END AS definition,CAST(CASE WHEN l.lanname='plpgsql' AND p.prokind='f' AND p.provolatile='v' AND NOT p.prosecdef AND NOT p.proisstrict AND NOT p.proleakproof AND p.pronargdefaults=0 AND p.proparallel='u' AND cardinality(p.proconfig)=1 AND p.proconfig[1]='search_path=pg_catalog, '||quote_ident(current_schema()) AND ((p.proname='mw_authority_bump' AND p.prorettype='pg_catalog.void'::regtype AND p.proargtypes::text='25 25') OR (p.proname<>'mw_authority_bump' AND p.prorettype='pg_catalog.trigger'::regtype AND p.pronargs=0)) THEN 1 ELSE 0 END AS BIGINT) AS valid FROM json_array_elements(?1::json) w LEFT JOIN pg_catalog.pg_namespace n ON n.nspname=current_schema() LEFT JOIN pg_catalog.pg_proc p ON p.pronamespace=n.oid AND p.proname=w.value->>'name' AND p.pronargs=(w.value->>'args')::bigint LEFT JOIN pg_catalog.pg_language l ON l.oid=p.prolang ORDER BY w.value->>'name' LIMIT 65").bind(names).fetch_all(&self.backend).await?;
                rows.len() == functions.len()
                    && rows.iter().all(|r| {
                        let (Ok(Some(name)), Ok(Some(body))) = (
                            r.try_get_opt_string("name"),
                            r.try_get_opt_string("definition"),
                        ) else {
                            return false;
                        };
                        functions.iter().any(|f| {
                            f.name == name
                                && f.args == r.get_i64("args")
                                && canonical_sql(&body) == canonical_sql(&f.body)
                                && r.get_i64("valid") == 1
                        })
                    })
            }
        };
        if !valid {
            return Ok(false);
        }
        let row = q(stamp_sql(self.backend.dialect()))
            .fetch_one(&self.backend)
            .await?;
        Ok(row.try_get_opt_string("publication_schema")?.as_deref() == Some(expected))
    }
}

#[cfg(test)]
pub(super) fn fixture_restore_trigger(dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[String; 2]> = OnceLock::new();
    &SQL.get_or_init(|| {
        [
            triggers(false)
                .into_iter()
                .find(|t| t.name == "authority_settings_update")
                .unwrap()
                .sql,
            triggers(true)
                .into_iter()
                .find(|t| t.name == "authority_settings")
                .unwrap()
                .sql,
        ]
    })[usize::from(dialect == Dialect::Postgres)]
}

/// `(trigger, table)` of every required trigger of one dialect, without extras.
#[cfg(test)]
pub(super) fn required_triggers(pg: bool) -> Vec<(String, String)> {
    triggers(pg).into_iter().map(|t| (t.name, t.table)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_required_trigger_table_is_stamped() {
        for pg in [false, true] {
            for trigger in triggers(pg) {
                assert!(
                    TABLES.contains(&trigger.table.as_str()),
                    "{} on {} (pg={pg}) is not in the stamp table list",
                    trigger.name,
                    trigger.table
                );
            }
        }
        // 0111/0112/0114 sources, 0078 tables and the mirror's own tables.
        for extra in [
            include_str!("../migrations_pg/0114_guard_input_publication.sql"),
            include_str!("../migrations/0114_guard_input_publication.sql"),
        ] {
            for trigger in triggers_from(extra, true) {
                assert!(TABLES.contains(&trigger.table.as_str()), "{}", trigger.table);
            }
        }
        for table in ["authority_epoch", "settings_revision", "authority_change_log"] {
            assert!(TABLES.contains(&table));
        }
    }

    #[test]
    fn postgres_required_set_has_statement_level_and_change_log_triggers() {
        let pg = triggers(true);
        assert!(pg.len() <= MAX_REQUIRED && triggers(false).len() <= MAX_REQUIRED);
        let kind = |name: &str| pg.iter().find(|t| t.name == name).map(|t| t.kind);
        // 0078: AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ... FOR EACH STATEMENT.
        assert_eq!(kind("settings_revision_setting_policy"), Some(4 | 8 | 16 | 32));
        assert_eq!(
            pg.iter()
                .filter(|t| t.name.starts_with("settings_revision_"))
                .count(),
            11
        );
        // 0120: row-level, no prune trigger.
        assert_eq!(kind("authority_change_log_epoch"), Some(1 | 4 | 8 | 16));
        assert_eq!(kind("authority_change_log_revision"), Some(1 | 4 | 8 | 16));
        assert_eq!(kind("authority_change_log_prune"), None);
        assert_eq!(kind("authority_epoch_monotonic"), Some(1 | 2 | 16));
        let functions = functions();
        let body = |name: &str| {
            functions
                .iter()
                .find(|f| f.name == name)
                .map(|f| canonical_sql(&f.body))
        };
        assert_eq!(
            body("settings_revision_bump").as_deref(),
            Some("BEGIN\n UPDATE settings_revision SET n=n+1 WHERE id=1;\n RETURN NULL;\nEND")
        );
        assert!(body("mw_authority_change_log_epoch").is_some());
        assert!(body("mw_authority_change_log_prune").is_none());
        assert!(functions.iter().all(|f| !f.body.contains("$$")));
    }
}

/// The tables the publication schema stamp covers.
#[cfg(test)]
pub(super) fn stamped_tables() -> &'static [&'static str] {
    TABLES
}
