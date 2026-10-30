//! Console routes for the country database (t96), under `{console}/security/geoip`.
//!
//! Reading needs unscoped `security.read`; every write needs unscoped
//! `security.write`. Changing the source (provider, key, custom URL) and
//! uploading a file also need a fresh step-up proof, because either decides which
//! database every country policy trusts. Every write is audited; no response,
//! audit row or log line carries the licence key or a URL that contains it.

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;

use super::{OpError, UpdateOutcome, provider};
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/security/geoip",
        Capability::SecurityRead,
        read,
    );
    r.admin_endpoint(
        Method::PUT,
        "/security/geoip/config",
        Capability::SecurityWrite,
        save,
    );
    r.admin_endpoint(
        Method::POST,
        "/security/geoip/download",
        Capability::SecurityWrite,
        download_now,
    );
    r.admin_endpoint(
        Method::POST,
        "/security/geoip/check",
        Capability::SecurityWrite,
        check,
    );
    r.admin_endpoint(
        Method::POST,
        "/security/geoip/rollback",
        Capability::SecurityWrite,
        rollback,
    );
    r.admin_endpoint(
        Method::POST,
        "/security/geoip/upload",
        Capability::SecurityWrite,
        upload,
    );
    r.admin_endpoint(
        Method::POST,
        "/security/geoip/lookup",
        Capability::SecurityRead,
        lookup,
    );
}

fn op_error(e: OpError) -> AdminApiError {
    let (status, code) = match &e {
        OpError::Environment => (StatusCode::CONFLICT, "geoipManagedByEnvironment"),
        OpError::NoDirectory => (StatusCode::CONFLICT, "geoipNoDataDirectory"),
        OpError::Busy => (StatusCode::CONFLICT, "geoipBusy"),
        OpError::NoPrevious => (StatusCode::CONFLICT, "geoipNoPrevious"),
        OpError::Invalid(_) => (StatusCode::UNPROCESSABLE_ENTITY, "geoipInvalidDatabase"),
        OpError::Config(_) => (StatusCode::UNPROCESSABLE_ENTITY, "geoipNotConfigured"),
        OpError::Fetch(_) => (StatusCode::BAD_GATEWAY, "geoipDownloadFailed"),
        OpError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "geoipStorageFailed"),
    };
    AdminApiError::new(status, code, e.to_string())
}

async fn view(state: &AppState) -> Response {
    let status = super::status(state).await;
    let tag = etag_of(&status["config"]);
    with_etag(
        ([(header::CACHE_CONTROL, "private, no-store")], Json(status)),
        &tag,
    )
}

async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityRead)?;
    Ok(view(&state).await)
}

fn refuse_when_environment() -> Result<(), AdminApiError> {
    if mw_engine::security::geoip::environment_path().is_some() {
        return Err(op_error(OpError::Environment));
    }
    Ok(())
}

async fn save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(update): AdminJson<provider::Update>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityWrite)?;
    refuse_when_environment()?;
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await?;
    let raw = state
        .store
        .get_setting(provider::KEY)
        .await
        .map_err(AdminApiError::internal)?;
    let previous = provider::load(&state.store)
        .await
        .map_err(|_| AdminApiError::internal("country database configuration"))?;
    if !headers.contains_key("if-match") {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "ifMatchRequired",
            "Reload the configuration before saving.",
        ));
    }
    check_if_match(&headers, &etag_of(&previous.public()), None, None)?;
    let key_changed = update.licence_key.is_some() || update.clear_licence_key;
    let config = update
        .prepare(&previous, &state.store)
        .map_err(|m| AdminApiError::invalid("invalidGeoipConfig", m, "/config"))?;
    let serialized = serde_json::to_string(&config).expect("geoip config");
    if !state
        .store
        .compare_and_set_setting(provider::KEY, raw.as_deref(), &serialized)
        .await
        .map_err(AdminApiError::internal)?
    {
        return Err(AdminApiError::stale(None, None));
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "geoip-configured",
        "geoip:config",
        Some(json!({"provider":previous.provider,"autoUpdate":previous.auto_update,"intervalDays":previous.interval_days,"customUrl":previous.custom_url})),
        Some(json!({"provider":config.provider,"autoUpdate":config.auto_update,"intervalDays":config.interval_days,
            "customUrl":config.custom_url,"licenceKeyChanged":key_changed,"licenceKeyConfigured":config.licence_key_sealed.is_some()})),
    )
    .await;
    Ok(view(&state).await)
}

fn outcome_json(outcome: &UpdateOutcome) -> serde_json::Value {
    match outcome {
        UpdateOutcome::Installed(s) => {
            json!({"result":"installed","source":s.source,"release":s.release,
            "sha256":s.sha256,"buildEpoch":s.build_epoch,"sizeBytes":s.size_bytes,"checksumVerified":s.checksum_verified})
        }
        UpdateOutcome::UpToDate => json!({"result":"up-to-date"}),
    }
}

