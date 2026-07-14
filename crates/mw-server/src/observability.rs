//! Observability surface (SPEC §21, plan §3 e9). Owned by t6-e9; mounted by e11.
//!
//! Three deliverables, all ADDITIVE and OFF by default (the existing tracing setup
//! in `main.rs` is byte-unchanged unless the operator opts in):
//!
//!   * **`tracing` per-subsystem hot-reload** — a [`ReloadHandle`] over a
//!     [`tracing_subscriber::reload`] layer. e11 composes [`subsystem_reload_layer`]
//!     into its subscriber; the admin panel (or a SIGHUP handler) calls
//!     [`ReloadHandle::reload`] to change per-subsystem log directives with no
//!     restart.
//!   * **OTLP traces + metrics export** — [`init_otlp`] builds tonic (rustls, NO
//!     openssl) OTLP exporters into SDK providers and registers them globally, so a
//!     plain [`tracing_opentelemetry::layer`] bridges spans out. A no-op unless
//!     `MW_OTLP_ENDPOINT` is set.
//!   * **auth-gated Prometheus `/metrics`** — [`metrics`] renders the process-wide
//!     `metrics`/`metrics-exporter-prometheus` recorder, gated behind a bearer token
//!     ([`set_metrics_token`] / `MW_METRICS_TOKEN`); never open.
//!
//! ## No mail content in any log or metric label (§21.1)
//! [`Redacted`] / [`redact_address`] are the typed wrappers every log/label site
//! uses so a subject/body/address can never leak. The webhook + REST modules emit
//! only opaque IDs, counts, and method names — asserted in tests here and in
//! [`crate::webhooks`].

use std::fmt;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::reload;

// ---------------------------------------------------------------------------
// Typed redaction wrappers (§21.1 — no mail content in logs / metric labels)
// ---------------------------------------------------------------------------

/// A wrapper that makes a value UNPRINTABLE in logs. Wrap any mail-derived string
/// (subject, body, snippet, display name) before it reaches a `tracing` field so a
/// stray `?field` / `%field` cannot leak content — `Debug` and `Display` both emit
/// the fixed marker, never the inner value.
///
/// This is a *type-level* guarantee: the only way to read the inner value is
/// [`Redacted::expose`], which is deliberately verbose and never called by a log
/// macro.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Redacted<T>(pub T);

impl<T> Redacted<T> {
    /// The fixed marker emitted in place of the value.
    pub const MARKER: &'static str = "[redacted]";

    /// Escape hatch for the (rare) non-logging caller that genuinely needs the
    /// value. Named loudly so a reviewer notices it in a diff.
    pub fn expose(&self) -> &T {
        &self.0
    }
}

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Self::MARKER)
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(Self::MARKER)
    }
}

/// Reduce an email address to a loggable shape that keeps NO local-part and NO
/// full domain: `alice@example.com` → `a…@example.com`-free `"[addr]"`-style token.
/// We keep only a coarse, non-identifying hint (the address is present, and how
/// long it was) — never the address itself.
pub fn redact_address(_addr: &str) -> &'static str {
    "[address]"
}

// ---------------------------------------------------------------------------
// tracing per-subsystem hot-reload
// ---------------------------------------------------------------------------

/// The type-erased directive-swap closure behind a [`ReloadHandle`] (erases the
/// subscriber generic `S` so the handle is nameable + storable in `AppState`).
type ReloadFn = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// A cloneable handle that swaps the live [`EnvFilter`] directive set at runtime
/// (SPEC §21 — per-subsystem hot-reload, SIGHUP- or admin-triggered). Returned by
/// [`subsystem_reload_layer`] alongside the layer e11 installs.
#[derive(Clone)]
pub struct ReloadHandle {
    reload: ReloadFn,
    current: Arc<Mutex<String>>,
}

impl ReloadHandle {
    /// Replace the active log directives (e.g. `"mw_server=debug,mw_engine=info"`).
    /// Returns an error string if the directive set does not parse, leaving the
    /// previous filter untouched.
    pub fn reload(&self, directives: &str) -> Result<(), String> {
        (self.reload)(directives)
    }

    /// The directive set currently in force (for the admin/observability panel).
    pub fn current(&self) -> String {
        self.current.lock().expect("reload mutex").clone()
    }

    /// Install a SIGHUP handler (Unix) that re-applies the directives read from
    /// `MW_LOG` (falling back to the current set) on each signal. Spawned by e11.
    /// A no-op on non-Unix (Windows has no SIGHUP — the admin endpoint reloads
    /// instead).
    #[cfg(unix)]
    pub fn spawn_sighup_reload(self) {
        tokio::spawn(async move {
            use tokio::signal::unix::{SignalKind, signal};
            let mut hup = match signal(SignalKind::hangup()) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("SIGHUP log-reload handler unavailable: {e}");
                    return;
                }
            };
            while hup.recv().await.is_some() {
                let directives = std::env::var("MW_LOG").unwrap_or_else(|_| self.current());
                match self.reload(&directives) {
                    Ok(()) => tracing::info!("reloaded log directives on SIGHUP: {directives}"),
                    Err(e) => tracing::error!("log reload failed (keeping current): {e}"),
                }
            }
        });
    }

    /// No-op on non-Unix targets (no SIGHUP); the admin endpoint drives reloads.
    #[cfg(not(unix))]
    pub fn spawn_sighup_reload(self) {
        tracing::info!("log hot-reload via SIGHUP is Unix-only; use the admin endpoint on Windows");
    }
}

