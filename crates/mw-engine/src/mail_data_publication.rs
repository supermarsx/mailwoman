//! Prepared default36 result DATA, with request-local authority/envelope.
//! `finish` is synchronous. The caller must pass a Store-minted observation
//! from its LAST JOINT auth+data query, never a prior preparation snapshot.
use crate::{Engine, index_publication::ProjectionObservation, v6::{AccountPostureSource,PreparedMailPosture}};
use mw_store::mail_data_publication::{MailDataDescriptor,MailDataFence};
use serde_json::{Value,json};
use std::{collections::{BTreeMap,BTreeSet,HashMap}, io::{self,Write}, sync::{Arc,Mutex,atomic::{AtomicBool,AtomicU64,Ordering}}, time::{Duration,Instant}};
use tokio::sync::{OwnedSemaphorePermit,Semaphore};
#[cfg(test)]
#[path="mail_data_publication_tests.rs"]
mod tests;

// KiB permits keep the finite16GiB ceiling portable even on32-bit builds.
// Rounding UP each retained object deliberately overcharges, never undercounts.
const MAX_TOKENS:usize=16*1024*1024;
const BUILD_RESERVATION:usize=9*1024*1024;
const MAX_OUTPUT:usize=8*1024*1024;
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Deserialize,serde::Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PreparedMailOptions {
    pub enabled:bool,
    #[serde(rename="budgetMiB")]
    pub budget_mib:u32,
    pub max_entries:usize,
    pub max_accounts:usize,
    pub idle_seconds:u32,
}
// The endpoint must explicitly apply a policy-fenced authorized config before
// enabling this new path. Unintegrated constructors keep ordinary reads.
impl Default for PreparedMailOptions {fn default()->Self{Self {enabled:false,budget_mib:64,max_entries:256,max_accounts:128,idle_seconds:30}}}
impl PreparedMailOptions {
    pub fn valid(&self)->bool {(16..=16384).contains(&self.budget_mib) && (1..=100000).contains(&self.max_entries) && (1..=10000).contains(&self.max_accounts) && (1..=3600).contains(&self.idle_seconds)}
    fn budget_bytes(&self)->u64{u64::from(self.budget_mib)*1024*1024}
}
fn tokens(bytes:usize)->usize {bytes.div_ceil(1024)}
const PROPERTIES:[&str;36]=[
    "id","blobId","from","to","subject","receivedAt","preview","keywords","mailboxIds","threadId",
    "pinned","snoozedUntil","followUpAt","hasAttachment","size","mailwomanFocus","mailwomanAuth","mailwomanPriority","mailwomanMentions",
    "cc","bcc","replyTo","sentAt","messageId","inReplyTo","references",
    "header:Delivered-To:asAddresses:all","header:X-Original-To:asAddresses:all","header:Envelope-To:asAddresses:all","header:X-Forwarded-To:asAddresses:all","header:Resent-To:asAddresses:all",
    "header:Expires:asText","header:Expiry-Date:asText","header:Reply-By:asText","header:Sensitivity:asText","header:Supersedes:asText",
];

