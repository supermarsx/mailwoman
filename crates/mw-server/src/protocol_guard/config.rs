//! Classes, modes and the cached, hot-reloadable policy of the protocol guard.
use crate::settings_registry;
use mw_store::Store;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

pub(crate) const PREFIX: &str = "security.protocol.";
/// A saved change reaches the guard within this long, or at once when the
/// settings routes invalidate the cache.
const CACHE_TTL: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode {
    Off,
    Detect,
    Enforce,
}
impl Mode {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "off" => Some(Self::Off),
            "detect" => Some(Self::Detect),
            "enforce" => Some(Self::Enforce),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Detect => "detect",
            Self::Enforce => "enforce",
        }
    }
}

/// One anomaly class; the registry key is `security.protocol.<key>Mode`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum Class {
    Framing,
    Host,
    HostPinning,
    PathControl,
    Target,
    PathLength,
    Method,
    DuplicateHeaders,
    Forwarded,
    HeaderHygiene,
    Body,
}
impl Class {
    pub const ALL: [Class; 11] = [
        Self::Framing,
        Self::Host,
        Self::HostPinning,
        Self::PathControl,
        Self::Target,
        Self::PathLength,
        Self::Method,
        Self::DuplicateHeaders,
        Self::Forwarded,
        Self::HeaderHygiene,
        Self::Body,
    ];
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|c| *c == self).unwrap_or(0)
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::Framing => "framing",
            Self::Host => "host",
            Self::HostPinning => "hostPinning",
            Self::PathControl => "pathControl",
            Self::Target => "target",
            Self::PathLength => "pathLength",
            Self::Method => "method",
            Self::DuplicateHeaders => "duplicateHeaders",
            Self::Forwarded => "forwarded",
            Self::HeaderHygiene => "headerHygiene",
            Self::Body => "body",
        }
    }
    /// Shipped default. Smuggling-class and syntactically invalid input is
    /// enforced; heuristics only detect.
    pub fn default_mode(self) -> Mode {
        match self {
            Self::Framing | Self::Host | Self::PathControl => Mode::Enforce,
            _ => Mode::Detect,
        }
    }
    /// Classes a route-prefix exemption can never switch off: a request that
    /// is ambiguous about where it ends, or hides a NUL/CRLF in its path, is
    /// not something a legitimate client needs.
    pub fn exemptible(self) -> bool {
        !matches!(self, Self::Framing | Self::PathControl)
    }
    /// Close the connection after a reject: framing ambiguity means the bytes
    /// after this request cannot be trusted to start a new one.
    pub fn closes_connection(self) -> bool {
        matches!(self, Self::Framing | Self::Host | Self::PathControl)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Config {
    modes: [Mode; 11],
    pub max_path: usize,
    pub max_query: usize,
    pub max_headers: usize,
    pub max_header_value: usize,
    pub max_cookie: usize,
    pub max_body: u64,
    pub methods: BTreeSet<String>,
    /// Lower-case hosts (no port, no trailing dot) the deployment answers for,
    /// from `MW_PUBLIC_URL` and `security.protocol.allowedHosts`. Empty: not pinned.
    pub hosts: BTreeSet<String>,
    pub exempt: Vec<String>,
}
const DEFAULT_METHODS: &[&str] = &[
    "GET",
    "HEAD",
    "POST",
    "PUT",
    "PATCH",
    "DELETE",
    "OPTIONS",
    // WebDAV / CalDAV / CardDAV verbs. The server mounts no inbound DAV routes
    // (only read-only `GET /dav/...` shares), so these reach a 405 at the router;
    // they stay allowed so a DAV client is not reported as a protocol anomaly.
    "PROPFIND",
    "PROPPATCH",
    "REPORT",
    "MKCOL",
    "MKCALENDAR",
    "COPY",
    "MOVE",
    "LOCK",
    "UNLOCK",
];
impl Default for Config {
    fn default() -> Self {
        let mut modes = [Mode::Off; 11];
        for class in Class::ALL {
            modes[class.index()] = class.default_mode();
        }
        Self {
            modes,
            max_path: 4096,
            max_query: 8192,
            max_headers: 64,
            max_header_value: 8192,
            max_cookie: 16384,
            max_body: 128 * 1024 * 1024,
            methods: DEFAULT_METHODS.iter().map(|m| (*m).to_owned()).collect(),
            hosts: public_host().into_iter().collect(),
            exempt: Vec::new(),
        }
    }
}
fn public_host() -> Option<String> {
    let raw = std::env::var("MW_PUBLIC_URL").ok()?;
    let url = reqwest::Url::parse(raw.trim()).ok()?;
    Some(normalise_host(url.host_str()?))
}
/// Lower-case, without brackets, port or trailing dot.
pub(crate) fn normalise_host(raw: &str) -> String {
    let raw = raw.trim().to_ascii_lowercase();
    let host = if let Some(rest) = raw.strip_prefix('[') {
        rest.split(']').next().unwrap_or("").to_owned()
    } else {
        raw.rsplit_once(':')
            .filter(|(_, port)| port.bytes().all(|b| b.is_ascii_digit()))
            .map_or(raw.clone(), |(host, _)| host.to_owned())
    };
    host.trim_end_matches('.').to_owned()
}
impl Config {
    pub fn mode(&self, class: Class) -> Mode {
        self.modes[class.index()]
    }
    #[cfg(test)]
    pub fn with(mut self, class: Class, mode: Mode) -> Self {
        self.modes[class.index()] = mode;
        self
    }
    /// A route prefix exempts heuristic classes only, on a path-segment boundary.
    /// `/` and prefixes without a leading slash never match: that would be the
    /// global off switch this setting must not become.
    pub fn exempts(&self, path: &str) -> bool {
        self.exempt.iter().any(|prefix| {
            path.strip_prefix(prefix.as_str()).is_some_and(|rest| {
                rest.is_empty() || prefix.ends_with('/') || rest.starts_with('/')
            })
        })
    }
    pub fn parse(rows: Vec<settings_registry::Effective>) -> Self {
        let mut config = Self::default();
        let get = |key: &str| {
            rows.iter()
                .find(|row| row.key.strip_prefix(PREFIX) == Some(key))
                .map(|row| &row.value)
        };
        for class in Class::ALL {
            if let Some(mode) = get(&format!("{}Mode", class.key()))
                .and_then(Value::as_str)
                .and_then(Mode::parse)
            {
                config.modes[class.index()] = mode;
            }
        }
        let number = |key: &str| get(key).and_then(Value::as_u64);
        if let Some(n) = number("maxPathBytes") {
            config.max_path = n as usize;
        }
        if let Some(n) = number("maxQueryBytes") {
            config.max_query = n as usize;
        }
        if let Some(n) = number("maxHeaderCount") {
            config.max_headers = n as usize;
        }
        if let Some(n) = number("maxHeaderValueBytes") {
            config.max_header_value = n as usize;
        }
        if let Some(n) = number("maxCookieBytes") {
            config.max_cookie = n as usize;
        }
        if let Some(n) = number("maxBodyBytes") {
            config.max_body = n;
        }
        let list = |key: &str| -> Option<Vec<String>> {
            Some(
                get(key)?
                    .as_array()?
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect(),
            )
        };
        if let Some(methods) = list("allowedMethods") {
            config.methods = methods.iter().map(|m| m.trim().to_owned()).collect();
        }
        if let Some(hosts) = list("allowedHosts") {
            config
                .hosts
                .extend(hosts.iter().map(|h| normalise_host(h)).filter(|h| !h.is_empty()));
        }
        if let Some(prefixes) = list("exemptPrefixes") {
            config.exempt = prefixes
                .into_iter()
                .map(|p| p.trim().to_owned())
                .filter(|p| p.starts_with('/') && p.len() > 1)
                .collect();
        }
        config
    }
}

struct Cached {
    at: Instant,
    epoch: u64,
    value: Arc<Config>,
}
#[derive(Default)]
pub(crate) struct Cache {
    value: RwLock<Option<Cached>>,
    epoch: std::sync::atomic::AtomicU64,
}
impl Cache {
    pub fn invalidate(&self) {
        self.epoch
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }
    /// The current policy. A store failure yields the shipped defaults, never
    /// "no guard": the enforce-by-default classes keep working without the database.
    pub async fn get(&self, store: &Store) -> Arc<Config> {
        let epoch = self.epoch.load(std::sync::atomic::Ordering::Acquire);
        if let Some(cached) = self
            .value
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .as_ref()
            .filter(|c| c.epoch == epoch && c.at.elapsed() < CACHE_TTL)
        {
            return cached.value.clone();
        }
        let value = match settings_registry::effective_all(store, &Default::default(), |def| {
            def.key.starts_with(PREFIX)
        })
        .await
        {
            Ok(rows) => Arc::new(Config::parse(rows)),
            Err(_) => Arc::new(Config::default()),
        };
        *self
            .value
            .write()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(Cached {
            at: Instant::now(),
            epoch,
            value: value.clone(),
        });
        value
    }
}
