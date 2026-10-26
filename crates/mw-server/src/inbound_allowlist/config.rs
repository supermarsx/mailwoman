use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::SocketAddr};

pub(super) const KEY: &str = "security.inbound-allowlist.v1";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Policy {
    pub enabled: bool,
    pub cidrs: Vec<String>,
    pub hostnames: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Dns {
    /// Empty means the operating system's configured recursive resolvers.
    pub servers: Vec<SocketAddr>,
    pub timeout_ms: u64,
    pub cache_seconds: u64,
    pub max_hosts: usize,
    pub max_addresses: usize,
    pub concurrency: usize,
}
impl Default for Dns {
    fn default() -> Self {
        Self {
            servers: vec![],
            timeout_ms: 2000,
            cache_seconds: 300,
            max_hosts: 64,
            max_addresses: 16,
            concurrency: 4,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub admin: Policy,
    pub webmail: Policy,
    pub dns: Dns,
}
impl Config {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(100..=5000).contains(&self.dns.timeout_ms)
            || !(30..=3600).contains(&self.dns.cache_seconds)
            || !(1..=64).contains(&self.dns.max_hosts)
            || !(1..=32).contains(&self.dns.max_addresses)
            || !(1..=4).contains(&self.dns.concurrency)
            || self.dns.servers.len() > 4
            || self
                .dns
                .servers
                .iter()
                .any(|s| s.port() == 0 || s.ip().is_unspecified() || s.ip().is_multicast())
        {
            return Err("DNS limits or resolver addresses are outside the permitted bounds.");
        }
        let mut hosts = BTreeSet::new();
        for policy in [&self.admin, &self.webmail] {
            if policy.cidrs.len() > 128 || policy.hostnames.len() > self.dns.max_hosts {
                return Err("Too many allowlist entries.");
            }
            if policy.enabled && policy.cidrs.is_empty() {
                return Err("An enabled policy requires at least one literal rescue IP or CIDR.");
            }
            if policy
                .cidrs
                .iter()
                .any(|s| s.len() > 64 || crate::scope_mw::proxy::Cidr::parse(s).is_none())
            {
                return Err("Invalid IP address or CIDR.");
            }
            for host in &policy.hostnames {
                // Absolute, ASCII DNS names only. No search suffixes, URLs, wildcards or IP literals.
                if host.len() > 253
                    || !host.contains('.')
                    || host.ends_with('.')
                    || host.parse::<std::net::IpAddr>().is_ok()
                    || host.split('.').any(|label| {
                        label.is_empty()
                            || label.len() > 63
                            || label.starts_with('-')
                            || label.ends_with('-')
                            || !label
                                .bytes()
                                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
                    })
                {
                    return Err(
                        "Use lowercase ASCII DNS names without a trailing dot, URL, port or wildcard.",
                    );
                }
                hosts.insert(host);
            }
        }
        if hosts.len() > self.dns.max_hosts {
            return Err("Too many distinct DNS names.");
        }
        Ok(())
    }
    pub fn hosts(&self) -> Vec<String> {
        [&self.admin, &self.webmail]
            .into_iter()
            .filter(|p| p.enabled)
            .flat_map(|p| p.hostnames.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    pub fn parse(raw: Option<&str>) -> Result<Self, ()> {
        let config: Self = match raw {
            None => Self::default(),
            Some(raw) if raw.len() <= 65536 => serde_json::from_str(raw).map_err(|_| ())?,
            _ => return Err(()),
        };
        config.validate().map_err(|_| ())?;
        Ok(config)
    }
}
