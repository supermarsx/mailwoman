//! Optional local-database country policy for new sign-ins only.
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
  "administratorDecision":policy.decision(AuthSurface::Admin,&lookup),"mailboxDecision":policy.decision(AuthSurface::Webmail,&lookup),"scopeDescription":"new-sign-ins-only"
 }))).into_response())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policy_is_explicit_for_scope_unknown_and_database_failure() {
        let mut p = Policy {
            scope: "off".into(),
            mode: "allow".into(),
            countries: vec!["PT".into()],
            unknown: "deny".into(),
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
