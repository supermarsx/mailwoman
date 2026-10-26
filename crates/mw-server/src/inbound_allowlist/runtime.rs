use super::{
    config::{Config, KEY, Policy},
    dns,
};
use futures_util::{StreamExt, stream};
use std::{
    collections::BTreeMap,
    net::IpAddr,
    sync::{Arc, Weak},
    time::{Duration, Instant},
};
use tokio::{sync::Mutex, task::JoinHandle};

#[derive(Default)]
pub(crate) struct Runtime {
    pub(super) inner: Mutex<Snapshot>,
}
#[derive(Default)]
pub(super) struct Snapshot {
    pub config: Option<Config>,
    entries: BTreeMap<String, Entry>,
    next: Option<Instant>,
    job: Option<JoinHandle<()>>,
}
struct Entry {
    answer: Option<dns::Answer>,
    error: Option<&'static str>,
}
impl Drop for Snapshot {
    fn drop(&mut self) {
        if let Some(job) = self.job.take() {
            job.abort();
        }
    }
}
impl Snapshot {
    pub fn replace(&mut self, config: Config) {
        if self.config.as_ref() == Some(&config) {
            return;
        }
        if let Some(job) = self.job.take() {
            job.abort();
        }
        self.entries.clear();
        self.next = None;
        self.config = Some(config);
    }
    fn allows(&self, policy: &Policy, ip: Option<IpAddr>, now: Instant) -> bool {
        if !policy.enabled {
            return true;
        }
        let Some(ip) = ip.map(|ip| ip.to_canonical()) else {
            return false;
        };
        policy.cidrs.iter().any(|cidr| {
            crate::scope_mw::proxy::Cidr::parse(cidr).is_some_and(|cidr| cidr.contains(ip))
        }) || policy.hostnames.iter().any(|host| {
            self.entries
                .get(host)
                .and_then(|e| e.answer.as_ref())
                .is_some_and(|answer| answer.until > now && answer.ips.contains(&ip))
        })
    }
    pub fn status(&self) -> serde_json::Value {
        let now = Instant::now();
        let rows: Vec<_> = self.config.as_ref().map(Config::hosts).unwrap_or_default().into_iter().map(|host| {
            let entry = self.entries.get(&host);
            let answer = entry.and_then(|e| e.answer.as_ref()).filter(|a| a.until > now);
            serde_json::json!({"hostname":host,"addresses":answer.map(|a| &a.ips),"validForSeconds":answer.map(|a| a.until.saturating_duration_since(now).as_secs()),"error":entry.and_then(|e| e.error),"usable":answer.is_some()})
        }).collect();
        serde_json::json!({"hosts":rows,"refreshing":self.job.as_ref().is_some_and(|j| !j.is_finished())})
    }
}
impl Runtime {
    pub async fn admit(
        self: &Arc<Self>,
        store: &mw_store::Store,
        admin: bool,
        ip: Option<IpAddr>,
    ) -> Result<bool, ()> {
        // Serialize with local CAS edits so an older read cannot restore revoked entries.
        let mut snapshot = tokio::time::timeout(Duration::from_secs(2), self.inner.lock())
            .await
            .map_err(|_| ())?;
        let raw = tokio::time::timeout(Duration::from_secs(2), store.get_setting(KEY))
            .await
            .map_err(|_| ())?
            .map_err(|_| ())?;
        let config = Config::parse(raw.as_deref())?;
        snapshot.replace(config.clone());
        self.refresh(&mut snapshot, &config);
        Ok(snapshot.allows(
            if admin {
                &config.admin
            } else {
                &config.webmail
            },
            ip,
            Instant::now(),
        ))
    }
    pub(super) fn refresh(self: &Arc<Self>, snapshot: &mut Snapshot, config: &Config) {
        let now = Instant::now();
        if snapshot.job.as_ref().is_some_and(|job| !job.is_finished())
            || snapshot.next.is_some_and(|next| next > now)
            || config.hosts().is_empty()
        {
            return;
        }
        snapshot.next = Some(now + Duration::from_secs(1));
        let weak = Arc::downgrade(self);
        let config = config.clone();
        snapshot.job = Some(tokio::spawn(refresh(weak, config)));
    }
}
async fn refresh(runtime: Weak<Runtime>, config: Config) {
    let resolver = dns::resolver(&config.dns);
    let mut results = stream::iter(config.hosts())
        .map(|host| {
            let resolver = &resolver;
            let dns = &config.dns;
            async move {
                let result = match resolver {
                    Ok(resolver) => dns::lookup(resolver, dns, &host).await,
                    Err(error) => Err(*error),
                };
                (host, result)
            }
        })
        .buffer_unordered(config.dns.concurrency);
    // Publish each completed lookup: another host's timeout must not consume
    // this answer's TTL before it can be used. No mutex is held across DNS IO.
    while let Some((host, result)) = results.next().await {
        let Some(owner) = runtime.upgrade() else {
            return;
        };
        let mut snapshot = owner.inner.lock().await;
        if snapshot.config.as_ref() != Some(&config) {
            return;
        }
        let now = Instant::now();
        let entry = snapshot.entries.entry(host).or_insert(Entry {
            answer: None,
            error: None,
        });
        match result {
            Ok(answer) => {
                entry.answer = Some(answer);
                entry.error = None;
            }
            Err(error) => {
                entry.error = Some(error);
                if entry.answer.as_ref().is_some_and(|a| a.until <= now) {
                    entry.answer = None;
                }
            }
        }
    }
    let Some(owner) = runtime.upgrade() else {
        return;
    };
    let mut snapshot = owner.inner.lock().await;
    if snapshot.config.as_ref() != Some(&config) {
        return;
    }
    let now = Instant::now();
    let delay = snapshot
        .entries
        .values()
        .filter_map(|entry| entry.answer.as_ref())
        .map(|answer| answer.until.saturating_duration_since(now) / 2)
        .min()
        .unwrap_or(Duration::from_secs(30))
        .clamp(Duration::from_secs(1), Duration::from_secs(30));
    snapshot.next = Some(now + delay);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn replacing_policy_cancels_owned_refresh_work() {
        use std::sync::atomic::{AtomicBool, Ordering};
        struct Cancelled(Arc<AtomicBool>);
        impl Drop for Cancelled {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        let marker = Cancelled(cancelled.clone());
        let (started, ready) = tokio::sync::oneshot::channel();
        let mut snapshot = Snapshot::default();
        snapshot.replace(Config::default());
        snapshot.job = Some(tokio::spawn(async move {
            let _marker = marker;
            let _ = started.send(());
            std::future::pending::<()>().await;
        }));
        ready.await.unwrap();
        let mut replacement = Config::default();
        replacement.dns.cache_seconds = 60;
        snapshot.replace(replacement);
        tokio::time::timeout(Duration::from_secs(1), async {
            while !cancelled.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(snapshot.job.is_none());
    }
    #[tokio::test]
    async fn database_revocation_unknown_source_and_corruption_are_authoritative() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let runtime = Arc::new(Runtime::default());
        let ip = Some("127.0.0.1".parse().unwrap());
        assert!(runtime.admit(&store, true, None).await.unwrap());
        let mut config = Config::default();
        config.admin.enabled = true;
        config.admin.cidrs = vec!["127.0.0.1".into()];
        let first = serde_json::to_string(&config).unwrap();
        assert!(
            store
                .compare_and_set_setting(KEY, None, &first)
                .await
                .unwrap()
        );
        assert!(runtime.admit(&store, true, ip).await.unwrap());
        assert!(!runtime.admit(&store, true, None).await.unwrap());
        assert!(runtime.admit(&store, false, None).await.unwrap());
        config.admin.cidrs = vec!["192.0.2.1".into()];
        let second = serde_json::to_string(&config).unwrap();
        assert!(
            store
                .compare_and_set_setting(KEY, Some(&first), &second)
                .await
                .unwrap()
        );
        assert!(!runtime.admit(&store, true, ip).await.unwrap());
        assert!(
            store
                .compare_and_set_setting(KEY, Some(&second), "broken")
                .await
                .unwrap()
        );
        assert!(runtime.admit(&store, true, ip).await.is_err());
    }
    #[test]
    fn expiry_revocation_unknown_and_surface_isolation() {
        let now = Instant::now();
        let ip: IpAddr = "192.0.2.9".parse().unwrap();
        let mut config = Config::default();
        config.admin.enabled = true;
        config.admin.cidrs = vec!["127.0.0.1".into()];
        config.admin.hostnames = vec!["office.example".into()];
        let mut state = Snapshot::default();
        state.replace(config.clone());
        state.entries.insert(
            "office.example".into(),
            Entry {
                answer: Some(dns::Answer {
                    ips: vec![ip],
                    until: now + Duration::from_secs(1),
                }),
                error: None,
            },
        );
        assert!(state.allows(&config.admin, Some(ip), now));
        assert!(!state.allows(&config.admin, Some(ip), now + Duration::from_secs(1)));
        assert!(!state.allows(&config.admin, None, now));
        assert!(state.allows(&config.webmail, None, now));
        config.admin.hostnames.clear();
        state.replace(config.clone());
        assert!(!state.allows(&config.admin, Some(ip), now));
    }
}
