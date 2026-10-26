//! Credential-free discovery for an unsaved SSH authority. Never changes trust.
use super::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Request {
    host: String,
    port: u16,
    #[serde(default)]
    allow_private: bool,
    connect_timeout_ms: u32,
}

pub(super) async fn inspect(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(request): AdminJson<Request>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::TunnelsTest)?;
    step_up(&state, &ctx).await?;
    let mut config = Config {
        id: String::new(),
        name: "Host key discovery".into(),
        host: request.host,
        port: request.port,
        username: "discovery".into(),
        enabled: false,
        allow_private: request.allow_private,
        host_key_pins: vec![],
        targets: vec![],
        keepalive_seconds: 30,
        connect_timeout_ms: request.connect_timeout_ms,
        max_channels: 1,
    };
    config.validate()?;
    rate_limit(&ctx)?;
    let jump = transport::jump_dialer(&state, &config).await?;
    let pin = mw_ssh_tunnel::observe_host_key(jump.as_ref(), &config.host, config.port,
        Duration::from_millis(u64::from(config.connect_timeout_ms))).await
        .map_err(|_| AdminApiError::conflict("hostKeyUnavailable",
            "Could not read the SSH host key. Check hostname, port, network access and private-network permission."))?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "TunnelHostKeyObserved",
        &format!("ssh:{}:{}", config.host, config.port),
        None,
        Some(json!({"algorithm":pin.algorithm,"fingerprint":pin.sha256,"trusted":false})),
    )
    .await;
    Ok(([(header::CACHE_CONTROL, "private, no-store")], Json(pin)).into_response())
}
