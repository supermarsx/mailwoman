//! Cached deployment policy; an OFF request does not query or mutate durable counters.
use crate::{scope_mw::proxy::Cidr, settings_registry};
use mw_store::{
    Store,
    request_protection_model::{Mode, Policy, WindowPolicy},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    net::IpAddr,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

const PREFIX: &str = "security.protection.";
const CACHE_TTL: Duration = Duration::from_secs(30);
#[derive(Clone, Debug)]
pub(super) struct Config {
    base: Policy,
    client_requests: u32,
    account_requests: u32,
    admin_requests: u32,
    adaptive_clients: bool,
    adaptive_accounts: bool,
    adaptive_admins: bool,
    exempt_clients: Vec<Cidr>,
    exempt_accounts: BTreeSet<String>,
    exempt_admins: BTreeSet<String>,
    pub fail_closed: bool,
}
impl Config {
    pub fn summary(&self) -> Value {
        let mode = |mode| match mode {
            Mode::Off => "off",
            Mode::Observe => "observe",
            Mode::Enforce => "enforce",
        };
        serde_json::json!({"rateMode":mode(self.base.rate_mode),"adaptiveMode":mode(self.base.adaptive_mode),"clientRequests":self.client_requests,"accountRequests":self.account_requests,"adminRequests":self.admin_requests,"rateWindowSecs":self.base.rate_window_secs,"adaptiveClients":self.adaptive_clients,"adaptiveAccounts":self.adaptive_accounts,"adaptiveAdmins":self.adaptive_admins,"short":self.base.short,"long":self.base.long,"normalCapPerBucket":self.base.normal_cap_per_bucket,"signals":self.base.signals,"throttleSecs":self.base.throttle_secs,"blockSecs":self.base.block_secs,"decaySecs":self.base.decay_secs,"storageFailure":if self.fail_closed {"closed"} else {"open"}})
    }
    pub fn enabled(&self) -> bool {
        !self.base.off()
    }
    pub fn account_dimension_enforced(&self) -> bool {
        (self.base.rate_mode == Mode::Enforce && self.account_requests > 0)
            || (self.base.adaptive_mode == Mode::Enforce && self.adaptive_accounts)
    }
    pub fn subject(&self, kind: &str, value: &str) -> Option<Policy> {
        let (limit, adaptive) = match kind {
            "client" => {
                let ip: IpAddr = value.parse().ok()?;
                if self
                    .exempt_clients
                    .iter()
                    .any(|network| network.contains(ip))
                {
                    return None;
                }
                (self.client_requests, self.adaptive_clients)
            }
            "account" if !self.exempt_accounts.contains(value) => {
                (self.account_requests, self.adaptive_accounts)
            }
            "admin" if !self.exempt_admins.contains(value) => {
                (self.admin_requests, self.adaptive_admins)
            }
            _ => return None,
        };
        let mut policy = self.base.clone();
        policy.rate_limit = limit;
        if limit == 0 {
            policy.rate_mode = Mode::Off;
        }
        if !adaptive {
            policy.adaptive_mode = Mode::Off;
        }
        (!policy.off()).then_some(policy)
    }
    fn parse(rows: Vec<settings_registry::Effective>) -> Result<Self, ()> {
        let values: BTreeMap<String, Value> =
            rows.into_iter().map(|row| (row.key, row.value)).collect();
        let get = |key: &str| values.get(&format!("{PREFIX}{key}")).ok_or(());
        let number = |key: &str| -> Result<u32, ()> {
            get(key)?
                .as_u64()
                .and_then(|value| u32::try_from(value).ok())
                .ok_or(())
        };
        let boolean = |key: &str| get(key)?.as_bool().ok_or(());
        let text = |key: &str| get(key)?.as_str().ok_or(());
        let list = |key: &str| -> Result<Vec<String>, ()> {
            get(key)?
                .as_array()
                .ok_or(())?
                .iter()
                .map(|value| value.as_str().map(str::to_owned).ok_or(()))
                .collect()
        };
        let mode = |key: &str| match text(key)? {
            "off" => Ok(Mode::Off),
            "observe" => Ok(Mode::Observe),
            "enforce" => Ok(Mode::Enforce),
            _ => Err(()),
        };
        let window = |name: &str| -> Result<WindowPolicy, ()> {
            Ok(WindowPolicy {
                seconds: number(&format!("{name}WindowSecs"))?,
                minimum_samples: number(&format!("{name}MinimumSamples"))?,
                absolute_anomalies: number(&format!("{name}AbsoluteAnomalies"))?,
                ratio_percent: u8::try_from(number(&format!("{name}RatioPercent"))?)
                    .map_err(|_| ())?,
            })
        };
        let mut signals = 0;
        for signal in list("signals")? {
            signals |= match signal.as_str() {
                "malformed" => 1,
                "oversized" => 2,
                "credential-rejected" => 4,
                "permission-denied" => 8,
                "body-deadline" => 16,
                "admin-route-miss" => 32,
                _ => return Err(()),
            };
        }
        let base = Policy {
            rate_mode: mode("rateMode")?,
            adaptive_mode: mode("adaptiveMode")?,
            rate_limit: 0,
            rate_window_secs: number("rateWindowSecs")?,
            short: window("short")?,
            long: window("long")?,
            normal_cap_per_bucket: number("normalCapPerBucket")?,
            signals,
            throttle_secs: number("throttleSecs")?,
            block_secs: number("blockSecs")?,
            decay_secs: number("decaySecs")?,
        };
        if !base.valid() {
            return Err(());
        }
        Ok(Self {
            base,
            client_requests: number("clientRequests")?,
            account_requests: number("accountRequests")?,
            admin_requests: number("adminRequests")?,
            adaptive_clients: boolean("adaptiveClients")?,
            adaptive_accounts: boolean("adaptiveAccounts")?,
            adaptive_admins: boolean("adaptiveAdmins")?,
            exempt_clients: list("exemptClients")?
                .iter()
                .map(|raw| Cidr::parse(raw).ok_or(()))
                .collect::<Result<_, _>>()?,
            exempt_accounts: list("exemptAccounts")?.into_iter().collect(),
            exempt_admins: list("exemptAdmins")?.into_iter().collect(),
            fail_closed: match text("storageFailure")? {
                "closed" => true,
                "open" => false,
                _ => return Err(()),
            },
        })
    }
}
struct Cached {
    at: Instant,
    epoch: u64,
    value: Arc<Config>,
}
#[derive(Default)]
pub(super) struct Cache {
    value: RwLock<Option<Cached>>,
    epoch: AtomicU64,
    refresh: tokio::sync::Mutex<()>,
}
impl Cache {
    pub fn invalidate(&self) {
        self.epoch.fetch_add(1, Ordering::AcqRel);
    }
    fn current(&self) -> Option<Arc<Config>> {
        let guard = self
            .value
            .read()
            .unwrap_or_else(|poison| poison.into_inner());
        guard
            .as_ref()
            .filter(|value| {
                value.epoch == self.epoch.load(Ordering::Acquire) && value.at.elapsed() < CACHE_TTL
            })
            .map(|value| value.value.clone())
    }
    pub async fn get(&self, store: &Store) -> Result<Arc<Config>, ()> {
        if let Some(value) = self.current() {
            return Ok(value);
        }
        let _lock = self.refresh.lock().await;
        if let Some(value) = self.current() {
            return Ok(value);
        }
        let epoch = self.epoch.load(Ordering::Acquire);
        let rows = settings_registry::effective_all(store, &Default::default(), |definition| {
            definition.key.starts_with(PREFIX)
        })
        .await
        .map_err(|_| ())?;
        let value = Arc::new(Config::parse(rows)?);
        // A concurrent successful policy write invalidates this snapshot. Never
        // promote a read begun before that write into the current generation.
        if epoch != self.epoch.load(Ordering::Acquire) {
            return Err(());
        }
        *self
            .value
            .write()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(Cached {
            at: Instant::now(),
            epoch,
            value: value.clone(),
        });
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::{ServerKey, settings_policy::SettingPolicyRow};
    async fn rows(store: &Store) -> Vec<settings_registry::Effective> {
        settings_registry::effective_all(store, &Default::default(), |row| {
            row.key.starts_with(PREFIX)
        })
        .await
        .unwrap()
    }
    #[tokio::test]
    async fn defaults_are_off_and_exemptions_do_not_cross_subject_dimensions() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let mut values = rows(&store).await;
        assert!(!Config::parse(values.clone()).unwrap().enabled());
        for (key, value) in [
            ("rateMode", serde_json::json!("enforce")),
            ("exemptClients", serde_json::json!(["192.0.2.0/24"])),
            ("exemptAccounts", serde_json::json!(["owned-A"])),
        ] {
            values
                .iter_mut()
                .find(|row| row.key == format!("{PREFIX}{key}"))
                .unwrap()
                .value = value;
        }
        let config = Config::parse(values).unwrap();
        assert!(config.subject("client", "::ffff:192.0.2.1").is_none());
        assert!(config.subject("account", "owned-A").is_none());
        assert!(config.subject("account", "owned-B").is_some());
        assert!(config.subject("admin", "owned-A").is_some());
        assert!(config.subject("client", "198.51.100.1").is_some());
    }
    #[tokio::test]
    async fn policy_cache_invalidates_locally_and_expires_for_other_replicas() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let cache = Cache::default();
        assert!(!cache.get(&store).await.unwrap().enabled());
        let row = SettingPolicyRow {
            key: format!("{PREFIX}rateMode"),
            global_value: Some(serde_json::json!("enforce").to_string()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "test".into(),
            updated_at: 1,
        };
        assert!(
            store
                .compare_setting_policy(&row.key, None, Some(&row))
                .await
                .unwrap()
        );
        assert!(!cache.get(&store).await.unwrap().enabled());
        cache.invalidate();
        assert!(cache.get(&store).await.unwrap().enabled());
        assert!(
            store
                .compare_setting_policy(&row.key, Some(&row), None)
                .await
                .unwrap()
        );
        cache.value.write().unwrap().as_mut().unwrap().at =
            Instant::now() - Duration::from_secs(31);
        assert!(!cache.get(&store).await.unwrap().enabled());
    }
    #[tokio::test]
    async fn missing_proxy_realm_refuses_only_enforcing_account_policies() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        for mode in ["off", "observe", "enforce"] {
            let mut values = rows(&store).await;
            values
                .iter_mut()
                .find(|row| row.key == format!("{PREFIX}rateMode"))
                .unwrap()
                .value = serde_json::json!(mode);
            assert_eq!(
                Config::parse(values).unwrap().account_dimension_enforced(),
                mode == "enforce"
            );
        }
    }
    #[test]
    fn network_exemptions_and_signals_are_validated_before_storage() {
        for (key, value, valid) in [
            (
                "exemptClients",
                serde_json::json!(["192.0.2.0/24", "2001:db8::/32"]),
                true,
            ),
            (
                "exemptClients",
                serde_json::json!(["arbitrary-proxy-header"]),
                false,
            ),
            (
                "exemptAccounts",
                serde_json::json!(["Account_A:verified"]),
                true,
            ),
            (
                "signals",
                serde_json::json!(["malformed", "credential-rejected"]),
                true,
            ),
            ("signals", serde_json::json!(["admin-route-miss"]), true),
            ("signals", serde_json::json!(["http-404"]), false),
        ] {
            let definition = settings_registry::lookup(&format!("{PREFIX}{key}")).unwrap();
            assert_eq!(
                settings_registry::validate(definition, &value, "/value").is_ok(),
                valid,
                "{key}"
            );
        }
    }
}
