//! Per-fetch time and redirect bounds a caller opts into (t56).
//!
//! Before t56 the only bound on a fetch was reqwest's `.timeout(FETCH_TIMEOUT)`
//! **per hop**: one 10 s clock over connect, TLS, headers and body together, reset
//! on every redirect, so a five-hop fetch could hold a slot for ~50 s plus DNS,
//! and a body trickling one byte every few seconds was only stopped by that total.
//! [`FetchTimeouts`] splits the clock into the phases that fail differently:
//!
//! | phase | what it bounds | measured from |
//! |---|---|---|
//! | `connect` | TCP connect **and** the TLS handshake (reqwest's connect timeout wraps the whole connector) | the start of the hop |
//! | `headers` | the status line and headers | the start of the hop (so it includes `connect`) |
//! | `idle` | the gap between two body chunks | the previous chunk (or the headers) |
//! | `total` | the whole fetch: DNS, every hop, every body | the first hop |
//!
//! `max_redirects` replaces [`crate::MAX_REDIRECTS`] for the fetch; every hop is
//! still resolved once, pinned and gated by the caller's address policy, exactly
//! as without timeouts. Nothing here widens what a fetch may reach.
//!
//! The bounds are opt-in through [`crate::plan::Request::timeouts`]: a class that
//! does not set them keeps the legacy per-hop behaviour unchanged.

use std::sync::OnceLock;
use std::time::Duration;

/// Time and redirect bounds for one fetch. See the module docs for what each
/// field measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FetchTimeouts {
    /// TCP connect plus TLS handshake, per hop.
    pub connect: Duration,
    /// Start of a hop to its response headers, per hop.
    pub headers: Duration,
    /// Longest gap between two body chunks.
    pub idle: Duration,
    /// The whole fetch, all hops and DNS included.
    pub total: Duration,
    /// Redirects followed; one more is refused as [`crate::Refusal::Upstream`].
    pub max_redirects: usize,
}

impl FetchTimeouts {
    /// The remote-image defaults (t56): connect 5 s, headers 10 s, idle 10 s,
    /// total 20 s, three redirects. The reasoning is in
    /// `docs/admin/settings.md#privacy.fetchTotalTimeoutMs`.
    pub const REMOTE_IMAGE: FetchTimeouts = FetchTimeouts {
        connect: Duration::from_secs(5),
        headers: Duration::from_secs(10),
        idle: Duration::from_secs(10),
        total: Duration::from_secs(20),
        max_redirects: 3,
    };
}

impl Default for FetchTimeouts {
    fn default() -> Self {
        Self::REMOTE_IMAGE
    }
}

/// Which bound stopped a fetch. Carries no URL, host or account, so a caller may
/// log it as it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutPhase {
    /// TCP connect or TLS handshake.
    Connect,
    /// No response headers in time.
    Headers,
    /// The body stalled between chunks.
    Idle,
    /// The whole-fetch deadline.
    Total,
}

impl TimeoutPhase {
    /// A stable lower-case name for logs and metrics.
    pub fn as_str(self) -> &'static str {
        match self {
            TimeoutPhase::Connect => "connect",
            TimeoutPhase::Headers => "headers",
            TimeoutPhase::Idle => "idle",
            TimeoutPhase::Total => "total",
        }
    }
}

/// Where a timed fetch reports which bound fired. Set at most once, by the
/// first bound that fires; empty when the fetch did not time out (or ran without
/// [`FetchTimeouts`]).
#[derive(Debug, Default)]
pub struct TimeoutReport(OnceLock<TimeoutPhase>);

impl TimeoutReport {
    /// An empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// The bound that fired, if one did.
    pub fn phase(&self) -> Option<TimeoutPhase> {
        self.0.get().copied()
    }

    pub(crate) fn record(&self, phase: TimeoutPhase) {
        let _ = self.0.set(phase);
    }
}

/// What a timed hop needs: the bounds and where to report.
#[derive(Clone, Copy)]
pub(crate) struct Timed<'a> {
    pub limits: FetchTimeouts,
    pub report: Option<&'a TimeoutReport>,
}

impl Timed<'_> {
    pub(crate) fn fired(&self, phase: TimeoutPhase) -> crate::Refusal {
        if let Some(report) = self.report {
            report.record(phase);
        }
        crate::Refusal::Timeout
    }
}