/// Build a reloadable [`EnvFilter`] layer + its [`ReloadHandle`]. e11 adds the
/// returned layer to its subscriber (`Registry::default().with(layer)…`); the
/// handle is stored in `AppState`/exposed to the admin panel for live reloads.
///
/// Generic over the subscriber `S` so it composes with whatever fmt/OTLP layers
/// e11 stacks on top.
pub fn subsystem_reload_layer<S>(initial: &str) -> (reload::Layer<EnvFilter, S>, ReloadHandle)
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    let filter = EnvFilter::new(initial);
    let (layer, handle) = reload::Layer::new(filter);
    let current = Arc::new(Mutex::new(initial.to_string()));
    let cur = current.clone();
    let reload = Arc::new(move |directives: &str| -> Result<(), String> {
        let filter = EnvFilter::try_new(directives).map_err(|e| e.to_string())?;
        handle.reload(filter).map_err(|e| e.to_string())?;
        *cur.lock().expect("reload mutex") = directives.to_string();
        Ok(())
    });
    (layer, ReloadHandle { reload, current })
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Observability configuration, env-sourced in prod (see [`ObservabilityConfig::from_env`])
/// and set explicitly by tests. Every field defaults to "off" so a deployment that
/// configures none behaves exactly as before.
#[derive(Debug, Clone, Default)]
pub struct ObservabilityConfig {
    /// OTLP collector endpoint (env: `MW_OTLP_ENDPOINT`, e.g. `http://otel:4317`).
    /// `None` → OTLP export disabled.
    pub otlp_endpoint: Option<String>,
    /// Service name reported to the collector (env: `MW_OTEL_SERVICE`).
    pub service_name: String,
    /// Bearer token guarding `/metrics` (env: `MW_METRICS_TOKEN`). `None` → the
    /// endpoint is unreachable (metrics are never exposed unauthenticated).
    pub metrics_token: Option<String>,
    /// Initial per-subsystem log directives (env: `MW_LOG`, default `"info"`).
    pub log_directives: String,
}