#[derive(Clone,PartialEq,Eq,Hash)]
struct Key {account:String,ids:Vec<String>}
struct Row {encoded:Box<[u8]>}
struct Ready {
    config_epoch:u64,
    source:MailDataFence,
    projection:ProjectionObservation,
    posture:Arc<dyn AccountPostureSource>,
    rows:BTreeMap<String,Row>,
    // The byte charge follows held/evicted Arcs until the LAST reader drops it.
    _charge:OwnedSemaphorePermit,
}
struct Cell {ready:Arc<Ready>,used:Instant,invalidated:bool}
struct Memory {entries:HashMap<Key,Cell>,pending:BTreeSet<(String,Vec<String>)>,validated:Option<MailDataFence>,config:PreparedMailOptions,config_epoch:u64,config_revision:Option<i64>}
pub(crate) struct PreparedMail {
    memory:Mutex<Memory>,
    bytes:Arc<Semaphore>,
    jobs:Arc<Semaphore>,
    workers:Arc<Semaphore>,
    stats:Stats,
    background_started:AtomicBool,
}
#[derive(Default)]
struct Stats {eligible:AtomicU64,hits:AtomicU64,misses:AtomicU64,builds:AtomicU64,evictions:AtomicU64,seed_calls:AtomicU64,seed_writes:AtomicU64,raw_results:AtomicU64,value_results:AtomicU64,hints:AtomicU64,hint_errors:AtomicU64,hint_reads:AtomicU64,hint_acks:AtomicU64}
#[derive(serde::Serialize)]
#[serde(rename_all="camelCase")]
pub struct PreparedMailStatus {
    pub entries:Option<usize>,pub accounts:Option<usize>,pub pending_builds:Option<usize>,pub invalidated_entries:Option<usize>,
    pub charged_bytes:u64,pub admitted_jobs:usize,pub active_preparers:usize,
    pub budget_bytes:Option<u64>,pub max_entries:Option<usize>,pub max_accounts:Option<usize>,
    pub enabled:Option<bool>,pub idle_seconds:Option<u32>,
    pub eligible_candidates:u64,pub ready_candidate_hits:u64,pub candidate_misses:u64,
    pub completed_builds:u64,pub evictions:u64,pub seed_calls:u64,pub seeded_rows:u64,
    pub raw_results:u64,pub compatibility_value_results:u64,
    pub handled_invalidation_hints:u64,pub invalidation_errors:u64,
    pub invalidation_poll_calls:u64,pub invalidation_ack_calls:u64,
}
impl Default for PreparedMail {
    fn default()->Self {
        Self {memory:Mutex::new(Memory {entries:HashMap::new(),pending:BTreeSet::new(),validated:None,config:Default::default(),config_epoch:0,config_revision:None}),
            bytes:Arc::new(Semaphore::new(MAX_TOKENS)),jobs:Arc::new(Semaphore::new(16)),workers:Arc::new(Semaphore::new(2)),stats:Default::default(),background_started:AtomicBool::new(false)}
    }
}

/// Opaque DATA candidate. Holding it never authorizes emission or dispatch.
pub struct PreparedMetadataCandidate {ready:Arc<Ready>,requested:Vec<String>}
impl std::fmt::Debug for PreparedMetadataCandidate {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        f.debug_struct("PreparedMetadataCandidate").field("ids",&self.requested.len()).finish_non_exhaustive()
    }
}
impl PreparedMetadataCandidate {
    pub fn descriptor(&self)->&MailDataDescriptor {self.ready.source.descriptor()}
}
/// Already encoded public JMAP body only; contains mail and must never be logged.
pub struct PreparedMetadataBytes(Vec<u8>);
impl PreparedMetadataBytes {pub fn into_bytes(self)->Vec<u8>{self.0}}
impl std::fmt::Debug for PreparedMetadataBytes {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        f.debug_struct("PreparedMetadataBytes").field("bytes",&self.0.len()).finish()
    }
}

