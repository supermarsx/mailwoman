use crate::{
    BoxFuture, CachedSts, DaneMode, PolicyError, StsCache, StsMode, StsPolicy, StsPolicyMode,
    TransportPolicyConfig, normalize_domain,
};
use hickory_net::proto::rr::rdata::TLSA;
use mw_egress::{
    FetchTimeouts, ResponseMetadata, TrafficClass,
    plan::{Request, send},
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;

#[derive(Debug, Clone)]
pub struct PolicyContext {
    pub doh_url: String,
    pub egress_domain: Option<String>,
    /// Stable authority namespace, not a credential. Distinct routes never share policies.
    pub cache_namespace: String,
}
#[derive(Debug, Clone, Default)]
pub struct TransportDecision {
    pub(crate) tlsa: Vec<TLSA>,
    pub(crate) sts_enforce: bool,
    pub(crate) dane_tls: bool,
    pub(crate) dane_name: Option<String>,
}
impl TransportDecision {
    pub fn requires_tls(&self) -> bool {
        self.sts_enforce || self.dane_tls
    }
    pub fn has_dane(&self) -> bool {
        !self.tlsa.is_empty()
    }
}
pub struct TransportPolicy {
    config: TransportPolicyConfig,
    context: PolicyContext,
    cache: Arc<dyn StsCache>,
    io: Arc<dyn PolicyIo>,
}
impl std::fmt::Debug for TransportPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransportPolicy")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}
trait PolicyIo: Send + Sync {
    fn txt<'a>(
        &'a self,
        context: &'a PolicyContext,
        name: &'a str,
    ) -> BoxFuture<'a, Result<Vec<String>, PolicyError>>;
    fn fetch<'a>(
        &'a self,
        context: &'a PolicyContext,
        domain: &'a str,
    ) -> BoxFuture<'a, Result<StsPolicy, PolicyError>>;
    fn dane<'a>(
        &'a self,
        context: &'a PolicyContext,
        host: &'a str,
        port: u16,
    ) -> BoxFuture<'a, Result<crate::dns::DaneRecords, PolicyError>>;
}
struct RoutedIo;
impl PolicyIo for RoutedIo {
    fn txt<'a>(
        &'a self,
        c: &'a PolicyContext,
        n: &'a str,
    ) -> BoxFuture<'a, Result<Vec<String>, PolicyError>> {
        Box::pin(crate::dns::txt(c, n))
    }
    fn dane<'a>(
        &'a self,
        c: &'a PolicyContext,
        h: &'a str,
        p: u16,
    ) -> BoxFuture<'a, Result<crate::dns::DaneRecords, PolicyError>> {
        Box::pin(crate::dns::dane(c, h, p))
    }
    fn fetch<'a>(
        &'a self,
        c: &'a PolicyContext,
        d: &'a str,
    ) -> BoxFuture<'a, Result<StsPolicy, PolicyError>> {
        Box::pin(async move {
            let domain = normalize_domain(d)?;
            let url =
                reqwest::Url::parse(&format!("https://mta-sts.{domain}/.well-known/mta-sts.txt"))
                    .map_err(|_| PolicyError::Sts)?;
            let metadata = ResponseMetadata::default();
            let mut request = Request::get(TrafficClass::DnsAuth)
                .accept("text/plain")
                .response_limit(crate::sts::MAX_POLICY_BYTES)
                .timeouts(timeouts())
                .capture_response(&metadata);
            if let Some(domain) = &c.egress_domain {
                request = request.for_domain(domain);
            }
            let response = send(url, &request).await;
            if metadata.snapshot().map(|m| m.status) != Some(200)
                || !response
                    .content_type
                    .as_deref()
                    .and_then(|v| v.split(';').next())
                    .is_some_and(|v| v.trim().eq_ignore_ascii_case("text/plain"))
            {
                return Err(PolicyError::Sts);
            }
            StsPolicy::parse(&response.outcome.map_err(|_| PolicyError::Sts)?)
        })
    }
}
pub(crate) fn timeouts() -> FetchTimeouts {
    FetchTimeouts {
        connect: Duration::from_secs(5),
        headers: Duration::from_secs(8),
        idle: Duration::from_secs(5),
        total: Duration::from_secs(12),
        max_redirects: 0,
    }
}
#[derive(Default)]
struct RefreshState {
    last_attempt: u64,
    last_id: Option<String>,
    last_warning: Option<u64>,
}
type Locks = Mutex<HashMap<String, Arc<tokio::sync::Mutex<RefreshState>>>>;
fn refresh_lock(key: &str) -> Result<Arc<tokio::sync::Mutex<RefreshState>>, PolicyError> {
    static LOCKS: OnceLock<Locks> = OnceLock::new();
    let mut locks = LOCKS
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| PolicyError::Cache)?;
    if let Some(lock) = locks.get(key) {
        return Ok(lock.clone());
    }
    // Idle synchronization entries are disposable; persisted policies are not.
    if locks.len() >= 1024 {
        locks.retain(|_, v| Arc::strong_count(v) > 1);
    }
    if locks.len() >= 1024 {
        return Err(PolicyError::Cache);
    }
    let lock = Arc::new(tokio::sync::Mutex::new(RefreshState::default()));
    locks.insert(key.into(), lock.clone());
    Ok(lock)
}
impl TransportPolicy {
    pub fn new(
        config: TransportPolicyConfig,
        context: PolicyContext,
        cache: Arc<dyn StsCache>,
    ) -> Result<Self, PolicyError> {
        config.validate()?;
        let url = reqwest::Url::parse(&context.doh_url).map_err(|_| PolicyError::Configuration)?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || context.cache_namespace.len() > 1024
        {
            return Err(PolicyError::Configuration);
        }
        Ok(Self {
            config,
            context,
            cache,
            io: Arc::new(RoutedIo),
        })
    }
    /// Called for every new SMTP connection, before opening a socket or sending AUTH.
    /// An expired policy is never extended merely because discovery is unavailable.
    pub async fn prepare(&self, host: &str, port: u16) -> Result<TransportDecision, PolicyError> {
        if !self.config.enabled() {
            return Ok(TransportDecision::default());
        }
        let host = normalize_domain(host)?;
        static ADMISSION: OnceLock<tokio::sync::Semaphore> = OnceLock::new();
        tokio::time::timeout(Duration::from_secs(25), async {
            let _permit = ADMISSION
                .get_or_init(|| tokio::sync::Semaphore::new(8))
                .acquire()
                .await
                .map_err(|_| PolicyError::Timeout)?;
            self.prepare_inner(
                &host,
                port,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| PolicyError::Configuration)?
                    .as_secs(),
            )
            .await
        })
        .await
        .map_err(|_| PolicyError::Timeout)?
    }
    async fn prepare_inner(
        &self,
        host: &str,
        port: u16,
        now: u64,
    ) -> Result<TransportDecision, PolicyError> {
        let mut decision = TransportDecision::default();
        if self.config.dane != DaneMode::Off {
            let found = self.io.dane(&self.context, host, port).await?;
            decision.dane_tls = !found.records.is_empty();
            decision.dane_name = found.base;
            decision.tlsa = found
                .records
                .into_iter()
                .filter(crate::tlsa::usable)
                .collect();
            if decision.tlsa.is_empty() && self.config.dane == DaneMode::Required {
                return Err(PolicyError::DaneRequired);
            }
        }
        // RFC 8461: authenticated usable DANE takes precedence over STS.
        // Failure to match these records never falls back to a weaker policy.
        if self.config.mta_sts != StsMode::Off && !decision.has_dane() {
            let policy = self.sts(now).await?;
            match policy {
                Some(p) if p.mode == StsPolicyMode::Enforce => {
                    if !p.matches(host) {
                        return Err(PolicyError::MxMismatch);
                    }
                    decision.sts_enforce = true;
                }
                _ if self.config.mta_sts == StsMode::Required => {
                    return Err(PolicyError::StsRequired);
                }
                _ => {}
            }
        }
        Ok(decision)
    }
    async fn sts(&self, now: u64) -> Result<Option<StsPolicy>, PolicyError> {
        let domain = normalize_domain(
            self.config
                .policy_domain
                .as_deref()
                .ok_or(PolicyError::Configuration)?,
        )?;
        let key = Sha256::digest(format!(
            "{}\0{}\0{domain}",
            self.context.cache_namespace,
            self.context.egress_domain.as_deref().unwrap_or("")
        ))
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
        let lock = refresh_lock(&key)?;
        let mut state = lock.lock().await;
        let cached = self.cache.load(&key).await?;
        if cached
            .as_ref()
            .is_some_and(|p| p.expires_at > now && !p.valid_at(now))
        {
            return Err(PolicyError::Cache);
        }
        // Retain an authenticated domain/id hint for periodic HTTPS refresh,
        // even when DNS TXT discovery has been stripped. Expired protection is
        // not applied, and only successful HTTPS can establish a new lifetime.
        let refresh_id = cached
            .as_ref()
            .filter(|p| {
                p.fetched_at <= now
                    && p.valid_at(p.fetched_at)
                    && now.saturating_sub(p.fetched_at) >= 86400
            })
            .map(|p| p.id.clone());
        let cached = cached.filter(|p| p.valid_at(now));
        let records = self
            .io
            .txt(&self.context, &format!("_mta-sts.{domain}"))
            .await;
        let id = records.and_then(|r| crate::sts::discovery_id(&r));
        let id = match id {
            Ok(Some(id)) => id,
            _ => match refresh_id {
                Some(id) => id,
                None => return Ok(cached.map(|p| p.policy)),
            },
        };
        if let Some(value) = &cached
            && value.id == id
            && now.saturating_sub(value.fetched_at) < 86400
        {
            return Ok(Some(value.policy.clone()));
        }
        if state.last_id.as_deref() == Some(&id) && now.saturating_sub(state.last_attempt) < 300 {
            return Ok(cached.map(|p| p.policy));
        }
        state.last_attempt = now;
        state.last_id = Some(id.clone());
        match self.io.fetch(&self.context, &domain).await {
            Ok(policy) => {
                let value = CachedSts {
                    id,
                    expires_at: now.saturating_add(policy.max_age),
                    fetched_at: now,
                    policy,
                };
                self.cache.store(&key, &value).await?;
                // Another process can publish a newer policy while this HTTPS
                // request is running. Store may correctly refuse our stale
                // write; never apply that rejected policy to this connection.
                let authoritative = self.cache.load(&key).await?.ok_or(PolicyError::Cache)?;
                if authoritative != value && !authoritative.valid_at(now) {
                    return Err(PolicyError::Cache);
                }
                Ok(Some(authoritative.policy))
            }
            Err(_) => {
                if cached
                    .as_ref()
                    .is_some_and(|p| p.policy.mode == StsPolicyMode::Enforce)
                    && state
                        .last_warning
                        .is_none_or(|last| now.saturating_sub(last) >= 300)
                {
                    state.last_warning = Some(now);
                    tracing::warn!(policy_key=%key,"MTA-STS enforcing policy refresh failed; retaining unexpired cached protection");
                }
                Ok(cached.map(|p| p.policy))
            }
        }
    }
}
