//! Schema/required-trigger eligibility for published DATA, not malicious-DBA
//! protection. Normal DDL changes the cheap stamp; cold validation checks exact
//! installed definitions before any prepared inputs can be reused under it.
use super::*;
use std::sync::OnceLock;
fn invalid(message:&'static str)->StoreError { StoreError::Invalid(message) }
const SQLITE_SQL: &str = concat!(include_str!("../migrations/0109_message_list_envelope.sql"),"\n",include_str!("../migrations/0110_verdict_source.sql"),"\n",include_str!("../migrations/0113_mail_data_publication.sql"));
const PG_SQL: &str = concat!(include_str!("../migrations_pg/0109_message_list_envelope.sql"),"\n",include_str!("../migrations_pg/0110_verdict_source.sql"),"\n",include_str!("../migrations_pg/0113_mail_data_publication.sql"));
const TABLES: &[&str] = &["accounts","messages","message_meta","message_list_envelope","bodies","mailboxes","address_books","contacts","zeroaccess_accounts","security_verdicts","mail_data_accounts","mail_data_rows","mail_data_outbox","changes","change_log_checkpoint","pim_changes","crypto_changes"];
pub(crate) fn stamp_expression(dialect: Dialect) -> &'static str {
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
    columns:String,
}
struct Function {
    name: String,
    args: i64,
    body: String,
}
fn triggers(pg: bool) -> Vec<Trigger> {
    let source = if pg { PG_SQL } else { SQLITE_SQL };
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
        let words = chunk.split_whitespace().collect::<Vec<_>>();
        let Some(on) = words.iter().position(|w| *w == "ON") else {
            continue;
        };
        let mut kind = 1i64;
        for token in &words[1..on] {
            kind |= match *token {
                "BEFORE" => 2,
                "INSERT" => 4,
                "DELETE" => 8,
                "UPDATE" => 16,
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
            columns: chunk.split_once("UPDATE OF ").and_then(|(_,tail)|tail.split_once(" ON ")).map(|(columns,_)|columns.split_whitespace().collect::<String>()).unwrap_or_default(),
        });
    }
    out
}
fn functions() -> Vec<Function> {
    PG_SQL
        .split("CREATE FUNCTION ")
        .skip(1)
        .filter_map(|chunk| {
            let (name, tail) = chunk.split_once('(')?;
            let args = tail.split_once(')')?.0;
            let body = chunk
                .split_once("AS $$")?
                .1
                .split_once("$$;")?
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
    pub async fn validate_mail_data_schema(
        &self,
        fence: &MailDataFence,
    ) -> Result<bool, StoreError> {
        if fence.descriptor.scope != self.cache_scope() {
            return Err(invalid("foreign publication schema source"));
        }
        let Some(expected) = fence.schema_stamp() else {
            return Ok(false);
        };
        let valid = match self.backend.dialect() {
            Dialect::Sqlite => {
                let required = triggers(false);
                if required.is_empty() || required.len() > 128 {
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
                let required = triggers(true);
                let functions = functions();
                if required.is_empty() || required.len() > 128 || functions.len() > 64 {
                    return Ok(false);
                }
                let names =
                    serde_json::to_string(&required.iter().map(|r| &r.name).collect::<Vec<_>>())
                        .map_err(|_| invalid("schema names encoding"))?;
                let rows=q("SELECT w.value AS name,c.relname AS table_name,p.proname AS function_name,CAST(t.tgtype AS BIGINT) AS kind,CAST(CASE WHEN t.tgenabled='O' AND NOT t.tgisinternal AND t.tgqual IS NULL AND t.tgnargs=0 AND t.tgconstraint=0 AND NOT t.tgdeferrable AND NOT t.tginitdeferred AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL AND pn.nspname=current_schema() THEN 1 ELSE 0 END AS BIGINT) AS enabled,(SELECT COALESCE(string_agg(a.attname,',' ORDER BY k.ord),'') FROM unnest(t.tgattr::smallint[]) WITH ORDINALITY k(num,ord) JOIN pg_catalog.pg_attribute a ON a.attrelid=t.tgrelid AND a.attnum=k.num) AS columns FROM json_array_elements_text(?1::json) w LEFT JOIN pg_catalog.pg_trigger t ON t.tgname=w.value AND t.tgrelid IN(SELECT c.oid FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=current_schema()) LEFT JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid LEFT JOIN pg_catalog.pg_proc p ON p.oid=t.tgfoid LEFT JOIN pg_catalog.pg_namespace pn ON pn.oid=p.pronamespace ORDER BY w.value LIMIT 129").bind(names).fetch_all(&self.backend).await?;
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
                                && r.get_i64("enabled") == 1 && r.get_opt_string("columns").as_deref()==Some(t.columns.as_str())
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
                let rows=q("SELECT w.value->>'name' AS name,CAST(p.pronargs AS BIGINT) AS args,CASE WHEN (SELECT COALESCE(SUM(octet_length(x.prosrc)),0) FROM pg_catalog.pg_proc x JOIN pg_catalog.pg_namespace ns ON ns.oid=x.pronamespace JOIN json_array_elements(?1::json) wanted ON wanted.value->>'name'=x.proname AND (wanted.value->>'args')::bigint=x.pronargs WHERE ns.nspname=current_schema())<=262144 AND octet_length(p.prosrc)<=16384 THEN p.prosrc ELSE NULL END AS definition,CAST(CASE WHEN l.lanname='plpgsql' AND p.prokind='f' AND p.provolatile='v' AND NOT p.prosecdef AND NOT p.proisstrict AND NOT p.proleakproof AND p.pronargdefaults=0 AND p.proparallel='u' AND p.prorettype='pg_catalog.trigger'::regtype AND p.pronargs=0 AND ((p.proname LIKE 'mail_data_%' AND cardinality(p.proconfig)=1 AND p.proconfig[1]='search_path=pg_catalog, '||quote_ident(current_schema())) OR (p.proname NOT LIKE 'mail_data_%' AND p.proconfig IS NULL)) THEN 1 ELSE 0 END AS BIGINT) AS valid FROM json_array_elements(?1::json) w LEFT JOIN pg_catalog.pg_namespace n ON n.nspname=current_schema() LEFT JOIN pg_catalog.pg_proc p ON p.pronamespace=n.oid AND p.proname=w.value->>'name' AND p.pronargs=(w.value->>'args')::bigint LEFT JOIN pg_catalog.pg_language l ON l.oid=p.prolang ORDER BY w.value->>'name' LIMIT 65").bind(names).fetch_all(&self.backend).await?;
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
