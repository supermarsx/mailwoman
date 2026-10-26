//! Optional local-database country policy for sign-ins and opted-in session requests.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use mw_engine::security::{CountryLookup, country_lookup};
use mw_store::AuthSurface;
use serde_json::{Value, json};
use std::net::IpAddr;
#[derive(Clone)]
struct Policy {
    scope: String,
    mode: String,
    countries: Vec<String>,
    unknown: String,
    enforce_sessions: bool,
}
fn unavailable() -> AdminApiError {
    AdminApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "signinGeographyUnavailable",
        "Sign-in country policy is temporarily unavailable",
    )
}
impl Policy {
    async fn load(state: &AppState) -> Result<Self, AdminApiError> {
        async fn value(state: &AppState, key: &str) -> Result<Value, AdminApiError> {
            crate::settings_registry::effective_for(&state.store, key, &Default::default())
                .await
                .map_err(|_| unavailable())?
                .map(|v| v.value)
                .ok_or_else(unavailable)
        }
        let enforce_sessions = value(state, "security.signin.geoEnforceSessions")
            .await?
            .as_bool()
            .ok_or_else(unavailable)?;
        let scope = value(state, "security.signin.geoScope")
            .await?
            .as_str()
            .ok_or_else(unavailable)?
            .to_owned();
        let mode = value(state, "security.signin.geoMode")
            .await?
            .as_str()
            .ok_or_else(unavailable)?
            .to_owned();
        let unknown = value(state, "security.signin.geoUnknown")
            .await?
            .as_str()
            .ok_or_else(unavailable)?
            .to_owned();
        let countries = value(state, "security.signin.geoCountries")
            .await?
            .as_array()
            .ok_or_else(unavailable)?
            .iter()
            .map(|v| v.as_str().map(str::to_owned).ok_or_else(unavailable))
            .collect::<Result<Vec<_>, _>>()?;
        if !matches!(scope.as_str(), "off" | "admin" | "mailbox" | "all")
            || !matches!(mode.as_str(), "allow" | "deny")
            || !matches!(unknown.as_str(), "allow" | "deny")
            || countries
                .iter()
                .any(|s| s.len() != 2 || !s.bytes().all(|b| b.is_ascii_uppercase()))
        {
            return Err(unavailable());
        }
        Ok(Self {
            scope,
            mode,
            countries,
            unknown,
            enforce_sessions,
        })
    }
    fn applies(&self, surface: AuthSurface) -> bool {
        self.scope == "all"
            || matches!(
                (self.scope.as_str(), surface),
                ("admin", AuthSurface::Admin) | ("mailbox", AuthSurface::Webmail)
            )
    }
    fn decision(&self, surface: AuthSurface, lookup: &CountryLookup) -> &'static str {
        if !self.applies(surface) {
            return "off";
        }
        match lookup {
            CountryLookup::DatabaseUnavailable => "unavailable",
            CountryLookup::Unknown => {
                if self.unknown == "allow" {
                    "allow"
                } else {
                    "deny"
                }
            }
            CountryLookup::Known(country) => {
                let listed = self.countries.contains(country);
                if (self.mode == "allow") == listed {
                    "allow"
                } else {
                    "deny"
                }
            }
        }
    }
}
pub(crate) async fn check(
    state: &AppState,
    surface: AuthSurface,
    source: Option<IpAddr>,
) -> Result<(), AdminApiError> {
    let policy = Policy::load(state).await?;
    if !policy.applies(surface) {
        return Ok(());
    }
    match policy.decision(surface, &country_lookup(source)) {
        "allow" => Ok(()),
        "unavailable" => Err(unavailable()),
        _ => Err(AdminApiError::new(
            StatusCode::FORBIDDEN,
            "signinGeographyDenied",
            "Sign-in is not permitted from this source location",
        )),
    }
}
/// Request admission does not mint or revoke a session. A blocked client can
/// still sign out and use the public sign-in/recovery UI; successful sign-in
/// completion remains subject to the original country policy.
pub(crate) async fn session_guard(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let Some(surface) = session_surface(request.uri().path(), request.headers()) else {
        return next.run(request).await;
    };
    let request = if let Some(kind) = signed_content_kind(request.uri().path()) {
        use axum::extract::FromRequestParts;
        let (mut parts, body) = request.into_parts();
        let token = axum::extract::Path::<String>::from_request_parts(&mut parts, &state).await;
        let valid = token.ok().is_some_and(|token| {
            let key = state
                .store
                .derive_key(crate::image_proxy::remote_content::KEY_LABEL);
            crate::image_proxy::remote_content::verify(
                &key,
                &token.0,
                chrono::Utc::now().timestamp(),
            )
            .is_ok_and(|claims| claims.k == kind)
        });
        let request = axum::extract::Request::from_parts(parts, body);
        if !valid {
            return next.run(request).await;
        }
        request
    } else {
        request
    };
    let enabled = crate::settings_registry::effective_for(
        &state.store,
        "security.signin.geoEnforceSessions",
        &Default::default(),
    )
    .await
    .ok()
    .flatten()
    .and_then(|effective| effective.value.as_bool());
    match enabled {
        Some(false) => return next.run(request).await,
        Some(true) => {}
        None => return unavailable().into_response(),
    }
    let policy = match Policy::load(&state).await {
        Ok(policy) => policy,
        Err(error) => return error.into_response(),
    };
    if !policy.enforce_sessions || !policy.applies(surface) {
        return next.run(request).await;
    }
    let source = crate::scope_mw::proxy::client_ip(request.headers(), request.extensions());
    let decision = policy.decision(surface, &country_lookup(source));
    if decision == "allow" {
        return next.run(request).await;
    }
    let mut response = AdminApiError::new(
        if decision == "unavailable" {
            StatusCode::SERVICE_UNAVAILABLE
        } else {
            StatusCode::FORBIDDEN
        },
        if decision == "unavailable" {
            "sessionGeographyUnavailable"
        } else {
            "sessionGeographyDenied"
        },
        "This authenticated request is not permitted from the current source location",
    )
    .into_response();
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        "private, no-store".parse().unwrap(),
    );
    response
}
fn signed_content_kind(path: &str) -> Option<crate::image_proxy::remote_content::Kind> {
    use crate::image_proxy::remote_content::Kind;
    [
        ("/api/rc/i/", Kind::Image),
        ("/api/rc/p/", Kind::Part),
        ("/api/rc/f/", Kind::File),
    ]
    .into_iter()
    .find_map(|(prefix, kind)| path.starts_with(prefix).then_some(kind))
}
fn session_surface(path: &str, headers: &HeaderMap) -> Option<AuthSurface> {
    if matches!(
        path,
        "/healthz"
            | "/readyz"
            | "/api/logout"
            | "/api/sso/logout"
            | "/api/public-config"
            | "/api/rc/frame"
            | "/oidc/.well-known/openid-configuration"
            | "/oidc/jwks"
            | "/oidc/logout"
            | "/oidc/logout/context"
    ) || path == "/api/login"
        || path.starts_with("/api/login/")
    {
        return None;
    }
    let admin = mw_admin::config::admin_path();
    if let Some(suffix) = path
        .strip_prefix(&admin)
        .filter(|s| s.is_empty() || s.starts_with('/'))
    {
        if suffix.is_empty()
            || suffix == "/logout"
            || suffix == "/login"
            || suffix.starts_with("/login/")
        {
            return None;
        }
        let cookie = crate::hardening::cookie(headers, &mw_admin::config::session_cookie_name());
        return (cookie.is_some() || crate::push_relay::bearer_token(headers).is_some())
            .then_some(AuthSurface::Admin);
    }
    if path != "/mcp"
        && !["/api/", "/jmap/", "/dav/", "/oauth/", "/oidc/", "/mcp/"]
            .iter()
            .any(|prefix| path.starts_with(prefix))
    {
        return None;
    }
    // Opaque reader frames intentionally omit cookies. Their signed image/part/
    // file URLs are session-bound authority and must not bypass this policy.
    let signed_content = signed_content_kind(path).is_some();
    (signed_content
        || headers.contains_key("x-api-key")
        || crate::cookie_value(headers).is_some()
        || crate::push_relay::bearer_token(headers).is_some())
    .then_some(AuthSurface::Webmail)
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/bans/geography",
        Capability::BansRead,
        diagnostic,
    );
}
async fn diagnostic(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    ext: axum::http::Extensions,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BansRead)?;
    let policy = Policy::load(&state).await?;
    let source = crate::scope_mw::proxy::client_ip(&headers, &ext);
    let lookup = country_lookup(source);
    let country = match &lookup {
        CountryLookup::Known(country) => Some(country),
        _ => None,
    };
    Ok(([(axum::http::header::CACHE_CONTROL,"private, no-store")],Json(json!({
  "sourceIp":source.map(|ip|ip.to_canonical().to_string()),"country":country,
  "database":if matches!(lookup,CountryLookup::DatabaseUnavailable){"unavailable"}else{"ready"},
  "scope":policy.scope,"mode":policy.mode,"countries":policy.countries,"unknown":policy.unknown,
  "administratorDecision":policy.decision(AuthSurface::Admin,&lookup),"mailboxDecision":policy.decision(AuthSurface::Webmail,&lookup),"enforceSessions":policy.enforce_sessions,"scopeDescription":if policy.enforce_sessions {"sign-ins-and-session-requests"} else {"new-sign-ins-only"}
 }))).into_response())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_surface_keeps_mixed_cookies_and_recovery_separate() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            format!(
                "{}=mail; {}=admin",
                crate::COOKIE_NAME,
                mw_admin::config::session_cookie_name()
            )
            .parse()
            .unwrap(),
        );
        assert!(matches!(
            session_surface("/api/me", &headers),
            Some(AuthSurface::Webmail)
        ));
        assert!(matches!(
            session_surface(&mw_admin::config::admin_route("/me"), &headers),
            Some(AuthSurface::Admin)
        ));
        assert!(session_surface("/admin-other/me", &headers).is_none());
        for path in [
            "/",
            "/assets/app.js",
            "/api/logout",
            "/api/sso/logout",
            "/api/public-config",
            "/api/login/passkey/finish",
            "/oidc/.well-known/openid-configuration",
        ] {
            assert!(session_surface(path, &headers).is_none(), "{path}");
        }
        assert!(session_surface(&mw_admin::config::admin_route("/logout"), &headers).is_none());
        headers.clear();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            "Bearer native-fixture".parse().unwrap(),
        );
        assert!(matches!(
            session_surface("/jmap/api", &headers),
            Some(AuthSurface::Webmail)
        ));
        assert!(matches!(
            session_surface(&mw_admin::config::admin_route("/me"), &headers),
            Some(AuthSurface::Admin)
        ));
        headers.clear();
        headers.insert("x-api-key", "mwk_fixture".parse().unwrap());
        assert!(matches!(
            session_surface("/api/v1/messages", &headers),
            Some(AuthSurface::Webmail)
        ));
        assert!(matches!(
            session_surface("/mcp", &headers),
            Some(AuthSurface::Webmail)
        ));
        headers.clear();
        assert!(matches!(
            session_surface("/api/rc/p/signed", &headers),
            Some(AuthSurface::Webmail)
        ));
        assert!(session_surface("/api/rc/frame", &headers).is_none());
    }
    #[test]
    fn policy_is_explicit_for_scope_unknown_and_database_failure() {
        let mut p = Policy {
            scope: "off".into(),
            mode: "allow".into(),
            countries: vec!["PT".into()],
            unknown: "deny".into(),
            enforce_sessions: false,
        };
        assert_eq!(
            p.decision(AuthSurface::Admin, &CountryLookup::DatabaseUnavailable),
            "off"
        );
        p.scope = "mailbox".into();
        assert_eq!(
            p.decision(AuthSurface::Admin, &CountryLookup::Known("US".into())),
            "off"
        );
        assert_eq!(
            p.decision(AuthSurface::Webmail, &CountryLookup::Known("PT".into())),
            "allow"
        );
        assert_eq!(
            p.decision(AuthSurface::Webmail, &CountryLookup::Known("US".into())),
            "deny"
        );
        assert_eq!(
            p.decision(AuthSurface::Webmail, &CountryLookup::Unknown),
            "deny"
        );
        p.unknown = "allow".into();
        assert_eq!(
            p.decision(AuthSurface::Webmail, &CountryLookup::Unknown),
            "allow"
        );
        assert_eq!(
            p.decision(AuthSurface::Webmail, &CountryLookup::DatabaseUnavailable),
            "unavailable"
        );
        p.mode = "deny".into();
        assert_eq!(
            p.decision(AuthSurface::Webmail, &CountryLookup::Known("PT".into())),
            "deny"
        );
        assert_eq!(
            p.decision(AuthSurface::Webmail, &CountryLookup::Known("US".into())),
            "allow"
        );
    }
}
