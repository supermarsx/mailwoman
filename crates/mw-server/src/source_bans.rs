//! Deployment IP bans use the same trusted-proxy authority as sign-in limits.
use crate::AppState;
use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub(crate) async fn guard(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    // Liveness/readiness remain usable by deployment probes, even if their
    // transport address is banned. There is no blanket loopback exemption.
    if matches!(request.uri().path(), "/healthz" | "/readyz") {
        return next.run(request).await;
    }
    let Some(ip) = crate::scope_mw::proxy::client_ip(request.headers(), request.extensions())
    else {
        // In-process synthetic requests have no peer; never trust a supplied
        // forwarding header as a substitute. Production installs ConnectInfo.
        return next.run(request).await;
    };
    let status = match banned(&state, &ip.to_string(), &mut request).await {
        Ok(false) => return next.run(request).await,
        Ok(true) => StatusCode::FORBIDDEN,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    };
    let mut response = (
        status,
        Json(serde_json::json!({"code":"sourceUnavailable","error":"Request unavailable."})),
    )
        .into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
}

/// `MW_GUARD_SNAPSHOT=bounded` (t34 step 2): the parsed ban list is reused
/// while the mirror shows no change to the `protection` global epoch (which
/// every `v6:admin:bans` write bumps) or any other global/settings revision.
/// Expiry is still evaluated per request against now.
/// Off mode (t50hp): the parsed list is reused while the request's settings
/// revision is the one it was read under (0123: every change of
/// `v6:admin:bans` bumps it), see [`crate::settings_request_revision::RevisionMemo`].
async fn banned(state: &AppState, ip: &str, request: &mut Request) -> Result<bool, mw_admin::AdminError> {
    let Some(worker) = state.native_publication.as_ref()
        .filter(|_| crate::request_inputs::snapshot::mirror(state).is_some())
    else {
        static MEMO: crate::settings_request_revision::RevisionMemo<Vec<mw_admin::BanEntry>> =
            crate::settings_request_revision::RevisionMemo::new();
        let bans = crate::settings_request_revision::memoised(
            &MEMO, &state.store, request, state.v6.admin.list_bans(),
        ).await?;
        let now = chrono::Utc::now();
        return Ok(bans.iter().any(|ban| ban.matches_ip(ip) && ban.active_at(now)));
    };
    let bans = Box::pin(worker.source_bans.get_or_load(state, state.v6.admin.list_bans())).await?;
    let now = chrono::Utc::now();
    Ok(bans.iter().any(|ban| ban.matches_ip(ip) && ban.active_at(now)))
}
