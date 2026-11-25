//! Time bounds for upstream mail connections (t63 item 0).
//!
//! An upstream that accepts the TCP connection and then sends nothing, stops in
//! the middle of an answer, or trickles bytes used to hold the caller for as
//! long as the socket stayed open. Every IMAP and POP3 exchange is now bounded:
//!
//! | phase | bound | variable (range) | default |
//! |---|---|---|---|
//! | TCP connect (each dial) | `connect` | `MW_UPSTREAM_CONNECT_TIMEOUT_SECS` (1–120) | 15 s |
//! | TLS handshake (implicit TLS, STARTTLS/STLS) and the server greeting | `greeting` | `MW_UPSTREAM_GREETING_TIMEOUT_SECS` (1–120) | 15 s |
//! | one response or line to a command | `read` + bytes received / `min_rate` | `MW_UPSTREAM_READ_TIMEOUT_SECS` (5–600) | 60 s, 16 KiB/s |
//!
//! The response bound grows with the bytes received, so a large FETCH that
//! arrives at 16 KiB/s or faster never times out, while a silent server is cut
//! off after `read` and one that trickles one byte at a time after about
//! `read`. IMAP IDLE waits are not bounded here (the caller races them
//! against its own keepalive timer).
//!
//! Invalid values are reported once on stderr and the default is used.
use std::future::Future;
use std::io;
use std::sync::OnceLock;
use std::time::Duration;

/// The bounds in force (see the module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpstreamTimeouts {
    pub connect: Duration,
    pub greeting: Duration,
    pub read: Duration,
    /// Bytes per second a long answer must keep up to extend its bound.
    pub min_rate: u64,
}

impl Default for UpstreamTimeouts {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(15),
            greeting: Duration::from_secs(15),
            read: Duration::from_secs(60),
            min_rate: 16 * 1024,
        }
    }
}

pub const CONNECT_VAR: &str = "MW_UPSTREAM_CONNECT_TIMEOUT_SECS";
pub const GREETING_VAR: &str = "MW_UPSTREAM_GREETING_TIMEOUT_SECS";
pub const READ_VAR: &str = "MW_UPSTREAM_READ_TIMEOUT_SECS";

/// Parse the three variables; an unset or empty one keeps its default.
pub fn parse_upstream_timeouts(var: impl Fn(&str) -> Option<String>) -> Result<UpstreamTimeouts, String> {
    let mut out = UpstreamTimeouts::default();
    let field = |name: &str, min: u64, max: u64, slot: &mut Duration| -> Result<(), String> {
        let Some(raw) = var(name).filter(|raw| !raw.trim().is_empty()) else { return Ok(()) };
        let secs = raw
            .trim()
            .parse::<u64>()
            .ok()
            .filter(|secs| (min..=max).contains(secs))
            .ok_or_else(|| format!("{name} must be a whole number of seconds from {min} to {max}, got {raw:?}"))?;
        *slot = Duration::from_secs(secs);
        Ok(())
    };
    field(CONNECT_VAR, 1, 120, &mut out.connect)?;
    field(GREETING_VAR, 1, 120, &mut out.greeting)?;
    field(READ_VAR, 5, 600, &mut out.read)?;
    Ok(out)
}

tokio::task_local! {
    static OVERRIDE: UpstreamTimeouts;
}

/// The bounds in force for this task: a test override, else the process
/// configuration (read once).
pub fn upstream_timeouts() -> UpstreamTimeouts {
    if let Ok(over) = OVERRIDE.try_with(|t| *t) {
        return over;
    }
    static CONFIGURED: OnceLock<UpstreamTimeouts> = OnceLock::new();
    *CONFIGURED.get_or_init(|| match parse_upstream_timeouts(|name| std::env::var(name).ok()) {
        Ok(timeouts) => timeouts,
        Err(error) => {
            eprintln!("[mw-net] {error}; using the default upstream timeouts");
            UpstreamTimeouts::default()
        }
    })
}

/// Run `future` (and everything it awaits in this task) with other bounds.
/// For tests; production reads the environment.
pub async fn with_upstream_timeouts<F: Future>(timeouts: UpstreamTimeouts, future: F) -> F::Output {
    OVERRIDE.scope(timeouts, future).await
}

pub(crate) fn timed_out(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, format!("upstream {what} timed out"))
}

