//! HTTP protocol anomaly guard.
//!
//! An early middleware that inspects what the application can still see of a
//! request (method, version, target, headers) before routing, authentication or
//! body parsing, and classes what it finds. Each class is `off`, `detect` (log,
//! count, feed the adaptive protection signal, allow) or `enforce` (reject).
//!
//! What it cannot do is listed in `docs/admin/protocol-guard.md`: hyper has
//! already refused or normalised the byte-level cases (obs-fold, bare CR, NUL
//! and control bytes, space before the colon, conflicting `Content-Length`, bad
//! versions, oversize heads) before this runs, and a front proxy that rewrites
//! requests hides what the client really sent from every check here.
//!
//! A finding is a class and a static code. No header value, cookie or query
//! text is logged, counted or reflected.
mod check;
pub(crate) mod config;
mod stats;
#[cfg(test)]
mod tests;

use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderValue, Version, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
pub(crate) use config::Cache;
use config::{Class, Config, Mode};
use std::{net::IpAddr, sync::Arc};

/// Coarse path class for logs: never the path itself.
fn path_class(path: &str) -> &'static str {
    let admin = mw_admin::config::admin_path();
    if path.strip_prefix(admin.as_str()).is_some_and(|r| r.is_empty() || r.starts_with('/')) {
        "admin"
    } else if path.starts_with("/jmap") || path.starts_with("/.well-known/jmap") {
        "jmap"
    } else if path.starts_with("/dav") {
        "dav"
    } else if path.starts_with("/api") {
        "api"
    } else {
        "other"
    }
}

/// The settings snapshot the Adaptive protection page shows.
pub(crate) async fn summary(state: &AppState) -> serde_json::Value {
    let config = state.request_protection.protocol.get(&state.store).await;
    stats::snapshot(&config)
}

pub(crate) async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    // In-process synthetic requests carry no peer; production always installs
    // `ConnectInfo`. Like `source_bans::guard`, nothing is assumed for them.
    if crate::scope_mw::proxy::peer_ip(request.extensions()).is_none() {
        return next.run(request).await;
    }
    let config = state.request_protection.protocol.get(&state.store).await;
    run(config, Some(&state), request, next).await
}

async fn run(
    config: Arc<Config>,
    state: Option<&AppState>,
    request: Request,
    next: Next,
) -> Response {
    use crate::scope_mw::proxy;
    let Some(peer) = proxy::peer_ip(request.extensions()) else {
        return next.run(request).await;
    };
    let proxies = proxy::ProxyConfig::from_env();
    let view = check::View {
        method: request.method(),
        uri: request.uri(),
        version: request.version(),
        headers: request.headers(),
        peer_trusted: proxies.is_trusted(peer),
        proxies_configured: !proxies.trusted.is_empty(),
        peer_loopback: peer.is_loopback(),
    };
    let mut findings = check::inspect(&config, &view);
    if findings.is_empty() {
        return next.run(request).await;
    }
    // Probes must stay reachable: health and readiness are never refused.
    let probe = matches!(request.uri().path(), "/healthz" | "/readyz");
    let action = |class: Class| match config.mode(class) {
        Mode::Enforce if probe => Mode::Detect,
        mode => mode,
    };
    findings.dedup_by_key(|f| (f.class, f.code));
    let source: Option<IpAddr> = proxy::client_ip(request.headers(), request.extensions());
    let class_of_path = path_class(request.uri().path());
    let mut reject = None;
    for finding in &findings {
        let act = action(finding.class);
        stats::record(finding.class, act);
        tracing::info!(
            target: "protocol_guard",
            class = finding.class.key(),
            code = finding.code,
            action = act.as_str(),
            source = source.map(|ip| ip.to_canonical().to_string()).as_deref().unwrap_or("unknown"),
            path_class = class_of_path,
            "protocol anomaly"
        );
        if act == Mode::Enforce && reject.is_none() {
            reject = Some(*finding);
        }
    }
    let first = findings[0];
    let version = request.version();
    let headers_for_audit = request.headers().clone();
    if let Some(state) = state {
        if stats::audit_due(first.class, source) {
            crate::audit::security_event(
                state,
                crate::audit::SecurityEvent::system("protocol-anomaly")
                    .ip(source)
                    .request(&headers_for_audit)
                    .detail(serde_json::json!({
                        "class": first.class.key(),
                        "code": first.code,
                        "action": action(first.class).as_str(),
                        "pathClass": class_of_path,
                    })),
            )
            .await;
        }
        // Feed the adaptive decision path with a typed signal. A forwarding
        // header from an unconfigured proxy topology is not abuse evidence, so
        // that class only signals once trusted proxies are declared.
        let signals = findings
            .iter()
            .any(|f| f.class != Class::Forwarded || view_proxies(&view));
        if signals
            && !probe
            && let Some(source) = source
            && let Some(limited) =
                crate::request_protection::observe_source(state, source, mw_store::request_protection_model::Signal::Protocol).await
        {
            return limited;
        }
    }
    match reject {
        Some(finding) => rejection(&finding, &config, version),
        None => next.run(request).await,
    }
}
fn view_proxies(view: &check::View<'_>) -> bool {
    view.proxies_configured
}

/// A minimal answer in the shape of the other error pages; it names the class
/// of problem, never the offending value.
fn rejection(finding: &check::Finding, config: &Config, version: Version) -> Response {
    use crate::error_pages::{ErrorKind, json};
    let status = finding.status();
    let mut response = match status {
        axum::http::StatusCode::METHOD_NOT_ALLOWED => json(ErrorKind::MethodNotAllowed),
        _ => json(ErrorKind::BadRequest),
    };
    *response.status_mut() = status;
    if status == axum::http::StatusCode::METHOD_NOT_ALLOWED {
        let allow = config
            .methods
            .iter()
            .take(7)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        if let Ok(value) = HeaderValue::from_str(&allow) {
            response.headers_mut().insert(header::ALLOW, value);
        }
    }
    if finding.class.closes_connection() && version != Version::HTTP_2 {
        response
            .headers_mut()
            .insert(header::CONNECTION, HeaderValue::from_static("close"));
    }
    response.into_response()
}