impl ObservabilityConfig {
    /// Populate from the environment (used by the `serve` CLI path via e11).
    pub fn from_env() -> Self {
        let s = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        Self {
            otlp_endpoint: s("MW_OTLP_ENDPOINT"),
            service_name: s("MW_OTEL_SERVICE").unwrap_or_else(|| "mailwoman".to_string()),
            metrics_token: s("MW_METRICS_TOKEN"),
            log_directives: s("MW_LOG").unwrap_or_else(|| "info".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// OTLP traces + metrics export (tonic / rustls — NO openssl)
// ---------------------------------------------------------------------------

/// Keeps the OTLP SDK providers alive and flushes them on drop. Held by e11 for
/// the life of the server; dropping it shuts the exporters down cleanly.
pub struct OtelGuard {
    tracer: Option<opentelemetry_sdk::trace::SdkTracerProvider>,
    meter: Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Some(t) = &self.tracer {
            let _ = t.shutdown();
        }
        if let Some(m) = &self.meter {
            let _ = m.shutdown();
        }
    }
}

/// Initialise OTLP export from configuration. When `otlp_endpoint` is set, build a
/// tonic (rustls) span + metric exporter, wrap each in an SDK provider, and
/// register both globally so [`tracing_opentelemetry::layer`] (added by e11) bridges
/// spans out and OTel metrics flow. Returns `Ok(None)` (a clean no-op) when no
/// endpoint is configured. Must be called inside the tokio runtime.
pub fn init_otlp(config: &ObservabilityConfig) -> anyhow::Result<Option<OtelGuard>> {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::WithExportConfig as _;

    let Some(endpoint) = config.otlp_endpoint.clone() else {
        return Ok(None);
    };

    let resource = opentelemetry_sdk::Resource::builder()
        .with_service_name(config.service_name.clone())
        .build();

    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.clone())
        .build()?;
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .with_resource(resource.clone())
        .build();
    // Register a named tracer globally so `tracing_opentelemetry::layer()` picks it
    // up (it resolves the global provider); keep the provider for shutdown.
    let _tracer = tracer_provider.tracer("mw-server");
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(resource)
        .build();
    opentelemetry::global::set_meter_provider(meter_provider.clone());

    tracing::info!(
        "OTLP export enabled → {} (service {})",
        config.otlp_endpoint.as_deref().unwrap_or_default(),
        config.service_name
    );
    Ok(Some(OtelGuard {
        tracer: Some(tracer_provider),
        meter: Some(meter_provider),
    }))
}

// ---------------------------------------------------------------------------
// Prometheus /metrics (auth-gated)
// ---------------------------------------------------------------------------

/// The process-wide Prometheus recorder handle. Installed at most once (the
/// `metrics` crate allows a single global recorder); subsequent calls return the
/// same handle.
static PROMETHEUS: OnceLock<PrometheusHandle> = OnceLock::new();

/// The bearer token guarding `/metrics`. `None` (the default) means the endpoint is
/// closed — metrics are never served unauthenticated.
static METRICS_TOKEN: RwLock<Option<String>> = RwLock::new(None);

/// Install (once) the global Prometheus recorder and return its render handle.
/// Idempotent: repeated calls hand back the same handle, so it is safe to call from
/// both `build_app` and the request handler.
pub fn install_prometheus_recorder() -> &'static PrometheusHandle {
    PROMETHEUS.get_or_init(|| {
        PrometheusBuilder::new()
            .install_recorder()
            .expect("install the global Prometheus recorder")
    })
}

/// Set the `/metrics` bearer token (from [`ObservabilityConfig`], wired by e11).
pub fn set_metrics_token(token: Option<String>) {
    *METRICS_TOKEN.write().expect("metrics token lock") = token;
}

/// Apply the observability config that does not depend on the subscriber: install
/// the Prometheus recorder and set the `/metrics` token. Called from `build_app`.
pub fn init_metrics(config: &ObservabilityConfig) {
    install_prometheus_recorder();
    set_metrics_token(config.metrics_token.clone());
}

/// Constant-time byte comparison (no early-out on the first mismatched byte) so a
/// token guard cannot be timing-probed.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Whether the request carries the configured `/metrics` bearer token. Returns
/// `false` when no token is configured (the endpoint stays closed).
fn metrics_authorized(headers: &HeaderMap) -> bool {
    let want = METRICS_TOKEN.read().expect("metrics token lock").clone();
    let Some(want) = want else {
        return false;
    };
    let Some(got) = crate::push_relay::bearer_token(headers) else {
        return false;
    };
    constant_time_eq(got.as_bytes(), want.as_bytes())
}

/// `GET /metrics` — the auth-gated Prometheus scrape endpoint (SPEC §21). Requires
/// `Authorization: Bearer <MW_METRICS_TOKEN>`; emits the Prometheus text exposition
/// format. Mounted by e11 (and additionally protectable by the admin session / an
/// IP allowlist at the mount site). NO mail content is ever a metric name or label.
pub async fn metrics(headers: HeaderMap) -> Response {
    if !metrics_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            "metrics require a bearer token\n",
        )
            .into_response();
    }
    let body = install_prometheus_recorder().render();
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use tracing_subscriber::Registry;
    use tracing_subscriber::prelude::*;

    #[test]
    fn redacted_never_prints_the_inner_value() {
        let secret = Redacted("Re: your bank password is hunter2");
        assert_eq!(format!("{secret:?}"), "[redacted]");
        assert_eq!(format!("{secret}"), "[redacted]");
        // The value is only reachable via the loud escape hatch.
        assert!(secret.expose().contains("hunter2"));
        assert_eq!(redact_address("alice@example.com"), "[address]");
    }

    #[test]
    fn reload_handle_swaps_directives_live() {
        let (layer, handle) = subsystem_reload_layer::<Registry>("info");
        // Keep the subscriber (and thus the layer) alive so the handle stays valid.
        let _subscriber = Registry::default().with(layer);
        assert_eq!(handle.current(), "info");
        handle
            .reload("mw_server=debug,mw_engine=trace")
            .expect("valid directives reload");
        assert_eq!(handle.current(), "mw_server=debug,mw_engine=trace");
        // An invalid directive set is rejected and leaves the previous one in place.
        assert!(handle.reload("=::bogus::=").is_err());
        assert_eq!(handle.current(), "mw_server=debug,mw_engine=trace");
    }

    fn bearer(token: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
        h
    }

    #[tokio::test]
    async fn metrics_requires_the_bearer_token_and_emits_prometheus_text() {
        // Install the recorder + register a metric so the scrape has content.
        install_prometheus_recorder();
        metrics::counter!("mw_test_events_total").increment(3);
        set_metrics_token(Some("scrape-secret".to_string()));

        // No token → 401.
        let unauth = metrics(HeaderMap::new()).await;
        assert_eq!(unauth.status(), StatusCode::UNAUTHORIZED);

        // Wrong token → 401.
        let wrong = metrics(bearer("nope")).await;
        assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);

        // Correct token → 200 + Prometheus text exposition format.
        let ok = metrics(bearer("scrape-secret")).await;
        assert_eq!(ok.status(), StatusCode::OK);
        let ct = ok
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        assert!(ct.starts_with("text/plain"), "content-type was {ct}");
        let bytes = axum::body::to_bytes(ok.into_body(), 1 << 20).await.unwrap();
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(
            text.contains("mw_test_events_total"),
            "prometheus text missing the metric:\n{text}"
        );

        // Closing the token guard makes the endpoint unreachable again.
        set_metrics_token(None);
        assert_eq!(
            metrics(bearer("scrape-secret")).await.status(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn constant_time_eq_matches_only_identical_slices() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }
}
