//! Startup bounds for the built-in HTTP server. An explicit value always wins and
//! must be inside its range or the server refuses to start. When a connection,
//! request, HTTP/2 stream or queue limit is not set, [`HttpLimits::from_env`]
//! derives it from the resources the process has (`super::resources`), never below
//! the fixed default.
use super::resources::{self, Auto, Resources};
use std::{io, time::Duration};

/// Where an effective value came from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Source {
    /// The fixed built-in value (no resource reading was taken).
    #[default]
    Default,
    /// Set by the operator.
    Explicit,
    /// Derived from the resources at start-up.
    Auto,
}
impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Explicit => "explicit",
            Self::Auto => "auto",
        }
    }
}

/// How the effective limits were reached, for the start-up line and diagnostics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Derivation {
    pub resources: Resources,
    pub db_pool: Option<u32>,
    pub handshakes: Option<u64>,
    pub fd_reserve: Option<u64>,
    /// The open-file limit is below what the connection floor needs.
    pub fd_short: bool,
    pub connections: Source,
    pub requests: Source,
    pub http2_streams: Source,
    pub queue_max: Source,
}

#[derive(Clone, Copy, Debug)]
pub struct HttpLimits {
    pub connections: usize,
    pub requests: usize,
    pub header_bytes: usize,
    pub headers: usize,
    pub header_timeout: Duration,
    pub body_timeout: Duration,
    pub http2_streams: u32,
    pub shutdown_timeout: Duration,
    /// How long a request at the concurrency limit waits for a slot; zero answers
    /// 503 at once.
    pub queue_wait: Duration,
    /// Requests that may wait at once; beyond it, 503 at once.
    pub queue_max: usize,
    pub derivation: Derivation,
}

pub const QUEUE_WAIT_DEFAULT_MS: u64 = 200;
pub const QUEUE_WAIT_MAX_MS: u64 = 2000;
pub const QUEUE_MAX_CEILING: usize = 65536;

impl Default for HttpLimits {
    fn default() -> Self {
        Self {
            connections: resources::CONNECTIONS_FLOOR,
            requests: resources::REQUESTS_FLOOR,
            header_bytes: 32768,
            headers: 100,
            header_timeout: Duration::from_secs(10),
            body_timeout: Duration::from_secs(60),
            http2_streams: resources::STREAMS_FLOOR,
            shutdown_timeout: Duration::from_secs(30),
            queue_wait: Duration::from_millis(QUEUE_WAIT_DEFAULT_MS),
            queue_max: resources::REQUESTS_FLOOR,
            derivation: Derivation::default(),
        }
    }
}

fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn number(raw: Option<String>, key: &str, min: u64, max: u64) -> io::Result<Option<u64>> {
    let Some(raw) = raw else { return Ok(None) };
    raw.trim()
        .parse::<u64>()
        .ok()
        .filter(|n| (min..=max).contains(n))
        .map(Some)
        .ok_or_else(|| invalid(format!("{key} must be an integer between {min} and {max}")))
}

impl HttpLimits {
    /// Read the environment and derive unset limits from the running system. When
    /// `MW_HTTP_MAX_CONNECTIONS` is unset this raises the soft open-file limit
    /// first (see [`Resources::probe`]).
    pub fn from_env() -> io::Result<Self> {
        let raise = std::env::var_os("MW_HTTP_MAX_CONNECTIONS").is_none();
        Self::read(|key| std::env::var(key).ok(), Some(Resources::probe(raise)))
    }

    /// The configured values with fixed defaults for unset ones (no system reading).
    pub fn from_env_without_probe() -> io::Result<Self> {
        Self::read(|key| std::env::var(key).ok(), None)
    }

