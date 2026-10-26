//! Inbound source admission. This does not grant proxy trust or outbound access.
mod config;
mod dns;
mod routes;
mod runtime;
pub(crate) use routes::register;
pub(crate) use runtime::Runtime;

use axum::{
    Json,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

fn surface(path: &str) -> Option<bool> {
    let admin = mw_admin::config::admin_path();
    if path == admin
        || path
            .strip_prefix(&admin)
            .is_some_and(|tail| tail.starts_with('/'))
    {
        return Some(true);
    }
    // Includes password/SSO APIs, JMAP, DAV and remote content downloads.
    [
        "/api", "/jmap", "/dav", "/rc", "/sso", "/oidc", "/oauth", "/mcp",
    ]
    .iter()
    .any(|base| {
        path == *base
            || path
                .strip_prefix(base)
                .is_some_and(|tail| tail.starts_with('/'))
    })
    .then_some(false)
}
pub(crate) async fn guard(
    State(state): State<crate::AppState>,
    request: Request,
    next: Next,
) -> Response {
    let Some(admin) = surface(request.uri().path()) else {
        return next.run(request).await;
    };
    let ip = crate::scope_mw::proxy::client_ip(request.headers(), request.extensions());
    match state.inbound_allowlist.admit(&state.store, admin, ip).await {
        Ok(true) => next.run(request).await,
        result => (
            if result.is_err() {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::FORBIDDEN
            },
            [("cache-control", "no-store")],
            Json(serde_json::json!({"code":"sourceUnavailable","error":"Request unavailable."})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_boundaries_are_not_prefix_guesses() {
        assert_eq!(surface("/admin/login"), Some(true));
        assert_eq!(surface("/jmap/api"), Some(false));
        assert_eq!(surface("/api/login"), Some(false));
        assert_eq!(surface("/apiary"), None);
        assert_eq!(surface("/healthz"), None);
    }
    #[test]
    fn invalid_names_and_bounds_fail_closed() {
        for name in [
            "localhost",
            "*.example.com",
            "https://example.com",
            "EXAMPLE.com",
            "office.example.",
            "192.0.2.1",
            "a..example",
            "a.example:53",
        ] {
            let mut c = config::Config::default();
            c.admin.hostnames = vec![name.into()];
            assert!(c.validate().is_err(), "{name}");
        }
        let mut c = config::Config::default();
        c.admin.enabled = true;
        assert!(c.validate().is_err());
        c.admin.cidrs.push("192.0.2.0/24".into());
        c.admin.hostnames.push("office.example".into());
        assert!(c.validate().is_ok());
        c.dns.concurrency = 5;
        assert!(c.validate().is_err());
        assert!(config::Config::parse(Some("broken")).is_err());
    }
}