struct Pending<'a>{service:&'a PreparedMail,key:(String,Vec<String>)}
struct Probe<'a>{stats:&'a Stats,hit:bool}
struct Finish<'a>{service:&'a PreparedMail,candidate:&'a PreparedMetadataCandidate,success:bool}
impl Drop for Finish<'_>{
    fn drop(&mut self){if !self.success{self.service.evict_same(&candidate_key(self.candidate),&self.candidate.ready);}}
}
fn candidate_key(candidate:&PreparedMetadataCandidate)->Key {
    let descriptor=candidate.descriptor();Key {account:descriptor.account().into(),ids:descriptor.ids().to_vec()}
}
impl Drop for Probe<'_>{fn drop(&mut self){if !self.hit{self.stats.misses.fetch_add(1,Ordering::Relaxed);}}}
impl Drop for Pending<'_> {
    fn drop(&mut self){if let Ok(mut memory)=self.service.memory.lock(){memory.pending.remove(&self.key);}}
}
impl PreparedMail {
    fn start(self:&Arc<Self>,store:mw_store::Store){
        let Ok(runtime)=tokio::runtime::Handle::try_current() else{return};
        if self.background_started.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_err(){return}
        let weak=Arc::downgrade(self);
        runtime.spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let Some(service)=weak.upgrade() else{break};
                if let Ok(mut memory)=service.memory.lock(){service.expired(&mut memory);}
                service.stats.hint_reads.fetch_add(1,Ordering::Relaxed);
                match store.mail_data_invalidations(16).await {
                    Ok(hints)=>for hint in hints {
                        if hint.zeroaccess(){service.clear_account(hint.account())}
                        else if let Ok(mut memory)=service.memory.lock(){
                            // Mark, do NOT blindly evict unchanged selected rows.
                            // All hits already cross the fresh final source fence.
                            for (key,cell) in &mut memory.entries {if key.account==hint.account(){cell.invalidated=true;}}
                        }
                        service.stats.hint_acks.fetch_add(1,Ordering::Relaxed);
                        match store.acknowledge_mail_data_invalidation(&hint).await {
                            Ok(true)=>{service.stats.hints.fetch_add(1,Ordering::Relaxed);},
                            Ok(false)=>{}, // Newer event retained by revision CAS.
                            Err(_)=>{service.stats.hint_errors.fetch_add(1,Ordering::Relaxed);},
                        }
                    },
                    Err(_)=>{service.stats.hint_errors.fetch_add(1,Ordering::Relaxed);},
                }
            }
        });
    }
    pub(crate) fn clear_account(&self,account:&str){
        if let Ok(mut memory)=self.memory.lock(){let before=memory.entries.len();memory.entries.retain(|key,_|key.account!=account);self.stats.evictions.fetch_add((before-memory.entries.len()) as u64,Ordering::Relaxed);}
    }
    fn expired(&self,memory:&mut Memory){
        let retain=Duration::from_secs(u64::from(memory.config.idle_seconds));
        let before=memory.entries.len();memory.entries.retain(|_,entry|entry.used.elapsed()<retain);
        self.stats.evictions.fetch_add((before-memory.entries.len()) as u64,Ordering::Relaxed);
    }
    fn get(&self,key:&Key)->Option<Arc<Ready>> {
        let mut memory=self.memory.try_lock().ok()?;
        self.expired(&mut memory);
        let cell=memory.entries.get_mut(key)?;
        cell.used=Instant::now();Some(cell.ready.clone())
    }
    fn recertified(&self,key:&Key,ready:&Arc<Ready>){
        if let Ok(mut memory)=self.memory.lock(){if let Some(cell)=memory.entries.get_mut(key){
            if Arc::ptr_eq(&cell.ready,ready){cell.invalidated=false;}
        }}
    }
    fn evict_same(&self,key:&Key,ready:&Arc<Ready>)->bool {
        let Ok(mut memory)=self.memory.lock() else{return false};
        if !memory.entries.get(key).is_some_and(|cell|Arc::ptr_eq(&cell.ready,ready)){return false}
        memory.entries.remove(key);self.stats.evictions.fetch_add(1,Ordering::Relaxed);true
    }
    fn reserve(&self)->Option<OwnedSemaphorePermit> {
        loop {
            let mut memory=self.memory.try_lock().ok()?;
            if !memory.config.enabled{return None}
            let charged=(MAX_TOKENS-self.bytes.available_permits()) as u64*1024;
            if charged+(tokens(BUILD_RESERVATION)*1024) as u64<=memory.config.budget_bytes() {
                if let Ok(charge)=self.bytes.clone().try_acquire_many_owned(tokens(BUILD_RESERVATION) as u32){return Some(charge)}
            }
            let oldest=memory.entries.iter().min_by_key(|(_,entry)|entry.used).map(|(key,_)|key.clone())?;
            memory.entries.remove(&oldest);
            self.stats.evictions.fetch_add(1,Ordering::Relaxed);
        }
    }
    fn insert(&self,key:Key,ready:Arc<Ready>){
        let Ok(mut memory)=self.memory.lock() else{return};
        if !memory.config.enabled || memory.config_epoch!=ready.config_epoch{return}
        self.expired(&mut memory);
        if memory.entries.insert(key,Cell {ready,used:Instant::now(),invalidated:false}).is_some(){self.stats.evictions.fetch_add(1,Ordering::Relaxed);}
        while memory.entries.len()>memory.config.max_entries
            || memory.entries.keys().map(|k|k.account.as_str()).collect::<BTreeSet<_>>().len()>memory.config.max_accounts
        {
            let Some(oldest)=memory.entries.iter().min_by_key(|(_,entry)|entry.used).map(|(key,_)|key.clone()) else{break};
            memory.entries.remove(&oldest);
            self.stats.evictions.fetch_add(1,Ordering::Relaxed);
        }
    }
}

