//! Additive fresh native-admission inputs. NOT an authenticated Session, allow
//! cache, mutation lease, or replacement for the complete server guard chain.
//! No production route calls this prerequisite API yet.
use crate::{
    Account, DomainConfigRow, Session, Store, StoreError,
    backend::{Dialect, Row, q},
    settings_policy::SettingsReadRevision,
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::OnceLock,
};

pub const MAX_DOMAINS: usize = 16;
pub const MAX_KEYS: usize = 256;
pub const MAX_PARAMETERS: usize = 128 * 1024;
pub const MAX_PAYLOAD: usize = 8 * 1024 * 1024;
const MAX_ROWS: usize = 5400;

/// Query shaping only: the caller MUST establish that these cover all names in
/// the fresh source before evaluating. Prepared names never establish identity.
#[derive(Clone)]
pub struct NativeAdmissionRequest {
    domains: Vec<String>,
    keys: Vec<String>,
}
impl std::fmt::Debug for NativeAdmissionRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeAdmissionRequest")
            .field("domains", &self.domains.len())
            .field("keys", &self.keys.len())
            .finish_non_exhaustive()
    }
}
impl NativeAdmissionRequest {
    pub fn new(domains: Vec<String>, keys: Vec<String>) -> Result<Self, StoreError> {
        if domains.len() > MAX_DOMAINS
            || keys.len() > MAX_KEYS
            || domains.iter().chain(&keys).any(|s| s.len() > 1024)
            || domains.iter().chain(&keys).map(String::len).sum::<usize>() > MAX_PARAMETERS
        {
            return Err(corrupt("input bound"));
        }
        Ok(Self { domains, keys })
    }
    pub fn domains(&self) -> &[String] {
        &self.domains
    }
    pub fn keys(&self) -> &[String] {
        &self.keys
    }
}
/// Opaque, request-local observation. Section errors are deliberately decoded at
/// the caller's ORIGINAL semantic boundary, not eagerly in SQL/field iteration.
pub struct NativeAdmissionSnapshot {
    scope: uuid::Uuid,
    rows: Vec<Row>,
    requested: NativeAdmissionRequest,
}
impl std::fmt::Debug for NativeAdmissionSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeAdmissionSnapshot")
            .field("rows", &self.rows.len())
            .finish_non_exhaustive()
    }
}
pub struct NativeSessionSource {
    pub session: Session,
    pub last_seen: Option<String>,
    pub authentication_method: String,
}
impl std::fmt::Debug for NativeSessionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeSessionSource")
            .finish_non_exhaustive()
    }
}
/// Raw fresh verified tuple: evaluation still must validate all owner/binding/
/// principal/realm relations before deriving any settings/storage namespace.
pub struct NativePrincipalSource {
    pub primary_account_id: String,
    pub principal_id: String,
    pub binding_id: String,
    pub binding_principal_id: String,
    pub state: String,
    pub binding_kind: String,
    pub retired_at: Option<i64>,
    pub auth_time: i64,
    pub authentication_method: String,
}
impl std::fmt::Debug for NativePrincipalSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativePrincipalSource")
            .finish_non_exhaustive()
    }
}
pub struct NativeDomainSource {
    pub canonical: String,
    pub second_alias: bool,
    pub configuration: Option<DomainConfigRow>,
}
impl std::fmt::Debug for NativeDomainSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeDomainSource").finish_non_exhaustive()
    }
}
fn corrupt(what: &str) -> StoreError {
    StoreError::Corrupt(format!("native admission {what}"))
}
fn text(r: &Row, key: &str) -> Result<String, StoreError> {
    r.try_get_opt_string(key)?
        .ok_or_else(|| corrupt("missing text"))
}
fn int(r: &Row, key: &str) -> Result<Option<i64>, StoreError> {
    use sqlx::Row as _;
    Ok(match r {
        Row::Sqlite(r) => r.try_get(key)?,
        Row::Postgres(r) => r.try_get(key)?,
    })
}
fn check(r: &Row) -> Result<(), StoreError> {
    if int(r, "invalid")? != Some(0) {
        return Err(corrupt("unsupported storage or payload bound"));
    }
    Ok(())
}
impl NativeAdmissionSnapshot {
    fn one(&self, tag: &str) -> Result<Option<&Row>, StoreError> {
        let row = self.rows.iter().find(|r| r.get_string("tag") == tag);
        if let Some(r) = row {
            check(r)?;
        }
        Ok(row)
    }
    /// Coverage of query shaping ONLY; never an admission/identity decision.
    pub fn covers_domains(&self, names: &[String]) -> bool {
        names.len() <= MAX_DOMAINS
            && names
                .iter()
                .all(|name| self.requested.domains.contains(name))
    }
    pub fn covers_keys(&self, keys: &[String]) -> bool {
        keys.len() <= MAX_KEYS && keys.iter().all(|key| self.requested.keys.contains(key))
    }
    fn scoped(&self, store: &Store) -> Result<(), StoreError> {
        if store.cache_scope() != self.scope {
            return Err(corrupt("foreign Store scope"));
        }
        Ok(())
    }
    pub fn revision(&self, store: &Store) -> Result<SettingsReadRevision, StoreError> {
        self.scoped(store)?;
        let r = self
            .one("revision")?
            .ok_or_else(|| corrupt("revision section"))?;
        let n = int(r, "revision")?;
        if n.is_some_and(|n| n < 0) {
            return Err(corrupt("negative revision"));
        }
        Ok(store.settings_revision_from_snapshot(n))
    }
    pub fn session(&self, store: &Store) -> Result<Option<NativeSessionSource>, StoreError> {
        self.scoped(store)?;
        let Some(r) = self.one("session")? else {
            return Ok(None);
        };
        let session = Session {
            id: text(r, "id")?,
            account_id: text(r, "account_id")?,
            username: text(r, "username")?,
            jmap_url: text(r, "jmap_url")?,
            api_url: text(r, "api_url")?,
            credentials: crate::decode_creds(&store.key.open(&r.get_blob("sealed_creds"))?)?,
        };
        Ok(Some(NativeSessionSource {
            session,
            last_seen: r.try_get_opt_string("last_seen").ok().flatten(),
            authentication_method: text(r, "authentication_method")?,
        }))
    }
    pub fn clocks(&self) -> Result<Option<(String, String)>, StoreError> {
        self.one("clock")?
            .map(|r| Ok((text(r, "clock_created_at")?, text(r, "activity_at")?)))
            .transpose()
    }
    pub fn bounded_shape(&self) -> bool {
        self.rows.iter().all(|r| check(r).is_ok())
    }
    pub fn account(&self) -> Result<Option<Account>, StoreError> {
        self.one("account")?
            .map(|row| {
                for name in ["id", "kind", "host", "tls", "username", "sync_policy_json"] {
                    text(row, name)?;
                }
                int(row, "port")?.ok_or_else(|| corrupt("missing account port"))?;
                Store::account_from_row(row)
            })
            .transpose()
    }
    pub fn names(&self) -> Result<Vec<String>, StoreError> {
        match self.one("names")? {
            Some(r) => crate::v6::parse_account_names(&text(r, "setting_value")?),
            None => Ok(Vec::new()),
        }
    }
    pub fn linked_policy_required(&self) -> Result<bool, StoreError> {
        let r = self
            .one("linked")?
            .ok_or_else(|| corrupt("linked section"))?;
        Ok(int(r, "required")? == Some(1))
    }
    pub fn principal(&self) -> Result<Option<NativePrincipalSource>, StoreError> {
        let Some(r) = self.one("principal")? else {
            return Ok(None);
        };
        Ok(Some(NativePrincipalSource {
            primary_account_id: text(r, "primary_account_id")?,
            principal_id: text(r, "principal_id")?,
            binding_id: text(r, "binding_id")?,
            binding_principal_id: text(r, "binding_principal_id")?,
            state: text(r, "state")?,
            binding_kind: text(r, "binding_kind")?,
            retired_at: int(r, "retired_at")?,
            auth_time: int(r, "auth_time")?
                .ok_or_else(|| corrupt("missing principal auth time"))?,
            authentication_method: text(r, "principal_auth_method")?,
        }))
    }
    pub fn domains(&self) -> HashMap<String, Result<NativeDomainSource, StoreError>> {
        self.rows
            .iter()
            .filter(|r| r.get_string("tag") == "domain")
            .map(|r| {
                let input = r.get_string("input");
                let value = (|| {
                    check(r)?;
                    Ok(NativeDomainSource {
                        canonical: text(r, "canonical")?,
                        second_alias: r.try_get_opt_string("second_alias")?.is_some(),
                        configuration: if r.try_get_opt_string("name")?.is_some() {
                            for name in [
                                "name",
                                "upstream_json",
                                "allowlist",
                                "blocklist",
                                "settings_json",
                            ] {
                                text(r, name)?;
                            }
                            for name in ["enabled", "preferred"] {
                                int(r, name)?.ok_or_else(|| corrupt("missing domain number"))?;
                            }
                            Some(crate::domain_config::domain(r))
                        } else {
                            None
                        },
                    })
                })();
                (input, value)
            })
            .collect()
    }
    pub fn held_addresses(&self) -> Vec<Result<String, StoreError>> {
        self.rows
            .iter()
            .filter(|r| r.get_string("tag") == "hold")
            .map(|r| {
                check(r)?;
                text(r, "address")
            })
            .collect()
    }
    pub fn exact_flags(&self) -> HashMap<String, Result<String, StoreError>> {
        self.rows
            .iter()
            .filter(|r| r.get_string("tag") == "flag")
            .map(|r| {
                (
                    r.get_string("input"),
                    (|| {
                        check(r)?;
                        text(r, "setting_value")
                    })(),
                )
            })
            .collect()
    }
    pub fn zero_access(&self) -> Result<bool, StoreError> {
        Ok(self
            .one("zero")?
            .map(|r| r.get_i64("enabled") != 0)
            .unwrap_or(false))
    }
    pub fn restrictions(&self) -> Result<Vec<crate::privacy::RestrictionRow>, StoreError> {
        let rows = self
            .rows
            .iter()
            .filter(|r| r.get_string("tag") == "restriction")
            .collect::<Vec<_>>();
        if rows.len() > 64 {
            return Err(corrupt("restriction count requires fallback"));
        }
        rows.into_iter()
            .map(|r| {
                check(r)?;
                for name in [
                    "id",
                    "account_id",
                    "address",
                    "domain",
                    "scopes_json",
                    "mail_delivery",
                    "reason",
                    "source",
                    "created_by",
                ] {
                    text(r, name)?;
                }
                for name in ["request_id", "lifted_by", "lift_reason"] {
                    r.try_get_opt_string(name)?;
                }
                int(r, "created_at")?.ok_or_else(|| corrupt("missing restriction timestamp"))?;
                for name in ["expires_at", "lifted_at"] {
                    int(r, name)?;
                }
                Ok(crate::privacy::restriction(r))
            })
            .collect()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Text,
    Int,
    Blob,
}
struct Field {
    name: &'static str,
    expr: String,
    kind: Kind,
    nullable: bool,
}
fn t(name: &'static str, expr: &str) -> Field {
    Field {
        name,
        expr: expr.into(),
        kind: Kind::Text,
        nullable: false,
    }
}
fn n(name: &'static str, expr: &str) -> Field {
    Field {
        name,
        expr: expr.into(),
        kind: Kind::Int,
        nullable: false,
    }
}
fn optional(mut f: Field) -> Field {
    f.nullable = true;
    f
}
struct Section {
    tag: &'static str,
    from: String,
    input: String,
    order: String,
    fields: Vec<Field>,
}
fn section(tag: &'static str, from: &str, fields: Vec<Field>) -> Section {
    Section {
        tag,
        from: from.into(),
        input: "NULL".into(),
        order: "NULL".into(),
        fields,
    }
}
fn sql(dialect: Dialect) -> &'static str {
    static SQLITE: OnceLock<String> = OnceLock::new();
    static PG: OnceLock<String> = OnceLock::new();
    match dialect {
        Dialect::Sqlite => SQLITE.get_or_init(|| build_sql(false)),
        Dialect::Postgres => PG.get_or_init(|| build_sql(true)),
    }
}
fn build_sql(pg: bool) -> String {
    let session = "FROM sessions s WHERE s.id=?1 AND s.linked_internal=0";
    let owned = "FROM sessions s JOIN accounts a ON a.id=s.account_id WHERE s.id=?1 AND s.linked_internal=0";
    let mut sections = vec![
        section(
            "revision",
            "FROM (SELECT 1) seed LEFT JOIN settings_revision r ON r.id=1",
            vec![optional(n("revision", "r.n"))],
        ),
        section(
            "session",
            session,
            vec![
                t("id", "s.id"),
                t("account_id", "s.account_id"),
                t("username", "s.username"),
                t("jmap_url", "s.jmap_url"),
                t("api_url", "s.api_url"),
                t("authentication_method", "s.authentication_method"),
                Field {
                    name: "sealed_creds",
                    expr: "s.sealed_creds".into(),
                    kind: Kind::Blob,
                    nullable: false,
                },
                optional(t(
                    "last_seen",
                    if pg {
                        "CASE WHEN octet_length(s.last_seen)<=1024 THEN s.last_seen ELSE NULL END"
                    } else {
                        "CASE WHEN typeof(s.last_seen)='text' AND octet_length(s.last_seen)<=1024 THEN s.last_seen ELSE NULL END"
                    },
                )),
            ],
        ),
        section(
            "clock",
            session,
            vec![
                t("clock_created_at", "s.created_at"),
                t("activity_at", "COALESCE(s.activity_at,s.created_at)"),
            ],
        ),
        section(
            "account",
            owned,
            vec![
                t("id", "a.id"),
                t(
                    "kind",
                    "CASE WHEN a.backend_kind='ews' THEN 'ews' ELSE a.kind END",
                ),
                t("host", "a.host"),
                n("port", "a.port"),
                t("tls", "a.tls"),
                t("username", "a.username"),
                t("sync_policy_json", "a.sync_policy_json"),
            ],
        ),
        section(
            "linked",
            session,
            vec![n(
                "required",
                "CAST(CASE WHEN EXISTS(SELECT 1 FROM linked_account l WHERE l.account_id=s.account_id) THEN 1 ELSE 0 END AS BIGINT)",
            )],
        ),
        section(
            "names",
            "FROM sessions s JOIN settings x ON x.key=('v6:account:names:'||s.account_id) WHERE s.id=?1 AND s.linked_internal=0",
            vec![t("setting_value", "x.value")],
        ),
        section(
            "principal",
            "FROM sessions s JOIN session_principal sp ON sp.session_id=s.id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id WHERE s.id=?1 AND s.linked_internal=0",
            vec![
                t("primary_account_id", "sp.primary_account_id"),
                t("principal_id", "sp.principal_id"),
                t("binding_id", "sp.binding_id"),
                t("binding_principal_id", "b.principal_id"),
                t("state", "p.state"),
                t("binding_kind", "b.kind"),
                n("auth_time", "sp.auth_time"),
                t("principal_auth_method", "sp.authentication_method"),
                optional(n("retired_at", "b.retired_at")),
            ],
        ),
        section(
            "zero",
            "FROM sessions s JOIN zeroaccess_accounts z ON z.account_id=s.account_id WHERE s.id=?1 AND s.linked_internal=0",
            vec![n("enabled", "z.enabled")],
        ),
    ];
    let mut domains = section(
        "domain",
        "FROM domains_in i LEFT JOIN domain_alias al ON al.alias=i.name LEFT JOIN domains d ON d.name=COALESCE(al.domain,i.name) LEFT JOIN domain_alias al2 ON al2.alias=COALESCE(al.domain,i.name)",
        vec![
            t("canonical", "COALESCE(al.domain,i.name)"),
            optional(t("second_alias", "al2.alias")),
            optional(t("name", "d.name")),
            optional(t("upstream_json", "d.upstream_json")),
            optional(t("allowlist", "d.allowlist")),
            optional(t("blocklist", "d.blocklist")),
            optional(t("settings_json", "d.settings_json")),
            optional(n("enabled", "d.enabled")),
            optional(n("preferred", "d.preferred")),
            optional(n("updated_at", "d.updated_at")),
        ],
    );
    domains.input = "i.name".into();
    sections.push(domains);
    let mut holds = section("hold", "FROM held h", vec![t("address", "h.address")]);
    holds.order = "h.address".into();
    sections.push(holds);
    let mut flags = section(
        "flag",
        "FROM settings x JOIN keys_in k ON k.name=x.key",
        vec![t("setting_value", "x.value")],
    );
    flags.input = "k.name".into();
    sections.push(flags);
    sections.push(section(
        "restriction",
        "FROM restricted rr JOIN account_restriction x ON x.id=rr.id",
        vec![
            t("id", "x.id"),
            t("account_id", "x.account_id"),
            t("address", "x.address"),
            t("domain", "x.domain"),
            t("scopes_json", "x.scopes_json"),
            t("mail_delivery", "x.mail_delivery"),
            t("reason", "x.reason"),
            t("source", "x.source"),
            optional(t("request_id", "x.request_id")),
            t("created_by", "x.created_by"),
            n("created_at", "x.created_at"),
            optional(n("expires_at", "x.expires_at")),
            optional(n("lifted_at", "x.lifted_at")),
            optional(t("lifted_by", "x.lifted_by")),
            optional(t("lift_reason", "x.lift_reason")),
        ],
    ));
    let mut columns = BTreeMap::new();
    for s in &sections {
        for f in &s.fields {
            columns.insert(f.name, f.kind);
        }
    }
    let metadata = sections
        .iter()
        .map(|s| {
            let weights = s
                .fields
                .iter()
                .map(|f| match f.kind {
                    Kind::Text => format!("6*CAST(COALESCE(octet_length({}),0) AS BIGINT)", f.expr),
                    Kind::Blob => format!("CAST(COALESCE(octet_length({}),0) AS BIGINT)", f.expr),
                    Kind::Int => "8".into(),
                })
                .collect::<Vec<_>>()
                .join("+");
            format!("SELECT COALESCE(SUM({weights}+1024),0) AS bytes {}", s.from)
        })
        .collect::<Vec<_>>()
        .join(" UNION ALL ");
    let frames=sections.iter().map(|s|{
 let shapes=if pg{"1=1".into()}else{s.fields.iter().map(|f|{let kind=match f.kind{Kind::Text=>"text",Kind::Int=>"integer",Kind::Blob=>"blob"};format!("typeof({}) IN ('{kind}'{})",f.expr,if f.nullable{",'null'"}else{""})}).collect::<Vec<_>>().join(" AND ")};
 let guard=format!("(SELECT ok FROM budget)=1 AND ({shapes})");
 let values=columns.iter().map(|(name,kind)|{let ty=match kind{Kind::Text=>"TEXT",Kind::Int=>"BIGINT",Kind::Blob=>if pg{"BYTEA"}else{"BLOB"}};match s.fields.iter().find(|f|f.name==*name){Some(f)=>format!("CASE WHEN {guard} THEN {} ELSE CAST(NULL AS {ty}) END AS {name}",f.expr),None=>format!("CAST(NULL AS {ty}) AS {name}")}}).collect::<Vec<_>>().join(",");
 format!("SELECT '{}' AS tag,CAST({} AS TEXT) AS input,CAST(CASE WHEN {guard} THEN 0 ELSE 1 END AS BIGINT) AS invalid,{},{} AS hold_sort {}",s.tag,s.input,values,s.order,s.from)
 }).collect::<Vec<_>>().join(" UNION ALL ");
    let inputs = if pg {
        "domains_in AS (SELECT value AS name FROM json_array_elements_text(?2::json)),keys_in AS (SELECT value AS name FROM json_array_elements_text(?3::json))"
    } else {
        "domains_in AS (SELECT CAST(value AS TEXT) AS name FROM json_each(?2)),keys_in AS (SELECT CAST(value AS TEXT) AS name FROM json_each(?3))"
    };
    let cols = columns.keys().copied().collect::<Vec<_>>().join(",");
    format!(
        "SELECT tag,input,invalid,{cols} FROM (WITH {inputs},held AS (SELECT address FROM mail_user WHERE status IN ('disabled','deleting','deleted') ORDER BY address LIMIT 5001),restricted AS (SELECT x.id FROM account_restriction x JOIN sessions s ON s.account_id=x.account_id WHERE s.id=?1 AND s.linked_internal=0 AND x.lifted_at IS NULL ORDER BY x.created_at DESC LIMIT 65),sizes AS ({metadata}),budget AS (SELECT CASE WHEN SUM(bytes)<={MAX_PAYLOAD} THEN 1 ELSE 0 END AS ok FROM sizes) {frames}) native_inputs ORDER BY tag,hold_sort"
    )
}
impl Store {
    /// One physical statement; joins/subqueries are NOT free work. This returns
    /// INPUTS ONLY. Caller must keep the original complete auth fallback and place
    /// evaluation after all body/permit awaits. Scope/revision never authorizes.
    pub async fn native_cookie_admission_snapshot(
        &self,
        cookie: &str,
        request: &NativeAdmissionRequest,
    ) -> Result<NativeAdmissionSnapshot, StoreError> {
        if cookie.is_empty() || cookie.len() > 1024 {
            return Err(corrupt("cookie bound"));
        }
        let domains =
            serde_json::to_string(&request.domains).map_err(|_| corrupt("domain encoding"))?;
        let keys = serde_json::to_string(&request.keys).map_err(|_| corrupt("key encoding"))?;
        if domains.len() + keys.len() > MAX_PARAMETERS {
            return Err(corrupt("encoded input bound"));
        }
        let rows = q(sql(self.backend.dialect()))
            .bind(cookie)
            .bind(domains)
            .bind(keys)
            .fetch_all(&self.backend)
            .await?;
        if rows.len() > MAX_ROWS {
            return Err(corrupt("row count"));
        }
        Ok(NativeAdmissionSnapshot {
            scope: self.cache_scope(),
            rows,
            requested: request.clone(),
        })
    }
}
#[cfg(test)]
#[path = "native_admission_snapshot_tests.rs"]
mod tests;
