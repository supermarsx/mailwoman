//! Bounded, best-effort event counts. Identity is retained locally only to recheck consent.
use crate::{
    AppState,
    analytics_adapter::{self, Screen},
    analytics_config::{self, Config},
};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;

pub struct Runtime {
    slots: Arc<Semaphore>,
    pending: AtomicUsize,
    delivered: AtomicU64,
    dropped: AtomicU64,
    rates: Mutex<HashMap<String, (Instant, u32)>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            slots: Arc::new(Semaphore::new(2)),
            pending: AtomicUsize::new(0),
            delivered: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            rates: Mutex::new(HashMap::new()),
        }
    }
}
pub async fn consent(store: &mw_store::Store, account: &str) -> bool {
    store
        .get_setting(&format!("ui-preferences:{account}"))
        .await
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .is_some_and(|value| {
            value["document"]["version"] == 1
                && value["document"]["preferences"]["analyticsConsent"] == true
        })
}
pub fn privacy_signal(headers: &axum::http::HeaderMap) -> bool {
    ["dnt", "sec-gpc"]
        .iter()
        .any(|key| headers.get(*key).is_some_and(|value| value == "1"))
}
async fn current_consent_and_config(
    store: &mw_store::Store,
    account: &str,
    expected: &Config,
) -> bool {
    expected.enabled
        && analytics_config::load(store)
            .await
            .is_ok_and(|current| current == *expected)
        && consent(store, account).await
}
impl Runtime {
    pub fn status(&self) -> serde_json::Value {
        serde_json::json!({"queued":self.pending.load(Ordering::Relaxed),"delivered":self.delivered.load(Ordering::Relaxed),"dropped":self.dropped.load(Ordering::Relaxed),"mode":"event-counts","uniqueVisitors":null})
    }
    fn admit(&self, account: &str) -> bool {
        let mut rates = self.rates.lock().expect("analytics rate lock");
        let now = Instant::now();
        rates.retain(|_, (start, _)| now.duration_since(*start) < Duration::from_secs(60));
        if rates.len() >= 1024 && !rates.contains_key(account) {
            return false;
        }
        if rates.get("").is_some_and(|(_, count)| *count >= 600)
            || rates.get(account).is_some_and(|(_, count)| *count >= 60)
        {
            return false;
        }
        for key in ["", account] {
            rates.entry(key.into()).or_insert((now, 0)).1 += 1;
        }
        true
    }
    pub fn enqueue(
        self: &Arc<Self>,
        state: AppState,
        session: mw_store::Session,
        config: Config,
        screen: Screen,
    ) -> bool {
        if !self.admit(&session.account_id) {
            return false;
        }
        if self
            .pending
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                (n < 256).then_some(n + 1)
            })
            .is_err()
        {
            return false;
        }
        let runtime = self.clone();
        let session_id = session.id;
        let account_id = session.account_id;
        tokio::spawn(async move {
            let outcome = tokio::time::timeout(Duration::from_secs(10), async {
                let _slot = runtime.slots.acquire().await.map_err(|_| ())?;
                if !consent(&state.store, &account_id).await {
                    return Err(());
                }
                let mut current = state.store.get_session(&session_id).await.map_err(|_| ())?;
                if current.account_id != account_id
                    || crate::security_center::enforce_session(&state, &current)
                        .await
                        .is_err()
                {
                    return Err(());
                }
                crate::domain_upgrade::refresh_session(&state, &mut current)
                    .await
                    .map_err(|_| ())?;
                let gate = crate::account_gate::for_account(&state.store, &account_id)
                    .await
                    .map_err(|_| ())?;
                if gate.disabled || gate.password_change_required {
                    return Err(());
                }
                // Alias/policy edits can occur while this job waits. Resolve the
                // authenticated identity's route again immediately before export.
                let identity = state
                    .store
                    .security_account_address(&account_id)
                    .await
                    .map_err(|_| ())?
                    .ok_or(())?;
                let domain = match identity.rsplit_once('@') {
                    Some((_, domain)) => Some(
                        state
                            .store
                            .canonical_policy_domain(domain)
                            .await
                            .map_err(|_| ())?,
                    ),
                    None => None,
                };
                if !current_consent_and_config(&state.store, &account_id, &config).await {
                    return Err(());
                }
                analytics_adapter::deliver(&state.store, &config, screen, domain.as_deref())
                    .await
                    .map_err(|_| ())
            })
            .await;
            if matches!(outcome, Ok(Ok(()))) {
                runtime.delivered.fetch_add(1, Ordering::Relaxed);
            } else {
                runtime.dropped.fetch_add(1, Ordering::Relaxed);
            }
            runtime.pending.fetch_sub(1, Ordering::SeqCst);
        });
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn account_and_global_rate_windows_are_bounded() {
        let runtime = Runtime::default();
        for _ in 0..60 {
            assert!(runtime.admit("a"));
        }
        assert!(!runtime.admit("a"));
        for account in 1..10 {
            for _ in 0..60 {
                assert!(runtime.admit(&account.to_string()));
            }
        }
        assert!(!runtime.admit("new-account"));
        assert_eq!(runtime.rates.lock().unwrap().len(), 11);
        let mut headers = axum::http::HeaderMap::new();
        assert!(!privacy_signal(&headers));
        headers.insert("sec-gpc", "1".parse().unwrap());
        assert!(privacy_signal(&headers));
        headers.clear();
        headers.insert("dnt", "1".parse().unwrap());
        assert!(privacy_signal(&headers));
    }
    #[tokio::test]
    async fn consent_is_explicit_versioned_and_account_bound() {
        let store = mw_store::Store::open(
            ":memory:",
            mw_store::ServerKey::from_hex(&"11".repeat(32)).unwrap(),
        )
        .await
        .unwrap();
        assert!(!consent(&store, "a").await);
        for value in [
            r#"{}"#,
            r#"{"document":{"version":2,"preferences":{"analyticsConsent":true}}}"#,
            r#"{"document":{"version":1,"preferences":{"analyticsConsent":"true"}}}"#,
        ] {
            store.set_setting("ui-preferences:a", value).await.unwrap();
            assert!(!consent(&store, "a").await);
        }
        store
            .set_setting(
                "ui-preferences:a",
                r#"{"document":{"version":1,"preferences":{"analyticsConsent":true}}}"#,
            )
            .await
            .unwrap();
        assert!(consent(&store, "a").await);
        assert!(!consent(&store, "b").await);
        let config = Config {
            enabled: true,
            provider: analytics_config::Provider::Matomo,
            endpoint: "https://stats.example/matomo.php".into(),
            site_id: "1".into(),
            allow_http: false,
            api_secret_sealed: None,
        };
        store
            .set_setting(
                analytics_config::KEY,
                &serde_json::to_string(&config).unwrap(),
            )
            .await
            .unwrap();
        assert!(current_consent_and_config(&store, "a", &config).await);
        let mut changed = config.clone();
        changed.site_id = "2".into();
        store
            .set_setting(
                analytics_config::KEY,
                &serde_json::to_string(&changed).unwrap(),
            )
            .await
            .unwrap();
        assert!(!current_consent_and_config(&store, "a", &config).await);
        store
            .set_setting(
                analytics_config::KEY,
                &serde_json::to_string(&config).unwrap(),
            )
            .await
            .unwrap();
        store
            .set_setting(
                "ui-preferences:a",
                r#"{"document":{"version":1,"preferences":{"analyticsConsent":false}}}"#,
            )
            .await
            .unwrap();
        assert!(!consent(&store, "a").await);
        assert!(!current_consent_and_config(&store, "a", &config).await);
    }
}