fn eligible(account:&str,args:&Value)->Option<Vec<String>> {
    let object=args.as_object()?;
    if object.keys().any(|key|!matches!(key.as_str(),"accountId"|"ids"|"properties"|"mailwomanBodyUse"|"fetchHTMLBodyValues"|"fetchTextBodyValues"|"fetchAllBodyValues"))
        || args.get("accountId").is_some_and(|value|value.as_str()!=Some(account))
        || ["mailwomanBodyUse","fetchHTMLBodyValues","fetchTextBodyValues","fetchAllBodyValues"].iter().any(|key|args.get(*key).is_some_and(|value|value.as_bool()!=Some(false)))
    {return None}
    let properties=args.get("properties")?.as_array()?;
    if properties.len()!=PROPERTIES.len(){return None}
    let requested=properties.iter().map(Value::as_str).collect::<Option<BTreeSet<_>>>()?;
    if requested!=PROPERTIES.into_iter().collect(){return None}
    let ids=args.get("ids")?.as_array()?;
    if ids.len()>50{return None}
    ids.iter().map(|value|value.as_str().filter(|id|!id.is_empty() && id.len()<=1024).map(str::to_owned)).collect()
}

fn supported(source:&Arc<dyn AccountPostureSource>,engine:&Engine)->bool {
    match source.prepared_mail_posture(){
        PreparedMailPosture::Unsupported=>false,
        PreparedMailPosture::ConstantStandard=>true,
        PreparedMailPosture::Store(scope)=>scope==*engine.store().cache_scope().as_bytes(),
    }
}