    /// Validate the explicit values only (no system reading, no side effect), for
    /// start-up validation before the database opens.
    pub fn validate_env(env: impl Fn(&str) -> Option<String>) -> Result<(), String> {
        Self::read(env, None)
            .and_then(|limits| limits.validate().map(|()| limits))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// `resources = None` uses the fixed defaults for unset values.
    pub(crate) fn read(
        env: impl Fn(&str) -> Option<String>,
        resources: Option<Resources>,
    ) -> io::Result<Self> {
        let get = |key: &str| env(key);
        let connections = number(
            get("MW_HTTP_MAX_CONNECTIONS"),
            "MW_HTTP_MAX_CONNECTIONS",
            16,
            resources::CONNECTIONS_CEILING as u64,
        )?;
        let requests = number(
            get("MW_HTTP_MAX_REQUESTS"),
            "MW_HTTP_MAX_REQUESTS",
            1,
            resources::REQUESTS_CEILING as u64,
        )?;
        let header_bytes = number(
            get("MW_HTTP_MAX_HEADER_BYTES"),
            "MW_HTTP_MAX_HEADER_BYTES",
            8192,
            65536,
        )?;
        let headers = number(get("MW_HTTP_MAX_HEADERS"), "MW_HTTP_MAX_HEADERS", 16, 256)?;
        let header_timeout = number(
            get("MW_HTTP_HEADER_TIMEOUT_MS"),
            "MW_HTTP_HEADER_TIMEOUT_MS",
            1000,
            60000,
        )?;
        let body_timeout = number(
            get("MW_HTTP_BODY_TIMEOUT_MS"),
            "MW_HTTP_BODY_TIMEOUT_MS",
            1000,
            300000,
        )?;
        let http2_streams = number(get("MW_HTTP2_MAX_STREAMS"), "MW_HTTP2_MAX_STREAMS", 1, 128)?;
        let shutdown_timeout = number(
            get("MW_HTTP_SHUTDOWN_TIMEOUT_MS"),
            "MW_HTTP_SHUTDOWN_TIMEOUT_MS",
            1000,
            120000,
        )?;
        let queue_wait = number(
            get("MW_HTTP_REQUEST_QUEUE_MS"),
            "MW_HTTP_REQUEST_QUEUE_MS",
            0,
            QUEUE_WAIT_MAX_MS,
        )?;
        let queue_max = number(
            get("MW_HTTP_REQUEST_QUEUE_MAX"),
            "MW_HTTP_REQUEST_QUEUE_MAX",
            1,
            QUEUE_MAX_CEILING as u64,
        )?;

        let base = Self::default();
        let mut derivation = Derivation::default();
        let auto: Option<Auto> = resources.map(|resources| {
            let cpus = resources.cpus.unwrap_or(1);
            let db_pool = resources::db_pool_estimate(
                get("MW_DB_PATH").as_deref(),
                get("MW_DB_POOL").as_deref(),
                cpus,
            );
            let handshakes = ["MW_TLS_MAX_HANDSHAKES", "MW_PROXY_MAX_HANDSHAKES"]
                .iter()
                .map(|key| {
                    get(key)
                        .and_then(|raw| raw.trim().parse::<u64>().ok())
                        .unwrap_or(128)
                })
                .sum();
            derivation.resources = resources;
            derivation.db_pool = Some(db_pool);
            derivation.handshakes = Some(handshakes);
            resources::derive(&resources, db_pool, handshakes)
        });
        let pick = |explicit: Option<u64>, auto: Option<u64>, fixed: u64| match (explicit, auto) {
            (Some(n), _) => (n, Source::Explicit),
            (None, Some(n)) => (n, Source::Auto),
            (None, None) => (fixed, Source::Default),
        };
        let (connections, connections_source) = pick(
            connections,
            auto.map(|a| a.connections as u64),
            base.connections as u64,
        );
        let (requests, requests_source) = pick(
            requests,
            auto.map(|a| a.requests as u64),
            base.requests as u64,
        );
        // Unset streams and queue follow the effective request limit, so an explicit
        // MW_HTTP_MAX_REQUESTS also sizes them.
        let follows = |source: Source| (source != Source::Default).then_some(());
        let (http2_streams, streams_source) = pick(
            http2_streams,
            follows(requests_source).map(|()| u64::from(resources::streams_for(requests as usize))),
            u64::from(base.http2_streams),
        );
        let (queue_max, queue_source) = pick(
            queue_max,
            follows(requests_source).map(|()| requests.min(QUEUE_MAX_CEILING as u64)),
            base.queue_max as u64,
        );
        derivation.fd_reserve = auto.and_then(|a| a.fd_reserve);
        derivation.fd_short =
            auto.is_some_and(|a| a.fd_short) && connections_source == Source::Auto;
        derivation.connections = connections_source;
        derivation.requests = requests_source;
        derivation.http2_streams = streams_source;
        derivation.queue_max = queue_source;
        Ok(Self {
            connections: connections as usize,
            requests: requests as usize,
            header_bytes: header_bytes.unwrap_or(base.header_bytes as u64) as usize,
            headers: headers.unwrap_or(base.headers as u64) as usize,
            header_timeout: header_timeout.map_or(base.header_timeout, Duration::from_millis),
            body_timeout: body_timeout.map_or(base.body_timeout, Duration::from_millis),
            http2_streams: http2_streams as u32,
            shutdown_timeout: shutdown_timeout.map_or(base.shutdown_timeout, Duration::from_millis),
            queue_wait: queue_wait.map_or(base.queue_wait, Duration::from_millis),
            queue_max: queue_max as usize,
            derivation,
        })
    }

    pub fn validate(self) -> io::Result<()> {
        if !(16..=resources::CONNECTIONS_CEILING).contains(&self.connections)
            || !(1..=resources::REQUESTS_CEILING).contains(&self.requests)
            || !(8192..=65536).contains(&self.header_bytes)
            || !(16..=256).contains(&self.headers)
            || !(1..=128).contains(&self.http2_streams)
            || !(Duration::from_secs(1)..=Duration::from_secs(60)).contains(&self.header_timeout)
            || !(Duration::from_secs(1)..=Duration::from_secs(300)).contains(&self.body_timeout)
            || !(Duration::from_secs(1)..=Duration::from_secs(120)).contains(&self.shutdown_timeout)
            || self.queue_wait > Duration::from_millis(QUEUE_WAIT_MAX_MS)
            || !(1..=QUEUE_MAX_CEILING).contains(&self.queue_max)
        {
            return Err(invalid(
                "HTTP startup limits outside supported bounds".into(),
            ));
        }
        Ok(())
    }

    /// The one start-up log line: each effective limit and where it came from.
    pub fn summary(&self) -> String {
        let d = &self.derivation;
        let reading =
            |value: Option<u64>| value.map_or_else(|| "unknown".to_owned(), |n| n.to_string());
        let mut line = format!(
            "HTTP limits: connections {} ({}), requests {} ({}), HTTP/2 streams {} ({}), \
             queue {} for up to {} ms ({})",
            self.connections,
            d.connections.as_str(),
            self.requests,
            d.requests.as_str(),
            self.http2_streams,
            d.http2_streams.as_str(),
            self.queue_max,
            self.queue_wait.as_millis(),
            d.queue_max.as_str(),
        );
        if d.db_pool.is_some() {
            line.push_str(&format!(
                "; from cpus {}, memory {} bytes, open files {}, database pool {}, handshake slots {}, fd reserve {}",
                reading(d.resources.cpus.map(u64::from)),
                reading(d.resources.memory_bytes),
                reading(d.resources.open_files),
                reading(d.db_pool.map(u64::from)),
                reading(d.handshakes),
                reading(d.fd_reserve),
            ));
        }
        line
    }
}

#[cfg(test)]
#[path = "http_limits_tests.rs"]
mod tests;
