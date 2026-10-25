//! Operator-triggered discovery. Every HTTP hop follows the admin-detect route.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    capabilities::Capability,
};
use axum::{
    Json,
    extract::{Path, State},
};
use mw_autoconfig::{DiscoverError, Fetcher, SrvResolver};
use mw_egress::{
    TrafficClass,
    plan::{EgressPlan, Request},
};
use serde_json::{Value, json};

struct RoutedFetcher {
    plan: EgressPlan,
    domain: String,
    resolver: mw_autoconfig::HickoryResolver,
}

#[async_trait::async_trait]
impl Fetcher for RoutedFetcher {
    async fn srv(&self, service: &str) -> Result<Vec<mw_autoconfig::SrvRecord>, DiscoverError> {
        self.resolver.lookup_srv(service).await
    }

    async fn get(&self, url: &str) -> Result<Option<String>, DiscoverError> {
        let url = reqwest::Url::parse(url)
            .map_err(|_| DiscoverError::Lookup("invalid discovery URL".into()))?;
        let request = Request::get(TrafficClass::AdminDetect)
            .for_domain(&self.domain)
            .accept("application/xml, application/json;q=0.9, */*;q=0.8");
        let result = mw_egress::plan::send_with(&self.plan, url, &request).await;
        Ok(result
            .outcome
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok()))
    }
}

pub(crate) async fn detect(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(name): Path<String>,
) -> Result<Json<Value>, AdminApiError> {
    ctx.require_unscoped(Capability::DomainsUpstreamWrite)?;
    let name = crate::domain_config::normalize_domain(&name)
        .map_err(|error| AdminApiError::invalid("invalidDomain", error, "/name"))?;
    let plan = crate::image_proxy::load_plan(&state.store)
        .await
        .map_err(|_| {
            AdminApiError::conflict(
                "discoveryUnavailable",
                "Egress configuration is unavailable.",
            )
        })?;
    let resolver = mw_autoconfig::HickoryResolver::new().map_err(|_| {
        AdminApiError::conflict("discoveryUnavailable", "DNS resolver is unavailable.")
    })?;
    let fetcher = RoutedFetcher {
        plan,
        domain: name.clone(),
        resolver,
    };
    let candidate = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        mw_autoconfig::discover_with(&format!("postmaster@{name}"), &fetcher),
    )
    .await
    .map_err(|_| AdminApiError::conflict("discoveryTimeout", "Discovery timed out."))?
    .map_err(|_| AdminApiError::not_found("No upstream configuration was discovered."))?;
    let endpoint = |spec: &mw_autoconfig::ServerSpec| {
        json!({
            "host":spec.host,"port":spec.port,"tls":match spec.tls {
                mw_autoconfig::TlsMode::Implicit => "implicit",
                mw_autoconfig::TlsMode::StartTls => "starttls",
                mw_autoconfig::TlsMode::None => "none",
            },"auth":"password","allowPrivate":false
        })
    };
    let proposal = if state.engine.is_some()
        && candidate.source != mw_autoconfig::DiscoverySource::Jmap
    {
        if let Some(pop3) = &candidate.pop3 {
            json!({"pop3":endpoint(pop3),"smtp":endpoint(&candidate.smtp)})
        } else {
            json!({"imap":endpoint(&candidate.imap),"smtp":endpoint(&candidate.smtp)})
        }
    } else if state.engine.is_none() && candidate.source == mw_autoconfig::DiscoverySource::Jmap {
        json!({"jmap":{"sessionUrl":format!("https://{name}/.well-known/jmap")}})
    } else {
        return Err(AdminApiError::conflict(
            "discoveryModeMismatch",
            "The discovered protocol does not match the server mode.",
        ));
    };
    crate::domain_config::UpstreamConfig::parse(&proposal.to_string()).map_err(|_| {
        AdminApiError::conflict(
            "discoveryUnsafe",
            "The discovered endpoint requires manual review.",
        )
    })?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "domain-detected",
        &format!("domain:{name}"),
        None,
        None,
    )
    .await;
    Ok(Json(
        json!({"name":name,"upstream":proposal,"source":candidate.source,
        "requiresOAuth":candidate.auth == mw_autoconfig::AuthMethod::OAuth2,"saved":false}),
    ))
}
