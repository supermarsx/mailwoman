//! Bounded publication of immutable source/policy DATA. No allow/Session cache.
use super::*;
use mw_store::{
    Store, StoreError,
    authority_publication::{
        AuthorityKind as Kind, AuthorityScope as Scope, AuthorityVersions, NativeAuthorityFence,
    },
    native_admission_snapshot::{
        NativeAdmissionSharedKind as PartKind, NativeAdmissionSharedPart as Part,
    },
    settings_policy::{CanonicalSettingSubject, SettingSubject},
};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
const MAX_ENTRIES: usize = 16384;
const MAX_PENDING: usize = 256;
const MAX_BYTES: usize = 128 * 1024 * 1024;
const MAX_COMPONENTS: usize = 32768;
const MAX_STORES: usize = 32;
type Key = [u8; 32];
#[derive(Clone, PartialEq, Eq, Hash)]
struct PartKey {
    version: AuthorityVersions,
    schema: String,
    kind: PartKind,
    input: String,
    selector: Vec<String>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct PolicyKey {
    version: AuthorityVersions,
    schema: String,
    input: String,
    selector: Vec<String>,
}
struct PolicyRows {
    rows: Vec<(&'static registry::SettingDef, registry::Effective)>,
    bytes: usize,
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct RouteKey {
    version: AuthorityVersions,
    schema: String,
    domain: String,
    listed: bool,
    source: [u8; 32],
}
pub(crate) struct Published {
    pub snapshot: NativeAdmissionSnapshot,
    pub domains: Vec<String>,
    pub subjects: Vec<String>,
    versions: AuthorityVersions,
    schema: String,
    policies: HashMap<String, Arc<PolicyRows>>,
    canonical: HashMap<String, CanonicalSettingSubject>,
    pub gates: [account_gate::final_snapshot::PublishedGateInputs; 2],
    pub directory: Arc<account_gate::final_snapshot::PublishedDirectoryInputs>,
    pub guard:Option<Arc<crate::published_guard_inputs::PreparedGuardInputs>>,
    pub guard_authority:String,
    pub(crate) environment: Option<Arc<crate::request_inputs::Environment>>,
    pub(crate) request: Option<Arc<crate::request_inputs::Prepared>>,
    /// t34 bounded guard snapshot: the last mirror-stamped authority fence for
    /// this entry. Empty unless `MW_GUARD_SNAPSHOT=bounded`.
    pub(crate) guard_fence: crate::request_inputs::snapshot::Slot,
    own_bytes: usize,
}
impl Published {
    pub(crate) fn canonical(&self, subject: &SettingSubject<'_>) -> Option<&CanonicalSettingSubject> {
        self.canonical.get(&subject_key(subject))
    }
    /// Borrow immutable policy DATA without rebuilding a resolved-value vector.
    /// A caller still needs a current request source proof before using it.
    pub(crate) fn value(&self, subject: &SettingSubject<'_>, key: &str) -> Option<&registry::Effective> {
        self.policies.get(&subject_key(subject))?.rows.iter()
            .find(|(_, row)| row.key == key).map(|(_, row)| row)
    }
    pub fn matches(&self, fence: &NativeAuthorityFence) -> bool {
        self.versions.matches(fence.versions()).unwrap_or(false)
            && fence.publication_schema() == Some(self.schema.as_str())
    }
    pub fn values(
        &self,
        subject: &SettingSubject<'_>,
        filter: impl Fn(&registry::SettingDef) -> bool,
    ) -> Option<Vec<registry::Effective>> {
        let rows = self.policies.get(&subject_key(subject))?;
        Some(
            rows.rows
                .iter()
                .filter(|(def, _)| filter(def))
                .map(|(_, row)| row.clone())
                .collect(),
        )
    }
}
struct Hint {
    key: Key,
    cookie: String,
    account: String,
    selector: Vec<String>,
}
struct Cell {
    entry: Arc<Published>,
    used: u64,
}
struct Memory {
    entries: HashMap<Key, Cell>,
    pending: HashSet<Key>,
    failed: HashMap<Key, Instant>,
    directories: HashMap<
        (AuthorityVersions, String),
        Weak<account_gate::final_snapshot::PublishedDirectoryInputs>,
    >,
    parts: HashMap<PartKey, Weak<Part>>,
    policies: HashMap<PolicyKey, Weak<PolicyRows>>,
    guards:HashMap<(AuthorityVersions,String),Weak<crate::published_guard_inputs::PreparedGuardInputs>>,
    routes: HashMap<RouteKey, Weak<crate::request_inputs::RouteData>>,
    clock: u64,
    validated_schema: Option<(AuthorityVersions, String)>,
}
impl Memory {
    fn empty() -> Self {
        Self {
            entries: HashMap::new(),
            pending: HashSet::new(),
            failed: HashMap::new(),
            directories: HashMap::new(),
            parts: HashMap::new(),
            policies: HashMap::new(),
            guards:HashMap::new(),
            routes: HashMap::new(),
            clock: 0,
            validated_schema: None,
        }
    }
    fn route_bytes(&self) -> usize {
        self.routes.iter().filter_map(|(key, value)| value.upgrade().map(|data|
            data.retained_bytes().saturating_add(key.schema.len()).saturating_add(key.domain.len()).saturating_add(256)))
            .sum::<usize>()
    }
    fn retained(&self) -> usize {
        self.route_bytes() + self.guards.values().filter_map(Weak::upgrade).map(|g|g.retained_bytes()).sum::<usize>() + self.entries
            .values()
            .map(|c| c.entry.own_bytes)
            .sum::<usize>()
            + self
                .directories
                .values()
                .filter_map(Weak::upgrade)
                .map(|d| d.retained_bytes())
                .sum::<usize>()
            + self
                .parts
                .values()
                .filter_map(Weak::upgrade)
                .map(|p| p.retained_bytes())
                .sum::<usize>()
            + self
                .policies
                .values()
                .filter_map(Weak::upgrade)
                .map(|p| p.bytes)
                .sum::<usize>()
    }
    fn prune(&mut self) {
        self.directories.retain(|_, v| v.strong_count() > 0);
        self.parts.retain(|_, v| v.strong_count() > 0);
        self.policies.retain(|_, v| v.strong_count() > 0);
        self.guards.retain(|_,v|v.strong_count()>0);
        self.routes.retain(|_,v|v.strong_count()>0);
        while self.entries.len() > MAX_ENTRIES
            || self.retained() > MAX_BYTES
            || self.directories.len() + self.parts.len() + self.policies.len()+self.guards.len()+self.routes.len() > MAX_COMPONENTS
        {
            let Some(key) = self
                .entries
                .iter()
                .min_by_key(|(_, v)| v.used)
                .map(|(k, _)| *k)
            else {
                break;
            };
            self.entries.remove(&key);
            self.directories.retain(|_, v| v.strong_count() > 0);
            self.parts.retain(|_, v| v.strong_count() > 0);
            self.policies.retain(|_, v| v.strong_count() > 0);
        self.guards.retain(|_,v|v.strong_count()>0);
            self.routes.retain(|_,v|v.strong_count()>0);
        }
    }
}
/// AppState owns this handle. The task has only a Weak handle while waiting;
/// closed-channel shutdown drops its Store clone. Registry has no lifetime cycle.
pub(crate) struct Worker {
    tx: mpsc::Sender<Hint>,
    memory: Mutex<Memory>,
    built: AtomicU64,
    installed: AtomicU64,
    rejected: AtomicU64,
    /// t34: this AppState's EpochMirror (bounded guard snapshot). The mirror
    /// registry holds only a weak handle; this keeps the poller alive exactly
    /// as long as the AppState that owns this worker.
    guard_mirror: OnceLock<Arc<mw_store::authority_publication::mirror::EpochMirror>>,
    /// t34 step 2: parsed `v6:admin:bans`, reused while the protection global
    /// epoch is unchanged (bounded mode only; enforcement still per request).
    pub(crate) source_bans: crate::request_inputs::snapshot::GlobalSlot<Vec<mw_admin::BanEntry>>,
}
impl Worker {
    pub(crate) fn guard_mirror(&self, store: &Store, bound: Duration) -> Arc<mw_store::authority_publication::mirror::EpochMirror> {
        self.guard_mirror.get_or_init(|| {
            let mirror = mw_store::authority_publication::mirror::EpochMirror::for_store(store, bound);
            crate::request_inputs::snapshot::register_for_witness(&mirror, store);
            mirror
        }).clone()
    }
    pub(crate) fn existing_guard_mirror(&self) -> Option<Arc<mw_store::authority_publication::mirror::EpochMirror>> {
        self.guard_mirror.get().cloned()
    }
    pub(crate) fn start(store: Store) -> Option<Arc<Self>> {
        static REGISTRY: OnceLock<Mutex<HashMap<uuid::Uuid, Weak<Worker>>>> = OnceLock::new();
        let mut registry = REGISTRY
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .ok()?;
        registry.retain(|_, v| v.strong_count() > 0);
        if let Some(worker) = registry.get(&store.instance_id()).and_then(Weak::upgrade) {
            return Some(worker);
        }
        if registry.len() >= MAX_STORES {
            return None;
        }
        let (tx, mut rx) = mpsc::channel(1024);
        let worker = Arc::new(Self {
            tx,
            memory: Mutex::new(Memory::empty()),
            built: AtomicU64::new(0),
            installed: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
            guard_mirror: OnceLock::new(),
            source_bans: Default::default(),
        });
        registry.insert(store.instance_id(), Arc::downgrade(&worker));
        let weak = Arc::downgrade(&worker);
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(1));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let mut cleanup_ticks = 0u8;
            loop {
                tokio::select! {
                    hint=rx.recv()=>{
                        let Some(hint)=hint else {break;};let Some(worker)=weak.upgrade() else {break;};
                        worker.built.fetch_add(1,Ordering::Relaxed);
                        let built=worker.build(&store,&hint).await;
                        if let Ok(mut memory)=worker.memory.lock() {
                            memory.pending.remove(&hint.key);
                            if let Ok(entry)=built {
                                memory.clock=memory.clock.wrapping_add(1);let used=memory.clock;
                                memory.entries.insert(hint.key,Cell {entry:Arc::new(entry),used});memory.prune();
                                worker.installed.fetch_add(1,Ordering::Relaxed);
                            } else {
                                worker.rejected.fetch_add(1,Ordering::Relaxed);
                                if memory.failed.len()>=1024 {memory.failed.retain(|_,until|*until>Instant::now());}
                                if memory.failed.len()<1024 {memory.failed.insert(hint.key,Instant::now()+Duration::from_secs(1));}
                            }
                        }
                    },
                    _=tick.tick()=>{
                        let Some(worker)=weak.upgrade() else {break;};
                        // ACK means invalidation hint handled, NEVER installed RAM.
                        // Every request still compares a new coherent SQL vector.
                        if let Ok(claims)=store.claim_authority_publications(32,chrono::Utc::now().timestamp(),30).await {
                            for claim in claims {
                                if let Ok(mut memory)=worker.memory.lock() {
                                    let scope=claim.stamp().scope();
                                    memory.entries.retain(|_,cell|!cell.entry.versions.contains(scope) || cell.entry.versions.observes(claim.stamp()));
                                    memory.prune();
                                }
                                let _=store.acknowledge_authority_publication(&claim).await;
                            }
                        }
                        cleanup_ticks=cleanup_ticks.wrapping_add(1);
                        if cleanup_ticks==64 {
                            cleanup_ticks=0;
                            let _=store.cleanup_native_authority_anchors(32).await;
                            let _=store.cleanup_obsolete_authority_scopes(32).await;
                        }
                    }
                }
            }
        });
        Some(worker)
    }
    #[cfg(test)]
    pub(super) fn statistics(&self) -> (u64, u64, u64, usize, usize) {
        let (entries, pending) = self
            .memory
            .lock()
            .map(|m| (m.entries.len(), m.pending.len()))
            .unwrap_or_default();
        (
            self.built.load(Ordering::Relaxed),
            self.installed.load(Ordering::Relaxed),
            self.rejected.load(Ordering::Relaxed),
            entries,
            pending,
        )
    }
    #[cfg(test)]
    pub(super) fn request_retention(&self, cookie: &str, environment: &crate::request_inputs::Environment)
        -> Option<(usize, usize, usize)>
    {
        let memory = self.memory.lock().ok()?;
        let key = key(cookie, &[REQUEST_INPUT_SELECTOR.into(), environment.fingerprint()]);
        let entry = &memory.entries.get(&key)?.entry;
        let private = entry.own_bytes + entry.snapshot.publication_parts().iter()
            .filter(|part| matches!(part.kind(), PartKind::Account | PartKind::Flags))
            .map(|part| part.retained_bytes()).sum::<usize>();
        Some((private, memory.retained(), MAX_BYTES))
    }
    #[cfg(test)]
    pub(super) fn route_retention_for_test(&self) -> (usize, usize) {
        let memory = self.memory.lock().unwrap();
        (memory.routes.values().filter(|route| route.strong_count() > 0).count(), memory.route_bytes())
    }
    #[cfg(test)]
    pub(super) async fn build_request_once_for_test(
        &self, store: &Store, cookie: &str, account: &str,
    ) -> Result<Option<Published>, StoreError> {
        let Some(environment) = crate::request_inputs::Environment::capture() else { return Ok(None); };
        let selector = vec![REQUEST_INPUT_SELECTOR.into(), environment.fingerprint()];
        self.build_once(store, &Hint {
            key: key(cookie, &selector), cookie: cookie.into(), account: account.into(), selector,
        }).await
    }
    #[cfg(test)]
    pub(super) async fn build_once_for_test(
        &self,
        store: &Store,
        cookie: &str,
        account: &str,
    ) -> Result<Option<Published>, StoreError> {
        let selector = selector(&Needs::default()).unwrap();
        self.build_once(
            store,
            &Hint {
                key: key(cookie, &selector),
                cookie: cookie.into(),
                account: account.into(),
                selector,
            },
        )
        .await
    }
    /// Only called after the existing current primary hint and same-owner
    /// selector validation. Arbitrary invalid cookies cannot queue build work.
    pub(super) fn hint(&self, cookie: &str, account: &str, needs: &Needs) {
        let Some(selector) = selector(needs) else { return; };
        self.hint_selector(cookie, account, selector);
    }
    /// Ordinary requests publish bounded INPUTS, never a body-specific allow.
    pub(crate) fn hint_request_inputs(&self, cookie: &str, account: &str) {
        #[cfg(feature="load-test-fixtures")]
        {
            if !crate::request_inputs_fixture::enabled() { return; }
            crate::request_inputs_fixture::record(crate::request_inputs_fixture::Event::Hint);
        }
        let Some(environment) = crate::request_inputs::process_environment() else { return; };
        self.hint_selector(cookie, account, vec![REQUEST_INPUT_SELECTOR.into(), environment.fingerprint.clone()]);
    }
    fn hint_selector(&self, cookie: &str, account: &str, selector: Vec<String>) {
        if cookie.is_empty()
            || cookie.len() > 1024
            || account.is_empty()
            || account.len() > 1024
            || !account.is_ascii()
        {
            return;
        }
        let key = key(cookie, &selector);
        let Ok(mut memory) = self.memory.try_lock() else {
            return;
        };
        if memory.entries.contains_key(&key)
            || memory.pending.len() >= MAX_PENDING
            || memory.pending.contains(&key)
            || memory
                .failed
                .get(&key)
                .is_some_and(|until| *until > Instant::now())
        {
            return;
        }
        memory.pending.insert(key);
        if self
            .tx
            .try_send(Hint {
                key,
                cookie: cookie.into(),
                account: account.into(),
                selector,
            })
            .is_err()
        {
            memory.pending.remove(&key);
        }
    }
    pub(super) fn invalidate(&self, cookie: &str, needs: &Needs) {
        if let Some(selector) = selector(needs) {
            if let Ok(mut memory) = self.memory.try_lock() {
                memory.entries.remove(&key(cookie, &selector));
            }
        }
    }
    pub(super) fn get(&self, cookie: &str, needs: &Needs) -> Option<Arc<Published>> {
        let selector = selector(needs)?;
        self.get_selector(cookie, &selector)
    }
    #[cfg(test)]
    pub(crate) fn request_inputs(&self, cookie: &str, environment: &crate::request_inputs::Environment) -> Option<Arc<Published>> {
        self.request_inputs_by_fingerprint(cookie, &environment.fingerprint())
    }
    pub(crate) fn request_inputs_by_fingerprint(&self, cookie: &str, fingerprint: &str) -> Option<Arc<Published>> {
        self.get_selector(cookie, &[REQUEST_INPUT_SELECTOR.into(), fingerprint.into()])
    }
    pub(crate) fn discard_request_inputs_by_fingerprint(&self, cookie: &str, fingerprint: &str, expected: &Arc<Published>) {
        let key = key(cookie, &[REQUEST_INPUT_SELECTOR.into(), fingerprint.into()]);
        if let Ok(mut memory) = self.memory.try_lock()
            && memory.entries.get(&key).is_some_and(|cell| Arc::ptr_eq(&cell.entry, expected)) {
            memory.entries.remove(&key);
        }
    }
    /// Lookup for a caller whose outer guards already CONSUMED published inputs.
    /// There a miss is a terminal refusal, not a fallback, so a lookup that only
    /// lost a race for the memory mutex must wait for it. Every critical section
    /// on this mutex is synchronous and short (no await while held). t34: the
    /// nonblocking lookup returned a spurious 503 `publishedGuardFenceRefusal`
    /// under concurrent load on a multi-thread runtime.
    pub(super) fn get_consumed(&self, cookie: &str, needs: &Needs) -> Option<Arc<Published>> {
        let selector = selector(needs)?;
        let key = key(cookie, &selector);
        let mut memory = self.memory.lock().ok()?;
        memory.clock = memory.clock.wrapping_add(1);
        let used = memory.clock;
        let cell = memory.entries.get_mut(&key)?;
        cell.used = used;
        Some(cell.entry.clone())
    }
    /// Test-only: hold the memory mutex on another OS thread for `hold`, the
    /// way a concurrent request on another runtime worker does for a moment.
    /// The returned receiver fires once the mutex is held.
    #[cfg(test)]
    pub(super) fn contend_for_test(self: Arc<Self>, hold: Duration) -> std::sync::mpsc::Receiver<()> {
        let (held, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let guard = self.memory.lock().unwrap();
            held.send(()).unwrap();
            std::thread::sleep(hold);
            drop(guard);
        });
        receiver
    }
    fn get_selector(&self, cookie: &str, selector: &[String]) -> Option<Arc<Published>> {
        let key = key(cookie, &selector);
        let mut memory = self.memory.try_lock().ok()?;
        memory.clock = memory.clock.wrapping_add(1);
        let used = memory.clock;
        let cell = memory.entries.get_mut(&key)?;
        cell.used = used;
        Some(cell.entry.clone())
    }
    fn route(&self, entry: &Published, identity: &crate::domain_config::IdentitySnapshot, listed: bool)
        -> Option<Arc<crate::request_inputs::RouteData>>
    {
        let row = identity.row.as_ref()?;
        if !row.enabled { return None; }
        let key = RouteKey {
            version: entry.versions.subset(&[
                Scope::new(Kind::Routing, "").ok()?,
                Scope::new(Kind::Domain, &identity.identity.domain).ok()?,
            ]).ok()?,
            schema: entry.schema.clone(), domain: identity.identity.domain.clone(), listed,
            source: Sha256::digest(row.upstream_json.as_bytes()).into(),
        };
        let existing = self.memory.lock().ok()?.routes.get(&key).and_then(Weak::upgrade);
        if let Some(existing) = existing {
            // Hashes index candidates, never replace exact semantic input proof.
            return (existing.source == row.upstream_json).then_some(existing);
        }
        let data = Arc::new(crate::request_inputs::RouteData::build(identity, listed)?);
        if data.retained_bytes() > 8 * 1024 * 1024 { return None; }
        let mut memory = self.memory.lock().ok()?;
        memory.prune();
        if let Some(existing) = memory.routes.get(&key).and_then(Weak::upgrade) {
            return (existing.source == row.upstream_json).then_some(existing);
        }
        if memory.directories.len() + memory.parts.len() + memory.policies.len()
            + memory.guards.len() + memory.routes.len() >= MAX_COMPONENTS { return None; }
        memory.routes.insert(key, Arc::downgrade(&data));
        Some(data)
    }

    async fn build(&self, store: &Store, hint: &Hint) -> Result<Published, StoreError> {
        for _ in 0..3 {
            if let Some(entry) = self.build_once(store, hint).await? {
                return Ok(entry);
            }
        }
        Err(StoreError::SourceChanged)
    }
    async fn build_once(
        &self,
        store: &Store,
        hint: &Hint,
    ) -> Result<Option<Published>, StoreError> {
        #[cfg(feature="load-test-fixtures")]
        if hint.selector.first().is_some_and(|key|key==REQUEST_INPUT_SELECTOR) {
            if !crate::request_inputs_fixture::enabled() { return Ok(None); }
            crate::request_inputs_fixture::record(crate::request_inputs_fixture::Event::Build);
        }
        let environment = if hint.selector.first().is_some_and(|key| key == REQUEST_INPUT_SELECTOR) {
            let Some(environment) = crate::request_inputs::Environment::capture() else { return Ok(None); };
            if hint.selector.get(1) != Some(&environment.fingerprint()) { return Ok(None); }
            Some(Arc::new(environment))
        } else { None };
        // Initial source is shaping only. It is discarded before the final
        // before/after capture, so seeding cannot bless a pre-write source.
        let initial = load_source(store, &hint.cookie).await?;
        let session = initial
            .session(store)?
            .ok_or(StoreError::SourceChanged)?
            .session;
        if session.account_id != hint.account {
            return Ok(None);
        }
        let principal = initial.principal()?.ok_or(StoreError::SourceChanged)?;
        let known_domains = domains(&initial)?;
        let mut seed_scopes = vec![
            Scope::new(Kind::Account, &session.account_id)?,
            Scope::new(Kind::Session, &session.id)?,
            Scope::new(Kind::Principal, &principal.principal_id)?,
        ];
        seed_scopes.extend(
            known_domains
                .iter()
                .map(|s| Scope::new(Kind::Domain, s))
                .collect::<Result<Vec<_>, _>>()?,
        );
        store.seed_authority_scopes(&seed_scopes).await?;
        let Some(root) = store
            .native_publication_fence(&hint.cookie, &known_domains, &[])
            .await?
        else {
            return Ok(None);
        };
        let Some(schema) = root.publication_schema().map(str::to_owned) else {
            return Ok(None);
        };
        let schema_key = (root.versions().subset(&[])?, schema.clone());
        let already = self
            .memory
            .lock()
            .map(|m| m.validated_schema.as_ref() == Some(&schema_key))
            .unwrap_or(false);
        if !already {
            if !store.validate_native_publication_schema(&root).await? {
                return Ok(None);
            }
            if let Ok(mut memory) = self.memory.lock() {
                memory.validated_schema = Some(schema_key);
            }
        }
        let shaping = load_source(store, &hint.cookie).await?;
        if domains(&shaping)? != known_domains {
            return Ok(None);
        }
        let Some(seed) = store.prepare_native_authority_seed(&shaping, &root).await? else {
            return Ok(None);
        };
        if !store.seed_native_authority_subjects(&seed).await? {
            return Ok(None);
        }
        let subjects = seed.subjects();
        let Some(before) = store
            .native_publication_fence(&hint.cookie, &known_domains, &subjects)
            .await?
        else {
            return Ok(None);
        };
        if before.publication_schema() != Some(schema.as_str()) {
            return Ok(None);
        }
        let mut source = load_source(store, &hint.cookie).await?;
        if !source.bounded_shape() || domains(&source)? != known_domains {
            return Ok(None);
        }
        let session = source
            .session(store)?
            .ok_or(StoreError::SourceChanged)?
            .session;
        if session.account_id != hint.account || source.native_subjects(store)?.is_none() {
            return Ok(None);
        }
        let names =
            names_for_store(&source, store, &session).map_err(|_| StoreError::SourceChanged)?;
        let raw_subjects = policy_subjects(&source, &session)?;
        let mut policy_inputs = Vec::new();
        let mut canonical_subjects = HashMap::new();
        // Different original subject spellings may canonicalize to the same
        // exact input/version. Resolve that component once in this build too;
        // nothing enters shared memory until the AFTER fence matches.
        let mut prepared_policies = HashMap::<PolicyKey, Arc<PolicyRows>>::new();
        for raw in raw_subjects {
            let canonical = store
                .settings_publication_canonical(&raw.as_subject())
                .await?;
            if (!canonical.domain.is_empty() && !known_domains.contains(&canonical.domain))
                || (!canonical.account_id.is_empty() && !subjects.contains(&canonical.account_id))
                || (!canonical.address.is_empty() && !subjects.contains(&canonical.address))
            {
                return Ok(None);
            }
            let identity = policy_identity(before.versions(), &schema, &canonical, &hint.selector)?;
            let existing = prepared_policies.get(&identity).cloned().or_else(|| {
                self.memory
                    .lock()
                    .ok()
                    .and_then(|m| m.policies.get(&identity).and_then(Weak::upgrade))
            });
            let prepared = if let Some(existing) = existing {
                existing
            } else {
                let inputs = store.settings_publication_inputs(&canonical).await?;
                prepare_policy_rows(registry::effective_all_from_inputs_with_env(
                    inputs,
                    |def| if environment.is_some() { crate::request_inputs::needed(def, &canonical) }
                        else { needed(def, &hint.selector) },
                    &[],
                    &|key| match &environment { Some(env) => env.lookup(key), None => registry::process_env(key) },
                ))?
            };
            prepared_policies.insert(identity.clone(), prepared.clone());
            canonical_subjects.insert(subject_key(&raw.as_subject()), canonical);
            policy_inputs.push((subject_key(&raw.as_subject()), identity, prepared));
        }
        let mut guard_key=crate::published_guard_inputs::PreparedGuardInputs::configuration_key(&before)?;
        if let Some(environment) = &environment {
            guard_key.1.push('\u{1f}');
            guard_key.1.push_str(&environment.fingerprint());
        }
        let mut guard=self.memory.lock().ok().and_then(|m|m.guards.get(&guard_key).and_then(Weak::upgrade));
        if guard.is_none() {
            // Optional guarded-read DATA. Unsupported/missing114 coverage does
            // not disable the existing native auth publication/ordinary path.
            guard=match &environment {
                Some(env) => crate::published_guard_inputs::PreparedGuardInputs::prepare_captured(store,&hint.cookie,&known_domains,&subjects,env).await.ok().flatten(),
                None => crate::published_guard_inputs::PreparedGuardInputs::prepare(store,&hint.cookie,&known_domains,&subjects).await.ok().flatten(),
            };
        }
        if guard.as_ref().is_some_and(|g|!g.matches_configuration(store,&before).unwrap_or(false)){guard=None;}
        #[cfg(test)]
        test_control::wait(test_control::Stage::PublicationBeforeFinal, &hint.cookie).await;
        let Some(after) = store
            .native_publication_fence(&hint.cookie, &known_domains, &subjects)
            .await?
        else {
            return Ok(None);
        };
        if !before.same_published_source(&after)? {
            return Ok(None);
        }
        if guard.as_ref().is_some_and(|g|!g.matches_configuration(store,&after).unwrap_or(false)){return Ok(None);}
        if environment.as_ref().is_some_and(|env| !env.matches()) { return Ok(None); }
        let guard_authority=source.native_subjects(store)?.ok_or(StoreError::SourceChanged)?.storage_authority;
        let versions = after.versions().clone();
        source.split_publication_parts();
        let directory_key = (
            versions.subset(&[Scope::new(Kind::Directory, "")?])?,
            schema.clone(),
        );
        let directory = self
            .memory
            .lock()
            .ok()
            .and_then(|m| m.directories.get(&directory_key).and_then(Weak::upgrade));
        let directory = match directory {
            Some(data) => data,
            None => Arc::new(
                account_gate::final_snapshot::PublishedDirectoryInputs::from_native(&source)?,
            ),
        };
        let gates = [
            prepared(&hint.account, &names.0)
                .map_err(|_| StoreError::SourceChanged)?
                .publication_inputs(&source)?,
            prepared(&hint.account, &names.1)
                .map_err(|_| StoreError::SourceChanged)?
                .publication_inputs(&source)?,
        ];
        let mut memory = self
            .memory
            .lock()
            .map_err(|_| StoreError::Invalid("publication memory unavailable"))?;
        memory.prune();
        memory
            .directories
            .insert(directory_key, Arc::downgrade(&directory));
        if memory.directories.len()
            + memory.parts.len()
            + memory.policies.len()+memory.guards.len()+memory.routes.len()+usize::from(guard.is_some())
            + source.publication_parts().len()
            + policy_inputs.len()
            > MAX_COMPONENTS
        {
            return Err(StoreError::Invalid("publication component budget"));
        }
        for index in 0..source.publication_parts().len() {
            let part = source.publication_parts()[index].clone();
            let scopes = match part.kind() {
                PartKind::Directory => vec![Scope::new(Kind::Directory, "")?],
                PartKind::Account => vec![Scope::new(Kind::Account, &hint.account)?],
                PartKind::Flags => vec![
                    Scope::new(Kind::Policy, "")?,
                    Scope::new(Kind::Account, &hint.account)?,
                ],
                PartKind::Domain => {
                    let rows = source.domains();
                    let row = rows.get(part.input()).ok_or(StoreError::SourceChanged)?;
                    let row = row.as_ref().map_err(|_| StoreError::SourceChanged)?;
                    vec![
                        Scope::new(Kind::Routing, "")?,
                        Scope::new(Kind::Domain, &row.canonical)?,
                    ]
                }
            };
            let part_key = PartKey {
                version: versions.subset(&scopes)?,
                schema: schema.clone(),
                kind: part.kind(),
                input: part.input().into(),
                selector: if part.kind() == PartKind::Flags {
                    source_keys(&source)
                } else {
                    Vec::new()
                },
            };
            if let Some(existing) = memory.parts.get(&part_key).and_then(Weak::upgrade) {
                source.replace_publication_part(index, existing)?;
            } else {
                memory.parts.insert(part_key, Arc::downgrade(&part));
            }
        }
        let mut policies = HashMap::new();
        for (raw, identity, data) in policy_inputs {
            memory.policies.insert(identity, Arc::downgrade(&data));
            if let Some(canonical) = canonical_subjects.get(&raw) {
                policies.insert(subject_key(&canonical.as_subject()), data.clone());
            }
            policies.insert(raw, data);
        }
        if let Some(guard)=&guard {memory.guards.insert(guard_key,Arc::downgrade(guard));}
        let own_bytes = gates.iter().map(|g| g.retained_bytes()).sum::<usize>()
            + source.publication_own_bytes()
            + names
                .0
                .iter()
                .chain(&names.1)
                .map(String::len)
                .sum::<usize>()
            + environment.as_ref().map_or(0, |env| env.retained_bytes())
            + canonical_subjects.iter().map(|(key, subject)| key.len() + subject.domain.len()
                + subject.account_id.len() + subject.address.len() + 256).sum::<usize>() + 4096
            + if crate::request_inputs::snapshot::mode() == crate::request_inputs::snapshot::Mode::Off { 0 }
                else { crate::request_inputs::snapshot::SLOT_BYTES };
        let mut entry = Published {
            snapshot: source,
            domains: known_domains,
            subjects,
            versions,
            schema,
            policies,
            canonical: canonical_subjects,
            gates,
            directory,
            own_bytes,
            guard,guard_authority,
            environment,
            request: None,
            guard_fence: Default::default(),
        };
        // Prepared route interning takes this same mutex. Never hold the
        // component-map lock across parser work or recursive acquisition.
        drop(memory);
        if entry.environment.is_some() {
            let Some(inputs) = crate::request_inputs::Prepared::build(&entry, &session, store,
                |identity, listed| self.route(&entry, identity, listed)) else { return Ok(None); };
            let bytes = inputs.retained_bytes();
            if bytes > 8 * 1024 * 1024 { return Ok(None); }
            entry.own_bytes = entry.own_bytes.checked_add(bytes).ok_or(StoreError::Invalid("request input retained bound"))?;
            entry.request = Some(Arc::new(inputs));
            // Ordinary consumers retain the compact typed projection, not an
            // Effective row/provenance vector for every registry key/subject.
            entry.policies.clear();
            entry.canonical.clear();
        }
        #[cfg(feature="load-test-fixtures")]
        if entry.request.is_some() {
            crate::request_inputs_fixture::record(crate::request_inputs_fixture::Event::Built);
        }
        Ok(Some(entry))
    }
}
fn prepare_policy_rows(rows: Vec<registry::Effective>) -> Result<Arc<PolicyRows>, StoreError> {
    let bytes = serde_json::to_vec(
        &rows
            .iter()
            .map(|r| (&r.key, &r.value, &r.user_value))
            .collect::<Vec<_>>(),
    )
    .map_err(|_| StoreError::Invalid("publication policy encoding"))?
    .len()
        + rows.len() * 256;
    let rows = rows
        .into_iter()
        .map(|row| {
            registry::lookup(&row.key)
                .map(|def| (def, row))
                .ok_or(StoreError::Invalid("publication definition disappeared"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(PolicyRows { rows, bytes }))
}
fn policy_identity(
    versions: &AuthorityVersions,
    schema: &str,
    canonical: &CanonicalSettingSubject,
    selector: &[String],
) -> Result<PolicyKey, StoreError> {
    let mut scopes = vec![
        Scope::new(Kind::Policy, "")?,
        Scope::new(Kind::Routing, "")?,
    ];
    if !canonical.domain.is_empty() {
        scopes.push(Scope::new(Kind::Domain, &canonical.domain)?);
    }
    for subject in [&canonical.account_id, &canonical.address] {
        if !subject.is_empty() {
            scopes.push(Scope::new(Kind::Subject, subject)?);
        }
    }
    Ok(PolicyKey {
        version: versions.subset(&scopes)?,
        schema: schema.into(),
        input: subject_key(&canonical.as_subject()),
        selector: selector.to_vec(),
    })
}
fn selector(needs: &Needs) -> Option<Vec<String>> {
    if needs.native
        || needs.link_limit
        || needs.upload_bytes.is_some()
        || needs.recipients != 0
        || needs.avatars.is_some()
    {
        return None;
    }
    let mut keys = needs
        .features
        .iter()
        .filter_map(|id| crate::features::feature(id))
        .flat_map(|f| f.capability_keys.iter().map(|k| (*k).to_owned()))
        .collect::<Vec<_>>();
    keys.extend(needs.required.iter().map(|k| (*k).to_owned()));
    keys.push("access.protocols".into());
    keys.push("performance.preparedMail".into());
    keys.sort();
    keys.dedup();
    (keys.len() <= 64).then_some(keys)
}
fn key(cookie: &str, selector: &[String]) -> Key {
    let mut h = Sha256::new();
    h.update(cookie.as_bytes());
    for k in selector {
        h.update([0]);
        h.update(k.as_bytes());
    }
    h.finalize().into()
}
const REQUEST_INPUT_SELECTOR: &str = "@request-inputs-v1";
fn needed(def: &registry::SettingDef, selector: &[String]) -> bool {
    selector.iter().any(|s| s == def.key)
        || matches!(
            def.key,
            "signin.domainMode" | "session.idleSecs" | "session.absoluteSecs"
        )
        || def.key.starts_with("ratelimit.")
}
fn subject_key(s: &SettingSubject<'_>) -> String {
    format!("{}\u{1f}{}\u{1f}{}", s.domain, s.account_id, s.address)
}
fn domains(source: &NativeAdmissionSnapshot) -> Result<Vec<String>, StoreError> {
    let mut domains = source
        .domains()
        .into_values()
        .map(|v| v.map(|v| v.canonical))
        .collect::<Result<Vec<_>, _>>()?;
    domains.sort();
    domains.dedup();
    if domains.len() > 16 {
        return Err(StoreError::Invalid("publication domain bound"));
    }
    Ok(domains)
}
fn policy_subjects(
    source: &NativeAdmissionSnapshot,
    session: &Session,
) -> Result<Vec<CanonicalSettingSubject>, StoreError> {
    let raw = session.username.trim().to_ascii_lowercase();
    let domain = raw
        .rsplit_once('@')
        .ok_or(StoreError::Invalid("publication qualified name"))?
        .1;
    let clock_domain = session.username.rsplit_once('@').map_or("", |(_, d)| d);
    let input = domain_config::QualifiedIdentityInput::parse(&session.credentials.username)
        .map_err(|_| StoreError::SourceChanged)?
        .ok_or(StoreError::SourceChanged)?;
    let mut rows = source.domains();
    let row = rows
        .remove(&input.domain)
        .ok_or(StoreError::SourceChanged)??;
    let canonical = input.canonical(row.canonical);
    let rate = crate::rate_leeway::domain_of(&session.username);
    let values = [
        SettingSubject {
            personal_owner: None,
            domain,
            account_id: &session.account_id,
            address: &raw,
        },
        SettingSubject {
            personal_owner: None,
            domain: clock_domain,
            account_id: &session.account_id,
            address: &session.username,
        },
        SettingSubject::default(),
        SettingSubject {
            personal_owner: None,
            domain: &canonical.domain,
            account_id: "",
            address: &canonical.address,
        },
        SettingSubject {
            domain: rate,
            ..Default::default()
        },
    ];
    let mut out = Vec::new();
    for s in values {
        let owned = CanonicalSettingSubject {
            personal_owner: None,
            domain: s.domain.into(),
            account_id: s.account_id.into(),
            address: s.address.into(),
        };
        if !out.contains(&owned) {
            out.push(owned);
        }
    }
    Ok(out)
}
async fn load_source(store: &Store, cookie: &str) -> Result<NativeAdmissionSnapshot, StoreError> {
    // All shaping reads are worker accounting, never hidden request reads.
    let first = store.primary_session_policy_snapshot(cookie).await?;
    let names = vec![
        first.session.username.clone(),
        first.session.credentials.username.clone(),
    ];
    let query =
        shaping(&first.session.account_id, &names).map_err(|_| StoreError::SourceChanged)?;
    let mut source = store
        .native_cookie_admission_snapshot(cookie, &query)
        .await?;
    let session = source
        .session(store)?
        .ok_or(StoreError::SourceChanged)?
        .session;
    let names = names_for_store(&source, store, &session).map_err(|_| StoreError::SourceChanged)?;
    let names = names.0.into_iter().chain(names.1).collect::<Vec<_>>();
    let complete = shaping(&session.account_id, &names).map_err(|_| StoreError::SourceChanged)?;
    if !source.covers_domains(complete.domains()) || !source.covers_keys(complete.keys()) {
        source = store
            .native_cookie_admission_snapshot(cookie, &complete)
            .await?;
    }
    Ok(source)
}
fn source_keys(source: &NativeAdmissionSnapshot) -> Vec<String> {
    source.publication_flag_keys()
}