async fn run_update(
    state: AppState,
    ctx: AdminCtx,
    force: bool,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityWrite)?;
    refuse_when_environment()?;
    let config = provider::load(&state.store).await.unwrap_or_default();
    let outcome = super::update(&state, &ctx.username, force).await;
    let action = if force {
        "geoip-database-downloaded"
    } else {
        "geoip-database-checked"
    };
    let detail = match &outcome {
        Ok(o) => outcome_json(o),
        Err(e) => json!({"result":"failed","source":config.provider,"reason":e.to_string()}),
    };
    if !matches!(outcome, Err(OpError::Busy | OpError::Environment)) {
        crate::admin_accounts::audit(
            &state,
            &ctx,
            action,
            "geoip:database",
            None,
            Some(detail.clone()),
        )
        .await;
        super::record_check(
            &state,
            outcome.as_ref().map(|_| ()).map_err(|e| e.to_string()),
            config.interval_days,
        )
        .await;
    }
    outcome.map_err(op_error)?;
    let mut body = super::status(&state).await;
    body["outcome"] = detail;
    Ok(([(header::CACHE_CONTROL, "private, no-store")], Json(body)).into_response())
}

async fn download_now(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    run_update(state, ctx, true).await
}

async fn check(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    run_update(state, ctx, false).await
}

async fn rollback(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityWrite)?;
    refuse_when_environment()?;
    let outcome = super::rollback(&ctx.username).await;
    if !matches!(outcome, Err(OpError::Busy)) {
        let detail = match &outcome {
            Ok(s) => {
                json!({"result":"rolled-back","source":s.source,"release":s.release,"sha256":s.sha256,"buildEpoch":s.build_epoch})
            }
            Err(e) => json!({"result":"failed","reason":e.to_string()}),
        };
        crate::admin_accounts::audit(
            &state,
            &ctx,
            "geoip-database-rolled-back",
            "geoip:database",
            None,
            Some(detail),
        )
        .await;
    }
    outcome.map_err(op_error)?;
    Ok(view(&state).await)
}

/// The largest upload accepted: a gzip of a database or the database itself.
const MAX_UPLOAD: usize = super::download::MAX_DOWNLOAD_BYTES;

async fn upload(
    State(state): State<AppState>,
    ctx: AdminCtx,
    body: Body,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityWrite)?;
    refuse_when_environment()?;
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await?;
    let bytes = axum::body::to_bytes(body, MAX_UPLOAD).await.map_err(|_| {
        AdminApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "geoipUploadTooLarge",
            "The file is larger than the 128 MB upload limit.",
        )
    })?;
    if bytes.is_empty() {
        return Err(AdminApiError::invalid(
            "geoipUploadEmpty",
            "Choose a .mmdb file to upload.",
            "/file",
        ));
    }
    let size = bytes.len();
    let outcome = super::install_upload(bytes.to_vec(), &ctx.username).await;
    if !matches!(outcome, Err(OpError::Busy)) {
        let detail = match &outcome {
            Ok(s) => {
                json!({"result":"installed","source":"upload","sha256":s.sha256,"buildEpoch":s.build_epoch,
                "databaseType":s.database_type,"sizeBytes":s.size_bytes})
            }
            Err(e) => json!({"result":"refused","uploadBytes":size,"reason":e.to_string()}),
        };
        crate::admin_accounts::audit(
            &state,
            &ctx,
            "geoip-database-uploaded",
            "geoip:database",
            None,
            Some(detail),
        )
        .await;
    }
    outcome.map_err(op_error)?;
    Ok(view(&state).await)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Lookup {
    ip: String,
}

/// A test lookup: the country the database in use gives an address. Unscoped
/// `security.read` only; the address is the administrator's own input and is not
/// stored or logged.
async fn lookup(
    State(_state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<Lookup>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityRead)?;
    let ip: std::net::IpAddr = body.ip.trim().parse().map_err(|_| {
        AdminApiError::invalid("invalidAddress", "Enter an IPv4 or IPv6 address.", "/ip")
    })?;
    let ip = ip.to_canonical();
    let result = match mw_engine::security::country_lookup(Some(ip)) {
        mw_engine::security::CountryLookup::Known(country) => {
            json!({"ip":ip.to_string(),"database":"ready","country":country})
        }
        mw_engine::security::CountryLookup::Unknown => {
            json!({"ip":ip.to_string(),"database":"ready","country":null})
        }
        mw_engine::security::CountryLookup::DatabaseUnavailable => {
            json!({"ip":ip.to_string(),"database":"unavailable","country":null})
        }
    };
    Ok(([(header::CACHE_CONTROL, "private, no-store")], Json(result)).into_response())
}