impl Engine {
    /// Apply already-authorized published delivery configuration BEFORE any
    /// preparation/final auth fence. No Store lookup occurs here. The endpoint
    /// must bind this config to its final policy source proof; this setter is
    /// DATA configuration, not authorization. Invalid inputs never apply.
    pub fn configure_prepared_mail(&self,config:PreparedMailOptions)->bool {
        self.configure_prepared_mail_inner(config,None)
    }
    /// The revision orders DATA configuration updates only; it is never an
    /// allow decision. A delayed older request cannot restore a disabled cache
    /// or its former budget after a newer request applied current policy.
    pub fn configure_prepared_mail_at_revision(&self,config:PreparedMailOptions,revision:&mw_store::settings_policy::SettingsReadRevision)->bool {
        if revision.scope()!=self.store().cache_scope() || revision.number().is_none_or(|n|n<0) {return false}
        self.configure_prepared_mail_inner(config,revision.number())
    }
    fn configure_prepared_mail_inner(&self,config:PreparedMailOptions,revision:Option<i64>)->bool {
        if !config.valid(){return false}
        let Ok(mut memory)=self.prepared_mail.memory.lock() else{return false};
        if let Some(previous)=memory.config_revision {
            let Some(next)=revision else{return false};
            // An unrelated account's setting can advance the global counter
            // without invalidating this account's still-current policy DATA.
            // An older identical config is harmless, but never moves the floor.
            if next<previous {return memory.config==config}
            if next==previous && memory.config!=config {return false}
        }
        if let Some(revision)=revision {memory.config_revision=Some(revision);}
        if memory.config==config{return true}
        let Some(epoch)=memory.config_epoch.checked_add(1) else{return false};
        self.prepared_mail.stats.evictions.fetch_add(memory.entries.len() as u64,Ordering::Relaxed);
        memory.entries.clear();memory.config=config;memory.config_epoch=epoch;
        // Held result/build permits stay charged after shrink/disable. reserve
        // cannot admit above the new limit; no permit is manufactured here.
        true
    }
    /// Aggregate cache-owned accounting, not process RSS or an allocator/CPU
    /// sandbox. Charged bytes include borrowed/evicted immutable result Arcs.
    pub fn prepared_mail_status(&self)->PreparedMailStatus {
        let service=&self.prepared_mail;
        let snapshot=service.memory.try_lock().ok().map(|m|((m.entries.len(),m.entries.keys().map(|key|key.account.as_str()).collect::<BTreeSet<_>>().len(),m.pending.len(),m.entries.values().filter(|cell|cell.invalidated).count()),m.config));
        let counts=snapshot.map(|s|s.0);let config=snapshot.map(|s|s.1);
        let s=&service.stats;
        PreparedMailStatus {entries:counts.map(|n|n.0),accounts:counts.map(|n|n.1),pending_builds:counts.map(|n|n.2),invalidated_entries:counts.map(|n|n.3),
            charged_bytes:(MAX_TOKENS-service.bytes.available_permits()) as u64*1024,admitted_jobs:16-service.jobs.available_permits(),active_preparers:2-service.workers.available_permits(),
            budget_bytes:config.map(|c|c.budget_bytes()),max_entries:config.map(|c|c.max_entries),max_accounts:config.map(|c|c.max_accounts),enabled:config.map(|c|c.enabled),idle_seconds:config.map(|c|c.idle_seconds),
            eligible_candidates:s.eligible.load(Ordering::Relaxed),ready_candidate_hits:s.hits.load(Ordering::Relaxed),candidate_misses:s.misses.load(Ordering::Relaxed),completed_builds:s.builds.load(Ordering::Relaxed),evictions:s.evictions.load(Ordering::Relaxed),seed_calls:s.seed_calls.load(Ordering::Relaxed),seeded_rows:s.seed_writes.load(Ordering::Relaxed),raw_results:s.raw_results.load(Ordering::Relaxed),compatibility_value_results:s.value_results.load(Ordering::Relaxed),handled_invalidation_hints:s.hints.load(Ordering::Relaxed),invalidation_errors:s.hint_errors.load(Ordering::Relaxed),invalidation_poll_calls:s.hint_reads.load(Ordering::Relaxed),invalidation_ack_calls:s.hint_acks.load(Ordering::Relaxed)}
    }
    /// Called BEFORE final authorization. A ready lookup performs no source
    /// SQL or Index query: it selects DATA only, which MUST still pass the LAST
    /// joint auth/data fence in finish. Source SQL remains on the cold path.
    /// A cold/malformed/busy candidate retains the complete ordinary dispatch.
    pub async fn prepare_metadata_candidate(&self,account:&str,args:&Value)->Option<PreparedMetadataCandidate> {
        let requested=eligible(account,args)?;
        let config_epoch={let memory=self.prepared_mail.memory.try_lock().ok()?;if !memory.config.enabled{return None}memory.config_epoch};
        self.prepared_mail.stats.eligible.fetch_add(1,Ordering::Relaxed);
        let mut probe=Probe {stats:&self.prepared_mail.stats,hit:false};
        let descriptor=self.store().mail_data_descriptor(account,&requested).ok()?;
        let key=Key {account:account.into(),ids:descriptor.ids().to_vec()};
        let posture=self.v6_hooks().publication_posture_source();
        if !supported(&posture,self){return None}
        let projection=self.index_publisher.projection_proof(account,descriptor.ids())?;
        self.prepared_mail.start(self.store().clone());
        if let Some(ready)=self.prepared_mail.get(&key) {
            // A newer configuration may have installed a different Ready after
            // our initial config observation. Do not evict that newer entry.
            if ready.config_epoch!=config_epoch{return None}
            if self.index_publisher.projection_is_current(account,descriptor.ids(),&ready.projection)
                && Arc::ptr_eq(&posture,&ready.posture)
                && self.prepared_mail.memory.lock().is_ok_and(|memory|memory.config.enabled && memory.config_epoch==config_epoch)
            {probe.hit=true;self.prepared_mail.stats.hits.fetch_add(1,Ordering::Relaxed);return Some(PreparedMetadataCandidate {ready,requested})}
            self.prepared_mail.evict_same(&key,&ready);
        }
        let before=self.store().mail_data_snapshot(&descriptor).await.ok()?;
        // Count is acquired before any waiting/preparation; duplicate warmups
        // are suppressed rather than spawning unbounded per-request tasks.
        let _job=self.prepared_mail.jobs.clone().try_acquire_owned().ok()?;
        let pending_key=(key.account.clone(),key.ids.clone());
        {
            let mut memory=self.prepared_mail.memory.try_lock().ok()?;
            if !memory.pending.insert(pending_key.clone()){return None}
        }
        let _pending=Pending {service:&self.prepared_mail,key:pending_key};
        let _worker=tokio::time::timeout(Duration::from_millis(50),self.prepared_mail.workers.clone().acquire_owned()).await.ok()?.ok()?;
        let mut charge=self.prepared_mail.reserve()?;
        let before=match before {
            Some(before) if !before.needs_seed()=>before,
            _=>{
                self.prepared_mail.stats.seed_calls.fetch_add(1,Ordering::Relaxed);
                let seeded=self.store().seed_mail_data(&descriptor).await.ok()?;
                self.prepared_mail.stats.seed_writes.fetch_add(seeded.account_rows+seeded.message_rows,Ordering::Relaxed);
                self.store().mail_data_snapshot(&descriptor).await.ok()??
            },
        };
        if !before.publishable(){return None}
        let validated=self.prepared_mail.memory.lock().ok()?.validated.as_ref().is_some_and(|old|old.same_schema_generation(&before));
        if !validated {
            if !self.store().validate_mail_data_schema(&before).await.ok()? {return None}
            self.prepared_mail.memory.lock().ok()?.validated=Some(before.clone());
        }
        let mut canonical=args.clone();canonical["ids"]=json!(descriptor.ids());
        let mut result=self.email_get_for_publication(account,&canonical,&before).await;
        if result.get("type").is_some(){return None}
        let after=self.store().mail_data_snapshot(&descriptor).await.ok()??;
        let current_posture=self.v6_hooks().publication_posture_source();
        if !before.same_data(&after) || !after.publishable()
            || self.prepared_mail.memory.lock().ok()?.config_epoch!=config_epoch
            || !Arc::ptr_eq(&posture,&current_posture) || !supported(&posture,self)
            || !self.index_publisher.projection_is_current(account,descriptor.ids(),&projection)
        {return None}
        let missing=result.get("notFound")?.as_array()?.iter().map(|value|value.as_str().map(str::to_owned)).collect::<Option<BTreeSet<_>>>()?;
        let list=std::mem::take(result.get_mut("list")?.as_array_mut()?);
        let mut rows=BTreeMap::new();
        let mut used=4096+after.retained_data_bytes()+key.account.capacity()
            +key.ids.capacity()*std::mem::size_of::<String>()+key.ids.iter().map(|id|id.capacity()+128).sum::<usize>();
        let mut encoded_total=0usize;
        for value in list {
            let id=value.get("id")?.as_str()?.to_owned();
            if !key.ids.iter().any(|requested|requested==&id) || rows.contains_key(&id)
                || value.as_object()?.keys().any(|key|!PROPERTIES.contains(&key.as_str()))
            {return None}
            let encoded=serde_json::to_vec(&value).ok()?.into_boxed_slice();
            encoded_total=encoded_total.checked_add(encoded.len())?;
            if encoded_total>MAX_OUTPUT{return None}
            used=used.checked_add(encoded.len()+id.capacity()+128)?;
            if used>BUILD_RESERVATION{return None}
            rows.insert(id,Row {encoded});
        }
        if key.ids.iter().any(|id|rows.contains_key(id)==missing.contains(id.as_str()))
            || rows.len()+missing.len()!=key.ids.len(){return None}
        // Release only unused reservation; the actual charge survives eviction
        // while any candidate/response reader still holds its immutable Arc.
        drop(charge.split(tokens(BUILD_RESERVATION)-tokens(used))?);
        let ready=Arc::new(Ready {config_epoch,source:after,projection,posture,rows,_charge:charge});
        self.prepared_mail.insert(key,ready.clone());
        self.prepared_mail.stats.builds.fetch_add(1,Ordering::Relaxed);
        Some(PreparedMetadataCandidate {ready,requested})
    }

