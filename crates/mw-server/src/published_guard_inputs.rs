//! Immutable guard INPUT preparation and sharing for explicitly declared reads.
//! No authenticated Session, plaintext credential or allow is cached.
use crate::{client_access, inbound_allowlist, maintenance_mode, settings_registry as registry};
use mw_store::{
    Store, StoreError,
    authority_publication::{GuardInputSection, NativeAuthorityFence, NativeGuardInputSnapshot},
    settings_policy::CanonicalSettingSubject,
};
use std::{collections::BTreeMap, io, sync::Arc};

const MAX_PREPARED_BYTES: usize = 8 * 1024 * 1024;

pub(crate) struct PreparedGuardInputs {
    source: NativeGuardInputSnapshot,
    globals: BTreeMap<String, registry::Effective>,
    inbound: Result<inbound_allowlist::PublishedInputs, ()>,
    client: Result<client_access::PublishedInputs, ()>,
    maintenance: Result<Arc<maintenance_mode::Config>, ()>,
    logical_bytes: usize,
    protocol:Arc<crate::protocol_guard::config::Config>,
    probe:Result<Arc<crate::admin_probe::config::Policy>,()>,
    protection:Result<Arc<crate::request_protection::config::Config>,()>,
    trust:Result<Arc<crate::trust_tier::Policy>,()>,
    global_knobs:crate::rate_leeway::Knobs,
    watch:Arc<crate::session_watch::Config>,
    geography:Result<Arc<crate::signin_geography::Policy>,()>,
}

