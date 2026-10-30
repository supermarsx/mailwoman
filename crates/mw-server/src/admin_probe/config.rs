//! The admin-probe policy, read from the `security.adminProbe.*` registry keys and
//! cached for [`CACHE_TTL`] (a write through the dedicated route invalidates it at
//! once on this instance; other instances see it within the TTL).
use super::tracker::{Limits, Signal};
use crate::{scope_mw::proxy::Cidr, settings_registry};
use mw_store::Store;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    net::IpAddr,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

pub(crate) const PREFIX: &str = "security.adminProbe.";
const CACHE_TTL: Duration = Duration::from_secs(30);

/// Every key, in the order the API lists them.
pub(crate) const KEYS: [&str; 14] = [
    "enabled",
    "graceAllowance",
    "windowSecs",
    "blockAfter",
    "tarpitDelayMs",
    "blockSecs",
    "maxBlockSecs",
    "repeatWindowSecs",
    "blockStatus",
    "signals",
    "probePaths",
    "allowlist",
    "auditIp",
    "fail2banLog",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AuditIp {
    /// IPv4 /24, IPv6 /48.
    Masked,
    Full,
}

#[derive(Clone, Debug)]
pub(crate) struct Policy {
    pub enabled: bool,
    pub limits: Limits,
    pub tarpit_ms: u32,
    pub block_status: u16,
    /// Lower-cased, each split into path segments.
    pub probe_paths: Vec<Vec<String>>,
    pub allowlist: Vec<Cidr>,
    pub audit_ip: AuditIp,
    pub fail2ban: bool,
    /// The effective values as the API shows them.
    pub values: BTreeMap<String, Value>,
}

impl Policy {
    pub fn allowlisted(&self, ip: IpAddr) -> bool {
        let ip = ip.to_canonical();
        self.allowlist.iter().any(|cidr| cidr.contains(ip))
    }

    pub fn summary(&self) -> Value {
        let mut out = serde_json::Map::new();
        for key in KEYS {
            if let Some(value) = self.values.get(key) {
                out.insert(key.to_string(), value.clone());
            }
        }
        Value::Object(out)
    }

    pub(crate) fn parse(rows: Vec<settings_registry::Effective>) -> Result<Self, ()> {
        let values: BTreeMap<String, Value> = rows
            .into_iter()
            .filter_map(|row| {
                row.key
                    .strip_prefix(PREFIX)
                    .map(|key| (key.to_string(), row.value))
            })
            .collect();
        let get = |key: &str| values.get(key).ok_or(());
        let number = |key: &str| -> Result<u32, ()> {
            get(key)?
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or(())
        };
        let boolean = |key: &str| get(key)?.as_bool().ok_or(());
        let text = |key: &str| get(key)?.as_str().ok_or(());
        let list = |key: &str| -> Result<Vec<String>, ()> {
            get(key)?
                .as_array()
                .ok_or(())?
                .iter()
                .map(|v| v.as_str().map(str::to_owned).ok_or(()))
                .collect()
        };
        let signals: Vec<Signal> = list("signals")?
            .iter()
            .map(|raw| Signal::parse(raw).ok_or(()))
            .collect::<Result<_, _>>()?;
        let limits = Limits {
            grace: number("graceAllowance")?,
            window_secs: number("windowSecs")?,
            block_after: number("blockAfter")?,
            block_secs: number("blockSecs")?,
            max_block_secs: number("maxBlockSecs")?,
            repeat_window_secs: number("repeatWindowSecs")?,
            signals: Limits::signal_mask(&signals),
        };
        if limits.window_secs == 0 || limits.block_after == 0 || limits.block_secs == 0 {
            return Err(());
        }
        Ok(Self {
            enabled: boolean("enabled")?,
            limits,
            tarpit_ms: number("tarpitDelayMs")?,
            block_status: match text("blockStatus")? {
                "429" => 429,
                "403" => 403,
                _ => return Err(()),
            },
            probe_paths: list("probePaths")?
                .iter()
                .map(|p| segments(&p.to_ascii_lowercase()))
                .filter(|s| !s.is_empty())
                .collect(),
            allowlist: list("allowlist")?
                .iter()
                .map(|raw| Cidr::parse(raw).ok_or(()))
                .collect::<Result<_, _>>()?,
            audit_ip: match text("auditIp")? {
                "masked" => AuditIp::Masked,
                "full" => AuditIp::Full,
                _ => return Err(()),
            },
            fail2ban: boolean("fail2banLog")?,
            values,
        })
    }

    /// Whether `path` (lower-cased segments) contains a probe-list entry as a
    /// contiguous run of whole segments: `/wp-admin` matches `/blog/wp-admin/x`
    /// but not `/wp-administration`.
    pub fn is_probe(&self, path: &[String]) -> bool {
        self.probe_paths.iter().any(|probe| {
            probe.len() <= path.len() && path.windows(probe.len()).any(|w| w == probe.as_slice())
        })
    }
}

pub(crate) fn segments(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

struct Cached {
    at: Instant,
    epoch: u64,
    value: Arc<Policy>,
}

#[derive(Default)]
pub(crate) struct Cache {
    value: RwLock<Option<Cached>>,
    epoch: AtomicU64,
    refresh: tokio::sync::Mutex<()>,
}

impl Cache {
    pub fn invalidate(&self) {
        self.epoch.fetch_add(1, Ordering::AcqRel);
    }
    fn current(&self) -> Option<Arc<Policy>> {
        let guard = self.value.read().unwrap_or_else(|p| p.into_inner());
        guard
            .as_ref()
            .filter(|c| c.epoch == self.epoch.load(Ordering::Acquire) && c.at.elapsed() < CACHE_TTL)
            .map(|c| c.value.clone())
    }
    /// The last policy read, however old (for the fail-open path).
    pub fn last(&self) -> Option<Arc<Policy>> {
        let guard = self.value.read().unwrap_or_else(|p| p.into_inner());
        guard.as_ref().map(|c| c.value.clone())
    }
    pub async fn get(&self, store: &Store) -> Result<Arc<Policy>, ()> {
        if let Some(value) = self.current() {
            return Ok(value);
        }
        let _lock = self.refresh.lock().await;
        if let Some(value) = self.current() {
            return Ok(value);
        }
        let epoch = self.epoch.load(Ordering::Acquire);
        let rows = settings_registry::effective_all(store, &Default::default(), |d| {
            d.key.starts_with(PREFIX)
        })
        .await
        .map_err(|_| ())?;
        let value = Arc::new(Policy::parse(rows)?);
        *self.value.write().unwrap_or_else(|p| p.into_inner()) = Some(Cached {
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
    use mw_store::ServerKey;
    use serde_json::json;

    async fn defaults() -> Policy {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let rows = settings_registry::effective_all(&store, &Default::default(), |d| {
            d.key.starts_with(PREFIX)
        })
        .await
        .unwrap();
        Policy::parse(rows).unwrap()
    }

    #[tokio::test]
    async fn defaults_are_on_and_generous() {
        let policy = defaults().await;
        assert!(policy.enabled);
        assert_eq!(policy.limits.grace, 10);
        assert_eq!(policy.limits.window_secs, 600);
        assert_eq!(policy.block_status, 429);
        assert_eq!(policy.audit_ip, AuditIp::Masked);
        assert!(policy.probe_paths.len() >= 15);
        assert!(policy.is_probe(&segments(mw_admin::config::DEFAULT_ADMIN_PATH)));
        for signal in Signal::ALL {
            assert!(policy.limits.counts(signal), "{signal:?}");
        }
        assert_eq!(policy.summary().as_object().unwrap().len(), KEYS.len());
    }

    #[tokio::test]
    async fn probe_entries_match_whole_segments() {
        let policy = defaults().await;
        let hit = |p: &str| policy.is_probe(&segments(&p.to_ascii_lowercase()));
        assert!(hit("/wp-admin"));
        assert!(hit("/WP-Admin/install.php"));
        assert!(hit("/blog/wp-login.php"));
        assert!(hit("/manager/html"));
        assert!(hit("/.git/config"));
        assert!(!hit("/wp-administration"));
        assert!(!hit("/manager"));
        assert!(!hit("/"));
        assert!(!hit("/.envelope"));
    }

    #[test]
    fn keys_are_in_the_registry() {
        for key in KEYS {
            let def = settings_registry::lookup(&format!("{PREFIX}{key}"))
                .unwrap_or_else(|| panic!("{key} missing from the registry"));
            assert_eq!(def.dedicated_route, Some(super::super::ROUTE), "{key}");
        }
    }

    #[test]
    fn list_values_are_validated_before_storage() {
        for (key, value, valid) in [
            ("allowlist", json!(["192.0.2.0/24", "2001:db8::/48"]), true),
            ("allowlist", json!(["office"]), false),
            (
                "probePaths",
                json!(["/wp-admin", "/manager/html", "/.env"]),
                true,
            ),
            ("probePaths", json!(["wp-admin"]), false),
            ("probePaths", json!(["/"]), false),
            ("probePaths", json!(["/a b"]), false),
            ("probePaths", json!(["/a//b"]), false),
            (
                "signals",
                json!(["probe-path", "admin-signin-failed"]),
                true,
            ),
            ("signals", json!(["http-404"]), false),
            ("blockStatus", json!("403"), true),
            ("blockStatus", json!("404"), false),
            ("graceAllowance", json!(0), true),
            ("graceAllowance", json!(-1), false),
            ("blockAfter", json!(0), false),
        ] {
            let def = settings_registry::lookup(&format!("{PREFIX}{key}")).unwrap();
            assert_eq!(
                settings_registry::validate(def, &value, "/value").is_ok(),
                valid,
                "{key} {value}"
            );
        }
    }
}
