//! Source versions for prepared mail DATA. No caller authority is cached here.
//! The final consumer must obtain this observation inside its final joint auth
//! statement; a standalone preparation snapshot is not a dispatch lease.
use crate::{Store, StoreError, backend::Dialect, q};
use serde::Deserialize;
use std::{collections::BTreeSet, path::PathBuf, sync::LazyLock};
#[path="mail_data_schema.rs"]
mod schema;
#[cfg(test)]
#[path="mail_data_publication_tests.rs"]
mod tests;

pub const MAIL_DATA_SCHEMA: u64 = 1;
pub const MAX_PREPARED_IDS: usize = 50;
pub const MAX_MAIL_FENCE_BYTES: usize = 64 * 1024;

#[derive(Clone,Copy,Debug,Default)]
pub struct MailDataSeedOutcome {pub account_rows:u64,pub message_rows:u64}
pub struct MailDataInvalidation {scope:uuid::Uuid,account:String,revision:String,zeroaccess:bool}
impl MailDataInvalidation {
    pub fn account(&self)->&str{&self.account}
    pub fn zeroaccess(&self)->bool{self.zeroaccess}
}
impl std::fmt::Debug for MailDataInvalidation {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("MailDataInvalidation([redacted])")}
}

#[derive(Clone)]
pub struct MailDataDescriptor {
    scope: uuid::Uuid,
    account: String,
    ids: Vec<String>,
}
impl std::fmt::Debug for MailDataDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailDataDescriptor").field("ids", &self.ids.len()).finish_non_exhaustive()
    }
}
impl MailDataDescriptor {
    pub fn account(&self) -> &str { &self.account }
    pub fn ids(&self) -> &[String] { &self.ids }
    pub(crate) fn input_json(&self) -> String {
        serde_json::to_string(&self.ids).expect("bounded owned strings")
    }
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
struct RowVersion {
    id: String,
    owned: i64,
    epoch: Option<String>,
    cache: Option<String>,
    content: Option<String>,
    light: i64,
}
#[derive(Clone, Deserialize)]
struct Payload {
    schema: u64,
    proof: Option<String>,
    incarnation: String,
    focus: String,
    policy: String,
    zeroaccess: i64,
    rows: Vec<RowVersion>,
    email: u64,
    mailbox: u64,
    submission: u64,
    pim: u64,
    crypto: u64,
}
#[derive(Clone)]
pub struct MailDataFence {
    descriptor: MailDataDescriptor,
    generation: Option<PathBuf>,
    payload: Payload,
}
impl std::fmt::Debug for MailDataFence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailDataFence").field("rows", &self.payload.rows.len())
            .field("standard", &self.standard()).finish_non_exhaustive()
    }
}
impl MailDataFence {
    pub fn descriptor(&self) -> &MailDataDescriptor { &self.descriptor }
    pub fn standard(&self) -> bool { self.payload.zeroaccess == 0 }
    pub fn schema_stamp(&self)->Option<&str>{self.payload.proof.as_deref()}
    pub fn same_schema_generation(&self,other:&Self)->bool {
        self.descriptor.scope==other.descriptor.scope && self.generation==other.generation
            && self.payload.proof.is_some() && self.payload.proof==other.payload.proof
    }
    pub fn email_state(&self) -> String { self.payload.email.to_string() }
    pub fn session_state(&self) -> String {
        let p=&self.payload;
        format!("e{}m{}s{}p{}c{}",p.email,p.mailbox,p.submission,p.pim,p.crypto)
    }
    pub fn retained_data_bytes(&self)->usize {
        std::mem::size_of::<Self>()+self.descriptor.account.capacity()
            +self.descriptor.ids.capacity()*std::mem::size_of::<String>()
            +self.descriptor.ids.iter().map(String::capacity).sum::<usize>()
            +self.payload.incarnation.capacity()+self.payload.focus.capacity()+self.payload.policy.capacity()
            +self.payload.proof.as_ref().map_or(0,String::capacity)
            +self.payload.rows.capacity()*std::mem::size_of::<RowVersion>()
            +self.payload.rows.iter().map(|row|row.id.capacity()
                +row.epoch.as_ref().map_or(0,String::capacity)
                +row.cache.as_ref().map_or(0,String::capacity)
                +row.content.as_ref().map_or(0,String::capacity)).sum::<usize>()
            +self.generation.as_ref().map_or(0,PathBuf::capacity)
    }
    /// State counters deliberately are not a cache identity: unrelated method
    /// state changes use the NEW final envelope, not a cached old state string.
    pub fn same_data(&self, other: &Self) -> bool {
        self.descriptor.scope == other.descriptor.scope
            && self.descriptor.account == other.descriptor.account
            && self.descriptor.ids == other.descriptor.ids
            && self.generation == other.generation
            && self.payload.schema == other.payload.schema
            && self.payload.proof == other.payload.proof
            && self.payload.incarnation == other.payload.incarnation
            && self.payload.focus == other.payload.focus
            && self.payload.policy == other.payload.policy
            && self.payload.zeroaccess == other.payload.zeroaccess
            && self.payload.rows == other.payload.rows
    }
    pub fn publishable(&self) -> bool {
        self.standard() && self.payload.proof.as_ref().is_some_and(|stamp|!stamp.is_empty() && stamp.len()<=128)
            && self.payload.rows.iter().all(|row|
            match (&row.epoch, &row.cache, &row.content) {
                (None,None,None) => row.owned==0 && row.light == 0,
                (Some(epoch),Some(cache),Some(content)) => row.owned==1 && nonce(epoch)
                    && cache.len()<=128 && content.len()<=128 && row.light == 1,
                _ => false,
            })
    }
    /// Unsupported schemas/legacy projections are not reasons for a no-op
    /// writer transaction. Only missing source-owned derived rows need seed.
    pub fn needs_seed(&self)->bool {
        self.standard() && self.payload.rows.iter().any(|row|row.owned==1 && row.epoch.is_none())
    }
    pub fn matches_owned_cache_revision(&self,id:&str,revision:&str)->bool {
        self.payload.rows.iter().any(|row|row.id==id && row.owned==1 && row.cache.as_deref()==Some(revision))
    }
}
fn nonce(value:&str)->bool {
    value.len()==32 && value.bytes().all(|c|c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn corrupt()->StoreError { StoreError::Corrupt("invalid prepared mail data witness".into()) }

impl Store {
    /// Coalesced invalidation hints only. ACK never certifies all RAM or a user
    /// authorization; EVERY successful result still needs a final joint fence.
    pub async fn mail_data_invalidations(&self,limit:u32)->Result<Vec<MailDataInvalidation>,StoreError>{
        if !(1..=32).contains(&limit){return Err(StoreError::Invalid("mail invalidation limit"))}
        let rows=q("SELECT CASE WHEN octet_length(o.account_id)<=1024 THEN o.account_id ELSE NULL END AS account_id,CASE WHEN octet_length(o.revision)<=32 THEN o.revision ELSE NULL END AS revision,COALESCE(z.enabled,0) AS zeroaccess FROM mail_data_outbox o JOIN accounts a ON a.id=o.account_id LEFT JOIN zeroaccess_accounts z ON z.account_id=a.id ORDER BY o.account_id LIMIT ?1").bind(limit as i64).fetch_all(&self.backend).await?;
        rows.into_iter().map(|row|{
            let account=row.try_get_opt_string("account_id")?.filter(|v|!v.is_empty()).ok_or_else(corrupt)?;
            let revision=row.try_get_opt_string("revision")?.filter(|v|nonce(v)).ok_or_else(corrupt)?;
            let zeroaccess=row.get_i64("zeroaccess");if !matches!(zeroaccess,0|1){return Err(corrupt())}
            Ok(MailDataInvalidation{scope:self.cache_scope(),account,revision,zeroaccess:zeroaccess==1})
        }).collect()
    }
    pub async fn acknowledge_mail_data_invalidation(&self,hint:&MailDataInvalidation)->Result<bool,StoreError>{
        if hint.scope!=self.cache_scope(){return Err(StoreError::Invalid("foreign mail invalidation scope"))}
        Ok(q("DELETE FROM mail_data_outbox WHERE account_id=?1 AND revision=?2").bind(&hint.account).bind(&hint.revision).execute(&self.backend).await?==1)
    }
    pub fn mail_data_descriptor(&self, account:&str, ids:&[String]) -> Result<MailDataDescriptor,StoreError> {
        if account.is_empty() || account.len()>1024 || ids.len()>MAX_PREPARED_IDS
            || ids.iter().any(|id|id.is_empty() || id.len()>1024)
        { return Err(StoreError::Invalid("prepared mail input limit exceeded")); }
        let ids=ids.iter().cloned().collect::<BTreeSet<_>>().into_iter().collect();
        Ok(MailDataDescriptor {scope:self.cache_scope(),account:account.into(),ids})
    }

    /// Lazy bounded seed only; no startup mailbox backfill. Source triggers
    /// maintain epochs afterward, even for independent raw SQL writers.
    pub async fn seed_mail_data(&self, descriptor:&MailDataDescriptor)->Result<MailDataSeedOutcome,StoreError> {
        self.check_mail_descriptor(descriptor)?;
        // Parent-first, SHORT metadata transaction. SQLite Backend::begin uses
        // BEGIN IMMEDIATE; PG parent lock prevents an orphan seed after erasure.
        // Commit before any preparation/index/HTTP authorization awaits.
        let mut tx=self.backend.begin().await?;
        let lock=match self.backend.dialect(){
            Dialect::Sqlite=>"SELECT id FROM accounts WHERE id=?1",
            Dialect::Postgres=>"SELECT id FROM accounts WHERE id=?1 FOR NO KEY UPDATE",
        };
        if q(lock).bind(&descriptor.account).fetch_all_tx(&mut tx).await?.is_empty(){
            tx.commit().await?;return Ok(MailDataSeedOutcome::default())
        }
        static ACCOUNT_SQLITE:LazyLock<String>=LazyLock::new(||seed_account("lower(hex(randomblob(16)))"));
        static ACCOUNT_PG:LazyLock<String>=LazyLock::new(||seed_account("md5(random()::text||clock_timestamp()::text)"));
        let sql=match self.backend.dialect(){Dialect::Sqlite=>ACCOUNT_SQLITE.as_str(),Dialect::Postgres=>ACCOUNT_PG.as_str()};
        let account_rows=q(sql).bind(&descriptor.account).execute_tx(&mut tx).await?;
        static ROW_SQLITE:LazyLock<String>=LazyLock::new(||seed_rows(Dialect::Sqlite));
        static ROW_PG:LazyLock<String>=LazyLock::new(||seed_rows(Dialect::Postgres));
        let sql=match self.backend.dialect(){Dialect::Sqlite=>ROW_SQLITE.as_str(),Dialect::Postgres=>ROW_PG.as_str()};
        let message_rows=q(sql).bind(&descriptor.account).bind(descriptor.input_json()).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(MailDataSeedOutcome {account_rows,message_rows})
    }

    /// Preparation-only observation. Successful HTTP emission must instead
    /// receive the same typed payload from the LAST JOINT auth/data statement.
    pub async fn mail_data_snapshot(&self, descriptor:&MailDataDescriptor)->Result<Option<MailDataFence>,StoreError> {
        self.check_mail_descriptor(descriptor)?;
        let before=self.publication_generation().await;
        static SQLITE:LazyLock<String>=LazyLock::new(||format!("SELECT {} AS mail_data",joined_mail_data_sql(Dialect::Sqlite,"?1","?2")));
        static PG:LazyLock<String>=LazyLock::new(||format!("SELECT {} AS mail_data",joined_mail_data_sql(Dialect::Postgres,"?1","?2")));
        let sql=match self.backend.dialect(){Dialect::Sqlite=>SQLITE.as_str(),Dialect::Postgres=>PG.as_str()};
        let row=q(sql).bind(&descriptor.account).bind(descriptor.input_json()).fetch_one(&self.backend).await?;
        if before!=self.publication_generation().await {return Err(StoreError::SourceChanged)}
        self.mail_data_from_joined(descriptor,before,row.try_get_opt_string("mail_data")?.as_deref())
    }

    fn check_mail_descriptor(&self, descriptor:&MailDataDescriptor)->Result<(),StoreError> {
        if descriptor.scope!=self.cache_scope(){return Err(StoreError::Invalid("foreign prepared mail scope"))}
        Ok(())
    }

    /// Crate-private factory: only the Store's real query observation can mint
    /// a final fence. `generation` comes from the final auth query's pinned
    /// serving-generation witness; callers must not add an await to obtain it.
    pub(crate) fn mail_data_from_joined(&self, descriptor:&MailDataDescriptor, generation:Option<PathBuf>, json:Option<&str>)->Result<Option<MailDataFence>,StoreError> {
        self.check_mail_descriptor(descriptor)?;
        let Some(json)=json else{return Ok(None)};
        if json.len()>MAX_MAIL_FENCE_BYTES{return Err(corrupt())}
        let mut payload:Payload=serde_json::from_str(json).map_err(|_|corrupt())?;
        if payload.schema!=MAIL_DATA_SCHEMA || !nonce(&payload.incarnation)
            || !nonce(&payload.focus) || !nonce(&payload.policy)
            || !matches!(payload.zeroaccess,0|1) || payload.rows.len()!=descriptor.ids.len()
        {return Err(corrupt())}
        payload.rows.sort_by(|a,b|a.id.cmp(&b.id));
        if payload.rows.iter().map(|row|&row.id).ne(descriptor.ids.iter()) {return Err(corrupt())}
        Ok(Some(MailDataFence {descriptor:descriptor.clone(),generation,payload}))
    }
}

fn seed_account(random:&str)->String {
    format!("INSERT INTO mail_data_accounts(account_id,incarnation,focus_revision,policy_revision) SELECT id,{random},{random},{random} FROM accounts WHERE id=?1 ON CONFLICT(account_id) DO NOTHING")
}
fn seed_rows(dialect:Dialect)->String {
    let (random,ids)=match dialect {
        Dialect::Sqlite=>("lower(hex(randomblob(16)))","SELECT value FROM json_each(?2)"),
        Dialect::Postgres=>("md5(random()::text||clock_timestamp()::text)","SELECT value::text FROM json_array_elements_text(?2::json)"),
    };
    format!("INSERT INTO mail_data_rows(stable_id,account_id,data_revision,cache_revision,content_revision) SELECT stable_id,account_id,{random},cache_revision,content_revision FROM messages WHERE account_id=?1 AND stable_id IN({ids}) ON CONFLICT(stable_id) DO NOTHING")
}

/// Fixed source SQL fragments only. Both arguments are authored SQL scalar
/// expressions/bind references, NEVER strings supplied by an HTTP caller.
/// Native authority-fence integration must bind the account to its real session
/// source row and enforce descriptor account equality before calling factory.
pub(crate) fn joined_mail_data_sql(dialect:Dialect, account:&'static str, ids:&'static str)->String {
    let (object,input,aggregate,empty,json_wrap)=match dialect {
        Dialect::Sqlite=>("json_object",format!("SELECT value AS id FROM json_each({ids})"),"json_group_array","'[]'",true),
        Dialect::Postgres=>("json_build_object",format!("SELECT value::text AS id FROM json_array_elements_text({ids}::json)"),"json_agg","'[]'::json",false),
    };
    // Bound every source scalar BEFORE constructing/transferring its JSON.
    let bounded=|column:&str,limit:usize|format!("CASE WHEN octet_length({column})<={limit} THEN {column} ELSE NULL END");
    // Actual source ownership/liveness uses only the messages record's EARLY
    // PK/account prefix. Revisions live in the compact transactionally updated
    // tuple, avoiding late-column traversal behind a multi-MiB envelope BLOB.
    let row=format!("{object}('id',r.id,'owned',CASE WHEN m.stable_id IS NULL THEN 0 ELSE 1 END,'epoch',{},'cache',{},'content',{},'light',CASE WHEN h.sealed_envelope IS NOT NULL AND octet_length(h.sealed_envelope)<={} THEN 1 ELSE 0 END)",bounded("v.data_revision",32),bounded("v.cache_revision",128),bounded("v.content_revision",128),crate::header_envelope::MAX_HEADER_ENVELOPE_BYTES+44);
    let rows=format!("(SELECT COALESCE({aggregate}({row}),{empty}) FROM ({input}) r LEFT JOIN messages m ON m.stable_id=r.id AND m.account_id={account} LEFT JOIN mail_data_rows v ON v.stable_id=m.stable_id AND v.account_id=m.account_id LEFT JOIN message_list_envelope h ON h.stable_id=m.stable_id AND h.account_id=m.account_id)");
    let rows=if json_wrap {format!("json({rows})")}else{rows};
    let core=|kind:&str|format!("(SELECT COALESCE(MAX(state),0) FROM (SELECT MAX(state) AS state FROM changes WHERE account_id={account} AND type='{kind}' UNION ALL SELECT head_state AS state FROM change_log_checkpoint WHERE account_id={account} AND type='{kind}') heads)");
    let sum=|table:&str,kinds:&str|format!("(SELECT COALESCE(SUM(state),0) FROM (SELECT MAX(state) AS state FROM {table} WHERE account_id={account} AND type IN({kinds}) GROUP BY type) heads)");
    let pim=sum("pim_changes","'Calendar','CalendarEvent','Task','Note','AddressBook','ContactCard','ContactGroup'");
    let crypto=sum("crypto_changes","'CryptoKey','MailRule'");
    let expression=format!("(SELECT {object}('schema',{MAIL_DATA_SCHEMA},'proof',{},'incarnation',{},'focus',{},'policy',{},'zeroaccess',COALESCE(z.enabled,0),'rows',{rows},'email',{},'mailbox',{},'submission',{},'pim',{pim},'crypto',{crypto}) FROM accounts a JOIN mail_data_accounts d ON d.account_id=a.id LEFT JOIN zeroaccess_accounts z ON z.account_id=a.id WHERE a.id={account})",schema::stamp_expression(dialect),bounded("d.incarnation",32),bounded("d.focus_revision",32),bounded("d.policy_revision",32),core("Email"),core("Mailbox"),core("EmailSubmission"));
    let expression=match dialect {Dialect::Sqlite=>expression,Dialect::Postgres=>format!("{expression}::text")};
    format!("(WITH mail_data_payload AS MATERIALIZED (SELECT {expression} AS value) SELECT CASE WHEN octet_length(value)<={MAX_MAIL_FENCE_BYTES} THEN value ELSE NULL END FROM mail_data_payload)")
}