// Deliberately no Debug/Serialize: config may contain a slug or password hash.
impl PreparedGuardInputs {
    /// Existing resolver/default/locks, resolved once during cold preparation.
    /// No legacy settings memo is certified by an authority epoch.
    pub(crate) async fn prepare(
        store: &Store,
        cookie: &str,
        domains: &[String],
        subjects: &[String],
    ) -> Result<Option<Arc<Self>>, StoreError> {
        let Some(source) = store
            .native_guard_input_source_snapshot(cookie, domains, subjects)
            .await?
        else {
            return Ok(None);
        };
        let canonical = CanonicalSettingSubject {
            personal_owner: None,
            domain: String::new(),
            account_id: String::new(),
            address: String::new(),
        };
        let inputs = store.settings_publication_inputs(&canonical).await?;
        let rows = registry::effective_all_from_inputs(inputs, |_| true, &[]);
        // Count encoded logical policy size without allocating an oversized
        // serialized copy. This is not an allocator/RSS or DB workspace bound.
        let mut count = CountBytes(0);
        serde_json::to_writer(
            &mut count,
            &rows
                .iter()
                .map(|r| (&r.key, &r.value, &r.user_value))
                .collect::<Vec<_>>(),
        )
        .map_err(|_| StoreError::Invalid("published guard policy byte bound"))?;
        let inbound = source
            .raw(store, GuardInputSection::InboundAllowlist)
            .map_err(|_| ())
            .and_then(inbound_allowlist::prepare_published_inputs);
        let client = source
            .raw(store, GuardInputSection::ClientAccess)
            .map_err(|_| ())
            .and_then(client_access::prepare_published_inputs);
        let maintenance = source
            .raw(store, GuardInputSection::Maintenance)
            .map_err(|_| ())
            .and_then(maintenance_mode::prepare_published_inputs);
        #[cfg(test)]
        test_control::wait().await;
        let Some(after) = store
            .native_publication_fence(cookie, domains, subjects)
            .await?
        else {
            return Ok(None);
        };
        if !source.same_source(store, &after)? {
            return Err(StoreError::SourceChanged);
        }
        let selected=|prefix:&str|rows.iter().filter(|r|r.key.starts_with(prefix)).cloned().collect::<Vec<_>>();
        let protocol=Arc::new(crate::protocol_guard::config::Config::parse(selected("security.protocol.")));
        let probe=crate::admin_probe::config::Policy::parse(selected(crate::admin_probe::config::PREFIX)).map(Arc::new);
        let protection=crate::request_protection::config::Config::parse(selected(mw_protection::SETTINGS_PREFIX)).map(Arc::new);
        let trust=crate::trust_tier::Policy::parse(selected(crate::trust_tier::PREFIX)).map(Arc::new);
        let global_knobs=crate::rate_leeway::Knobs::from_rows(&rows);
        let watch=crate::session_watch::prepare_published_inputs(&rows);
        let geography=crate::signin_geography::prepare_published_inputs(&rows);
        let globals = rows.into_iter().map(|row| (row.key.clone(), row)).collect();
        Ok(Some(Arc::new(Self {
            source,
            globals,
            inbound,
            client,
            maintenance,
            logical_bytes: count.0,
            protocol,probe,protection,trust,global_knobs,watch,geography,
        })))
    }
    /// An initial preparation frame is not admission. Every future consumer
    /// must compare a NEW exact source vector at its original/late boundary.
    pub(crate) fn matches(
        &self,
        store: &Store,
        current: &NativeAuthorityFence,
    ) -> Result<bool, StoreError> {
        self.source.same_source(store, current)
    }
    /// GLOBAL configuration DATA can be shared between accounts. This never
    /// certifies a primary actor: the consumer ALSO compares its auth entry
    /// against the freshly source-derived complete per-request frame.
    pub(crate) fn matches_configuration(&self,store:&Store,current:&NativeAuthorityFence)->Result<bool,StoreError> {
        if self.source.source().publication_schema().is_none()||self.source.source().publication_schema()!=current.publication_schema(){return Ok(false);}
        let own=Self::configuration_key(self.source.source())?;
        let fresh=Self::configuration_key(current)?;
        if !self.source.belongs_to(store){return Err(StoreError::Invalid("foreign guard globals"));}
        Ok(own==fresh)
    }
    pub(crate) fn configuration_key(source:&NativeAuthorityFence)->Result<(mw_store::authority_publication::AuthorityVersions,String),StoreError> {
        use mw_store::authority_publication::{AuthorityScope,AuthorityKind};
        Ok((source.versions().subset(&[AuthorityScope::new(AuthorityKind::Policy,"")?,AuthorityScope::new(AuthorityKind::Routing,"")?,AuthorityScope::new(AuthorityKind::Protection,"")?])?,source.publication_schema().ok_or(StoreError::Invalid("missing guard schema"))?.into()))
    }
    /// A conservative logical charge, not a process RSS/allocator guarantee.
    /// Fourfold encoding/source charge plus512KiB covers parsed static config
    /// structures while shared Arcs are charged once by the existing worker.
    pub(crate) fn retained_bytes(&self)->usize {self.logical_bytes.saturating_add(self.source.logical_section_bytes()).saturating_mul(4).saturating_add(512*1024)}
    pub(crate) fn geography(&self)->Option<Arc<crate::signin_geography::Policy>>{self.geography.as_ref().ok().cloned()}
    pub(crate) fn trust(&self)->Option<Arc<crate::trust_tier::Policy>>{self.trust.as_ref().ok().cloned()}
    pub(crate) fn global_knobs(&self)->crate::rate_leeway::Knobs {self.global_knobs}
    pub(crate) fn watch(&self)->Arc<crate::session_watch::Config>{self.watch.clone()}
    pub(crate) fn protocol(&self)->Arc<crate::protocol_guard::config::Config>{self.protocol.clone()}
    pub(crate) fn probe(&self)->Option<Arc<crate::admin_probe::config::Policy>>{self.probe.as_ref().ok().cloned()}
    pub(crate) fn protection(&self)->Option<Arc<crate::request_protection::config::Config>>{self.protection.as_ref().ok().cloned()}
    pub(crate) fn global(&self, key: &'static str) -> Option<&registry::Effective> {
        self.globals.get(key)
    }
    pub(crate) fn inbound(&self) -> Result<&inbound_allowlist::PublishedInputs, ()> {
        self.inbound.as_ref().map_err(|_| ())
    }
    pub(crate) fn client(&self) -> Result<&client_access::PublishedInputs, ()> {
        self.client.as_ref().map_err(|_| ())
    }
    pub(crate) fn maintenance_arc(&self)->Option<Arc<maintenance_mode::Config>> {self.maintenance.as_ref().ok().cloned()}
    pub(crate) fn maintenance(&self) -> Result<&maintenance_mode::Config, ()> {
        self.maintenance.as_deref().map_err(|_| ())
    }
    /// Source bans retain the original admin parser/admission path until its
    /// own pure seam is reviewed; this source value alone cannot bypass it.
    pub(crate) fn source_bans_raw<'a>(
        &'a self,
        store: &Store,
    ) -> Result<Option<&'a str>, StoreError> {
        self.source.raw(store, GuardInputSection::SourceBans)
    }
    pub(crate) fn logical_policy_bytes(&self) -> usize {
        self.logical_bytes
    }
}
struct CountBytes(usize);
impl io::Write for CountBytes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self
            .0
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_PREPARED_BYTES)
            .ok_or_else(|| io::Error::other("bounded guard policy encoding"))?;
        self.0 = next;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test_control {
    use std::sync::Arc;
    pub(super) struct Pause {
        pub entered: tokio::sync::Notify,
        pub release: tokio::sync::Notify,
    }
    tokio::task_local! {pub(super) static PAUSE:Arc<Pause>;}
    pub(super) async fn wait() {
        if let Ok(pause) = PAUSE.try_with(Clone::clone) {
            pause.entered.notify_one();
            pause.release.notified().await;
        }
    }
}
#[cfg(test)]
#[path = "published_guard_input_tests.rs"]
mod tests;