    fn prepared_current(&self,candidate:&PreparedMetadataCandidate,joint:&MailDataFence)->bool {
        let ready=&candidate.ready;
        if !self.prepared_mail.memory.lock().is_ok_and(|memory|memory.config.enabled && memory.config_epoch==ready.config_epoch){return false}
        let posture=self.v6_hooks().publication_posture_source();
        joint.publishable() && ready.source.same_data(joint)
            && Arc::ptr_eq(&posture,&ready.posture) && supported(&posture,self)
            && self.index_publisher.projection_is_current(joint.descriptor().account(),joint.descriptor().ids(),&ready.projection)
    }

    /// Pure compare-and-discard for a candidate whose final joint source row
    /// is missing/invalid before finish can be called. Removes only that exact
    /// Ready Arc, never a newer concurrent result for the same requested IDs.
    /// This is not an authorization decision or a reason to replay dispatch.
    pub fn discard_prepared_metadata(&self,candidate:&PreparedMetadataCandidate)->bool {
        self.prepared_mail.evict_same(&candidate_key(candidate),&candidate.ready)
    }

    /// PURE. `joint` MUST come from caller's final complete auth+DATA statement.
    /// Different call IDs/states are escaped here and never enter the cache key.
    pub fn finish_prepared_metadata(&self,candidate:PreparedMetadataCandidate,joint:&MailDataFence,call_id:&str)->Option<PreparedMetadataBytes> {
        let mut finish=Finish {service:&self.prepared_mail,candidate:&candidate,success:false};
        if call_id.len()>1024 || !self.prepared_current(&candidate,joint){return None}
        let mut out=Bounded(Vec::new());
        out.write_all(b"{\"methodResponses\":[[\"Email/get\",{").ok()?;
        out.write_all(b"\"accountId\":").ok()?;serde_json::to_writer(&mut out,joint.descriptor().account()).ok()?;
        out.write_all(b",\"state\":").ok()?;serde_json::to_writer(&mut out,&joint.email_state()).ok()?;
        out.write_all(b",\"list\":[").ok()?;
        let mut first=true;
        for id in &candidate.requested {
            if let Some(row)=candidate.ready.rows.get(id) {
                if !first{out.write_all(b",").ok()?}first=false;out.write_all(&row.encoded).ok()?;
            }
        }
        out.write_all(b"],\"notFound\":[").ok()?;first=true;
        for id in &candidate.requested {if !candidate.ready.rows.contains_key(id){
            if !first{out.write_all(b",").ok()?}first=false;serde_json::to_writer(&mut out,id).ok()?;
        }}
        out.write_all(b"]},").ok()?;serde_json::to_writer(&mut out,call_id).ok()?;
        out.write_all(b"]],\"sessionState\":").ok()?;serde_json::to_writer(&mut out,&joint.session_state()).ok()?;
        out.write_all(b"}").ok()?;
        // Check again after copying/encoding, so a concurrent accepted writer
        // cannot silently turn a long encoding into a certified current result.
        if !self.prepared_current(&candidate,joint){return None}
        self.prepared_mail.recertified(&candidate_key(&candidate),&candidate.ready);
        finish.success=true;
        self.prepared_mail.stats.raw_results.fetch_add(1,Ordering::Relaxed);
        Some(PreparedMetadataBytes(out.0))
    }

