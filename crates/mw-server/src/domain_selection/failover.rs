//! Bounded mail-backend failover (t39 decision D3).
//!
//! Before any credential is sent, each candidate gets a credential-free transport
//! check (DNS, TCP connect, the TLS or STARTTLS handshake and the greeting). The
//! first candidate that passes is the only backend that then receives the
//! credentials. A candidate is skipped only for a DNS, connect, timeout, TLS or
//! certificate failure; any other failure (for example the address policy
//! refusing the answer) stops the walk. Whatever the credentialed connection
//! later returns, including an authentication rejection, is final for that
//! attempt: credentials are never replayed to a second backend.
//!
//! Health is process-local. A backend that failed its check is moved behind the
//! healthy ones for a backoff that doubles per consecutive failure (30 s up to
//! 15 min). When the backoff of the selected backend ends it is tried first
//! again, and one success clears its record, so connections return to it.
use crate::{
    domain_config::Endpoint,
    login_failure::{LoginFailure, LoginFailureReason},
};
use std::{
    collections::HashMap,
    future::Future,
    io,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

/// Backends one connection attempt may check, the selected one included.
pub(crate) const MAX_BACKENDS: usize = 3;
const BASE_BACKOFF: Duration = Duration::from_secs(30);
const MAX_BACKOFF: Duration = Duration::from_secs(15 * 60);
const MAX_TRACKED: usize = 4096;

#[derive(Debug, Clone, Copy)]
struct Health {
    failures: u32,
    down_until: Instant,
}
static HEALTH: OnceLock<Mutex<HashMap<String, Health>>> = OnceLock::new();
fn health() -> &'static Mutex<HashMap<String, Health>> {
    HEALTH.get_or_init(Default::default)
}
fn key(endpoint: &Endpoint) -> String {
    format!(
        "{}:{}",
        endpoint
            .host
            .trim_matches(['[', ']'])
            .trim_end_matches('.')
            .to_ascii_lowercase(),
        endpoint.port
    )
}

pub(crate) fn backoff(failures: u32) -> Duration {
    let shift = failures.saturating_sub(1).min(16);
    BASE_BACKOFF.saturating_mul(1u32 << shift).min(MAX_BACKOFF)
}

fn in_backoff(endpoint: &Endpoint, now: Instant) -> bool {
    health()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&key(endpoint))
        .is_some_and(|h| h.down_until > now)
}
fn mark_down(endpoint: &Endpoint, now: Instant) {
    let mut all = health().lock().unwrap_or_else(|e| e.into_inner());
    all.retain(|_, h| h.down_until + MAX_BACKOFF > now);
    let k = key(endpoint);
    if all.len() >= MAX_TRACKED && !all.contains_key(&k) {
        return;
    }
    let entry = all.entry(k).or_insert(Health {
        failures: 0,
        down_until: now,
    });
    entry.failures = entry.failures.saturating_add(1);
    entry.down_until = now + backoff(entry.failures);
}
/// Returns whether the backend had a failure record (it was down).
fn mark_up(endpoint: &Endpoint) -> bool {
    health()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&key(endpoint))
        .is_some()
}

/// Candidate order: healthy backends in configured order, then those in backoff.
pub(crate) fn order(chain: &[(String, Endpoint)], now: Instant) -> Vec<usize> {
    let (healthy, resting): (Vec<usize>, Vec<usize>) =
        (0..chain.len()).partition(|&i| !in_backoff(&chain[i].1, now));
    healthy
        .into_iter()
        .chain(resting)
        .take(MAX_BACKENDS)
        .collect()
}

/// Failures that happen before any credential is sent and justify trying the
/// next backend.
pub(crate) fn allows_failover(reason: LoginFailureReason) -> bool {
    matches!(
        reason,
        LoginFailureReason::Dns
            | LoginFailureReason::Connect
            | LoginFailureReason::Timeout
            | LoginFailureReason::Tls
            | LoginFailureReason::Certificate
    )
}

#[derive(Debug)]
pub(crate) struct Chosen<T> {
    /// Index into the chain; 0 is the selected backend.
    pub index: usize,
    pub value: T,
}