/// `future` within `limit`, else a `TimedOut` I/O error naming `what`.
pub async fn within<T, F>(limit: Duration, what: &str, future: F) -> io::Result<T>
where
    F: Future<Output = io::Result<T>>,
{
    tokio::time::timeout(limit, future).await.map_err(|_| timed_out(what))?
}

/// The bound of one response: starts at `base` and grows by one second per
/// `min_rate` bytes received.
#[derive(Debug)]
pub struct ReadDeadline {
    started: tokio::time::Instant,
    base: Duration,
    min_rate: u64,
    received: u64,
}

impl ReadDeadline {
    /// A response to a command (`read` bound).
    pub fn response() -> Self {
        let t = upstream_timeouts();
        Self::new(t.read, t.min_rate)
    }
    /// The server greeting (`greeting` bound).
    pub fn greeting() -> Self {
        let t = upstream_timeouts();
        Self::new(t.greeting, t.min_rate)
    }
    pub fn new(base: Duration, min_rate: u64) -> Self {
        Self { started: tokio::time::Instant::now(), base, min_rate: min_rate.max(1), received: 0 }
    }
    fn deadline(&self) -> tokio::time::Instant {
        let earned = Duration::from_secs_f64(self.received as f64 / self.min_rate as f64);
        self.started + self.base + earned
    }
    /// One read (returning the byte count) within the current bound.
    pub async fn read<F>(&mut self, read: F) -> io::Result<usize>
    where
        F: Future<Output = io::Result<usize>>,
    {
        let n = tokio::time::timeout_at(self.deadline(), read)
            .await
            .map_err(|_| timed_out("response"))??;
        self.received += n as u64;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variables_parse_within_their_ranges() {
        let none = |_: &str| None;
        assert_eq!(parse_upstream_timeouts(none).unwrap(), UpstreamTimeouts::default());
        let set = |name: &str| match name {
            CONNECT_VAR => Some("5".to_owned()),
            GREETING_VAR => Some(" 20 ".to_owned()),
            READ_VAR => Some("300".to_owned()),
            _ => None,
        };
        let parsed = parse_upstream_timeouts(set).unwrap();
        assert_eq!((parsed.connect, parsed.greeting, parsed.read),
            (Duration::from_secs(5), Duration::from_secs(20), Duration::from_secs(300)));
        for (name, raw) in [(CONNECT_VAR, "0"), (CONNECT_VAR, "121"), (GREETING_VAR, "x"), (READ_VAR, "4"), (READ_VAR, "601")] {
            let err = parse_upstream_timeouts(|n: &str| (n == name).then(|| raw.to_owned())).unwrap_err();
            assert!(err.contains(name), "{err}");
        }
    }

    /// A silent peer is cut off at the base bound; a trickling one shortly
    /// after; a steady stream above the rate is not.
    #[tokio::test]
    async fn a_response_bound_grows_only_with_progress() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let base = Duration::from_millis(300);
        let (mut client, server) = tokio::io::duplex(1 << 16);
        let mut silent = ReadDeadline::new(base, 16 * 1024);
        let mut buf = [0u8; 1024];
        let started = std::time::Instant::now();
        let err = silent.read(client.read(&mut buf)).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() >= base && started.elapsed() < base * 4, "{:?}", started.elapsed());
        drop(server);

        // One byte every 100 ms: earns 1/16384 s per byte, so it ends near 300 ms.
        let (mut client, mut server) = tokio::io::duplex(1 << 16);
        let trickle = tokio::spawn(async move {
            loop {
                if server.write_all(b"x").await.is_err() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        let mut slow = ReadDeadline::new(base, 16 * 1024);
        let started = std::time::Instant::now();
        let ended = loop {
            if let Err(err) = slow.read(client.read(&mut buf)).await {
                break err;
            }
        };
        assert_eq!(ended.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < base * 4, "{:?}", started.elapsed());
        trickle.abort();

        // 64 KiB every 50 ms for 2 s (> 16 KiB/s): never cut off.
        let (mut client, mut server) = tokio::io::duplex(1 << 20);
        tokio::spawn(async move {
            for _ in 0..40 {
                if server.write_all(&[0u8; 64 * 1024]).await.is_err() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });
        let mut steady = ReadDeadline::new(base, 16 * 1024);
        let mut big = vec![0u8; 64 * 1024];
        let mut total = 0;
        while total < 40 * 64 * 1024 {
            total += steady.read(client.read(&mut big)).await.unwrap();
        }
    }
}