    /// Compatibility consumer: no Index/derived work on hit. The legacy Value
    /// API reparses stored bytes and serializes JSON; only raw emission avoids
    /// those costs. Do not call this the completed raw multi-call objective.
    pub fn finish_prepared_metadata_value(&self,candidate:&PreparedMetadataCandidate,joint:&MailDataFence)->Option<Value> {
        let mut finish=Finish {service:&self.prepared_mail,candidate,success:false};
        if !self.prepared_current(candidate,joint){return None}
        let mut list=Vec::new();let mut missing=Vec::new();
        for id in &candidate.requested {match candidate.ready.rows.get(id){Some(row)=>list.push(serde_json::from_slice::<Value>(&row.encoded).ok()?),None=>missing.push(id.clone())}}
        let response=json!({"accountId":joint.descriptor().account(),"state":joint.email_state(),"list":list,"notFound":missing});
        if !self.prepared_current(candidate,joint){return None}
        self.prepared_mail.recertified(&candidate_key(candidate),&candidate.ready);
        finish.success=true;
        self.prepared_mail.stats.value_results.fetch_add(1,Ordering::Relaxed);
        Some(response)
    }
}

struct Bounded(Vec<u8>);
impl Write for Bounded {
    fn write(&mut self,bytes:&[u8])->io::Result<usize>{
        if self.0.len().checked_add(bytes.len()).is_none_or(|n|n>MAX_OUTPUT){return Err(io::Error::other("prepared mail output bound"))}
        self.0.reserve_exact(bytes.len());self.0.extend_from_slice(bytes);Ok(bytes.len())
    }
    fn flush(&mut self)->io::Result<()>{Ok(())}
}