/// Walk `chain` with the credential-free `check`. `domain` and backend IDs are
/// written to the operations log (Admin > Application logs); hosts are not.
pub(crate) async fn choose<T, F, Fut>(
    domain: &str,
    chain: &[(String, Endpoint)],
    check: F,
) -> Result<Chosen<T>, LoginFailure>
where
    F: Fn(Endpoint) -> Fut,
    Fut: Future<Output = io::Result<T>>,
{
    let mut last = LoginFailure::new(LoginFailureReason::NotConfigured);
    let mut failed: Option<(usize, LoginFailureReason)> = None;
    for index in order(chain, Instant::now()) {
        let (id, endpoint) = &chain[index];
        match check(endpoint.clone()).await {
            Ok(value) => {
                let recovered = mark_up(endpoint);
                if let Some((from, reason)) = failed {
                    tracing::warn!(
                        target: "mailwoman::backend_failover",
                        event = "failover",
                        domain,
                        from = chain[from].0.as_str(),
                        to = id.as_str(),
                        reason = reason.code(),
                        "mail backend unreachable; connected to the next backend without retrying credentials"
                    );
                } else if index != 0 {
                    tracing::warn!(
                        target: "mailwoman::backend_failover",
                        event = "selected-backend-resting",
                        domain,
                        from = chain[0].0.as_str(),
                        to = id.as_str(),
                        "selected mail backend is in backoff after a recent failure; connected to the next backend"
                    );
                } else if recovered {
                    tracing::warn!(
                        target: "mailwoman::backend_failover",
                        event = "returned-to-selected",
                        domain,
                        to = id.as_str(),
                        "selected mail backend is reachable again; new connections use it"
                    );
                }
                return Ok(Chosen { index, value });
            }
            Err(error) => {
                let failure = LoginFailure::from_io(&error);
                if !allows_failover(failure.reason) {
                    return Err(failure);
                }
                mark_down(endpoint, Instant::now());
                if failed.is_none() {
                    failed = Some((index, failure.reason));
                }
                tracing::warn!(
                    target: "mailwoman::backend_failover",
                    event = "check-failed",
                    domain,
                    backend = id.as_str(),
                    reason = failure.reason.code(),
                    "mail backend failed its credential-free connection check"
                );
                last = failure;
            }
        }
    }
    Err(last)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_config::TlsMode;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    fn endpoint(host: &str) -> Endpoint {
        Endpoint {
            host: host.into(),
            port: 993,
            tls: TlsMode::Implicit,
            auth: "password".into(),
            allow_private: false,
            security: None,
            authentication: None,
        }
    }
    fn chain(hosts: &[&str]) -> Vec<(String, Endpoint)> {
        hosts
            .iter()
            .map(|h| (h.split('.').next().unwrap().to_string(), endpoint(h)))
            .collect()
    }
    fn refused() -> io::Error {
        io::Error::new(io::ErrorKind::ConnectionRefused, "refused")
    }

    // Each test uses its own host names; the health map is process-wide.
    #[tokio::test]
    async fn a_dead_selected_backend_fails_over_to_the_next() {
        let chain = chain(&["a1.fo.test", "b1.fo.test", "c1.fo.test"]);
        let checked = Arc::new(Mutex::new(Vec::new()));
        let seen = checked.clone();
        let chosen = choose("example.test", &chain, move |e: Endpoint| {
            seen.lock().unwrap().push(e.host.clone());
            async move {
                if e.host.starts_with("a1") {
                    Err(refused())
                } else {
                    Ok(e.host)
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(chosen.index, 1);
        assert_eq!(chosen.value, "b1.fo.test");
        assert_eq!(*checked.lock().unwrap(), ["a1.fo.test", "b1.fo.test"]);
    }

    #[tokio::test]
    async fn tls_and_certificate_failures_fail_over_but_policy_refusal_does_not() {
        let tls = || {
            io::Error::other(rustls::Error::InvalidCertificate(
                rustls::CertificateError::Expired,
            ))
        };
        let chain1 = chain(&["a2.fo.test", "b2.fo.test"]);
        let chosen = choose("example.test", &chain1, move |e: Endpoint| async move {
            if e.host.starts_with("a2") {
                Err(tls())
            } else {
                Ok(())
            }
        })
        .await
        .unwrap();
        assert_eq!(chosen.index, 1);

        let chain2 = chain(&["a3.fo.test", "b3.fo.test"]);
        let calls = Arc::new(AtomicUsize::new(0));
        let n = calls.clone();
        let error = choose("example.test", &chain2, move |_e: Endpoint| {
            n.fetch_add(1, Ordering::SeqCst);
            async {
                Err::<(), _>(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "address policy",
                ))
            }
        })
        .await
        .unwrap_err();
        assert_eq!(error.reason, LoginFailureReason::Policy);
        assert_eq!(calls.load(Ordering::SeqCst), 1, "no second backend checked");
    }

    #[test]
    fn an_authentication_rejection_never_allows_failover() {
        assert!(!allows_failover(LoginFailureReason::Auth));
        assert!(!allows_failover(LoginFailureReason::Protocol));
        assert!(!allows_failover(LoginFailureReason::Policy));
        assert!(allows_failover(LoginFailureReason::Connect));
        assert!(allows_failover(LoginFailureReason::Certificate));
    }

    #[tokio::test]
    async fn backoff_moves_a_failed_backend_back_and_success_returns_to_it() {
        let chain = chain(&["a4.fo.test", "b4.fo.test"]);
        let now = Instant::now();
        assert_eq!(order(&chain, now), [0, 1]);
        mark_down(&chain[0].1, now);
        assert_eq!(order(&chain, now), [1, 0], "resting backend goes last");
        assert_eq!(
            order(&chain, now + backoff(1) + Duration::from_millis(1)),
            [0, 1],
            "after its backoff the selected backend is first again"
        );
        mark_down(&chain[0].1, now);
        assert_eq!(
            health().lock().unwrap()[&key(&chain[0].1)].failures,
            2,
            "consecutive failures lengthen the backoff"
        );
        assert!(mark_up(&chain[0].1));
        assert_eq!(order(&chain, now), [0, 1], "one success clears the record");
        assert_eq!(backoff(1), Duration::from_secs(30));
        assert_eq!(backoff(2), Duration::from_secs(60));
        assert_eq!(backoff(40), MAX_BACKOFF);
    }

    #[tokio::test]
    async fn connections_return_to_the_selected_backend_once_its_backoff_ends() {
        let chain = chain(&["a6.fo.test", "b6.fo.test"]);
        let down = Arc::new(AtomicUsize::new(1));
        let check = |down: Arc<AtomicUsize>| {
            move |e: Endpoint| {
                let down = down.clone();
                async move {
                    if e.host.starts_with("a6") && down.load(Ordering::SeqCst) == 1 {
                        Err(refused())
                    } else {
                        Ok(())
                    }
                }
            }
        };
        let first = choose("example.test", &chain, check(down.clone()))
            .await
            .unwrap();
        assert_eq!(first.index, 1, "dead selected backend: fallback used");
        down.store(0, Ordering::SeqCst);
        let resting = choose("example.test", &chain, check(down.clone()))
            .await
            .unwrap();
        assert_eq!(
            resting.index, 1,
            "still resting: the fallback is checked first"
        );
        // End the backoff as if its time had passed.
        health()
            .lock()
            .unwrap()
            .get_mut(&key(&chain[0].1))
            .unwrap()
            .down_until = Instant::now();
        let back = choose("example.test", &chain, check(down.clone()))
            .await
            .unwrap();
        assert_eq!(back.index, 0, "sticky return to the selected backend");
        assert!(!health().lock().unwrap().contains_key(&key(&chain[0].1)));
    }

    #[tokio::test]
    async fn attempts_are_bounded_and_all_down_reports_the_last_failure() {
        let mut long = chain(&["a5.fo.test", "b5.fo.test", "c5.fo.test"]);
        long.push(("d5".into(), endpoint("d5.fo.test")));
        let calls = Arc::new(AtomicUsize::new(0));
        let n = calls.clone();
        let error = choose("example.test", &long, move |_e: Endpoint| {
            n.fetch_add(1, Ordering::SeqCst);
            async { Err::<(), _>(refused()) }
        })
        .await
        .unwrap_err();
        assert_eq!(error.reason, LoginFailureReason::Connect);
        assert_eq!(calls.load(Ordering::SeqCst), MAX_BACKENDS);
    }
}
