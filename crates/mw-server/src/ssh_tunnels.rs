//! Deployment-wide saved SSH authorities. No credential is returned by this API.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson, with_etag},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::Engine as _;
use mw_ssh_tunnel::HostKeyPin;
use mw_store::{SshCredentialChange, SshTunnelDelete, SshTunnelMutation, SshTunnelRow};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    net::IpAddr,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};
#[path = "ssh_tunnel_transport.rs"]
mod transport;
pub(crate) use transport::hosting_dialer;
#[path = "ssh_tunnel_inspect.rs"]
mod inspect_draft;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Target {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub allow_private: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Config {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub enabled: bool,
    #[serde(default)]
    pub allow_private: bool,
    pub host_key_pins: Vec<HostKeyPin>,
    pub targets: Vec<Target>,
    pub keepalive_seconds: u32,
    pub connect_timeout_ms: u32,
    pub max_channels: u32,
}
fn invalid(code: &'static str, detail: &str, field: &str) -> AdminApiError {
    AdminApiError::invalid(code, detail, field)
}
pub(crate) fn normalize_host(host: &str) -> Result<String, AdminApiError> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ip.to_string());
    }
    if host.len() > 253
        || host.is_empty()
        || host.chars().any(|c| c.is_control() || c.is_whitespace())
        || host.contains(['/', ':', '@', '*', '\\'])
    {
        return Err(invalid(
            "invalidHost",
            "Use a hostname or IP address without a scheme or port",
            "/host",
        ));
    }
    crate::domain_config::normalize_domain(host)
        .map_err(|_| invalid("invalidHost", "Invalid hostname", "/host"))
}
impl Config {
    fn validate(&mut self) -> Result<(), AdminApiError> {
        if self.name.trim().is_empty()
            || self.name.len() > 128
            || self.name.chars().any(char::is_control)
        {
            return Err(invalid(
                "invalidName",
                "Use a nonempty name up to 128 bytes",
                "/name",
            ));
        }
        self.host = normalize_host(&self.host)?;
        if self.port == 0
            || self.username.is_empty()
            || self.username.len() > 128
            || self.username.chars().any(char::is_control)
        {
            return Err(invalid(
                "invalidAuthority",
                "A port and SSH username are required",
                "/username",
            ));
        }
        if !(10..=300).contains(&self.keepalive_seconds)
            || !(100..=60000).contains(&self.connect_timeout_ms)
            || !(1..=128).contains(&self.max_channels)
        {
            return Err(invalid(
                "invalidLimits",
                "Keepalive must be 10–300 seconds, timeout 100–60000 ms and channels 1–128",
                "/connectTimeoutMs",
            ));
        }
        if self.targets.len() > 128 || self.host_key_pins.len() > 16 {
            return Err(invalid(
                "tooManyEntries",
                "At most 128 targets and 16 pins are allowed",
                "/targets",
            ));
        }
        let mut targets = BTreeSet::new();
        for target in &mut self.targets {
            target.host = normalize_host(&target.host)?;
            if target.port == 0 || !targets.insert((target.host.clone(), target.port)) {
                return Err(invalid(
                    "invalidTargets",
                    "Targets must have nonzero ports and be unique",
                    "/targets",
                ));
            }
        }
        self.host_key_pins
            .sort_by(|a, b| (&a.algorithm, &a.sha256).cmp(&(&b.algorithm, &b.sha256)));
        self.host_key_pins.dedup();
        for pin in &self.host_key_pins {
            if !matches!(
                pin.algorithm.as_str(),
                "ssh-ed25519" | "ecdsa-sha2-nistp256" | "ecdsa-sha2-nistp384" | "ssh-rsa"
            ) || pin
                .sha256
                .strip_prefix("SHA256:")
                .and_then(|v| {
                    base64::engine::general_purpose::STANDARD_NO_PAD
                        .decode(v)
                        .ok()
                })
                .is_none_or(|bytes| bytes.len() != 32)
            {
                return Err(invalid(
                    "invalidHostKeyPin",
                    "Use an approved algorithm and a SHA256 fingerprint",
                    "/hostKeyPins",
                ));
            }
        }
        if self.enabled && (self.targets.is_empty() || self.host_key_pins.is_empty()) {
            return Err(invalid(
                "incompleteTunnel",
                "Enabled tunnels require verified pins and allowed targets",
                "/enabled",
            ));
        }
        Ok(())
    }
    fn authority(&self) -> (&str, u16, &str, &[HostKeyPin]) {
        (&self.host, self.port, &self.username, &self.host_key_pins)
    }
    pub(crate) fn from_row(row: &SshTunnelRow) -> Result<Self, AdminApiError> {
        Ok(Self {
            id: row.id.clone(),
            name: row.name.clone(),
            host: row.host.clone(),
            port: row.port,
            username: row.username.clone(),
            enabled: row.enabled,
            allow_private: row.allow_private,
            host_key_pins: serde_json::from_str(&row.pins_json)
                .map_err(|_| AdminApiError::internal("Invalid saved SSH pins"))?,
            targets: serde_json::from_str(&row.targets_json)
                .map_err(|_| AdminApiError::internal("Invalid saved SSH targets"))?,
            keepalive_seconds: row.keepalive_seconds,
            connect_timeout_ms: row.connect_timeout_ms,
            max_channels: row.max_channels,
        })
    }
    fn row(&self, ctx: &AdminCtx) -> SshTunnelRow {
        let now = chrono::Utc::now().timestamp();
        SshTunnelRow {
            id: self.id.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            port: self.port,
            username: self.username.clone(),
            enabled: self.enabled,
            allow_private: self.allow_private,
            pins_json: serde_json::to_string(&self.host_key_pins).expect("serializable pins"),
            targets_json: serde_json::to_string(&self.targets).expect("serializable targets"),
            keepalive_seconds: self.keepalive_seconds,
            connect_timeout_ms: self.connect_timeout_ms,
            max_channels: self.max_channels,
            credential_configured: false,
            auth_kind: None,
            public_key: String::new(),
            key_algorithm: String::new(),
            revision: String::new(),
            created_by: ctx.username.clone(),
            created_at: now,
            updated_at: now,
        }
    }
}
#[derive(Default, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase", deny_unknown_fields)]
enum CredentialInput {
    #[default]
    Keep,
    Clear,
    Generate,
    Set {
        kind: String,
        #[serde(rename = "privateKey")]
        private_key: Option<String>,
        passphrase: Option<String>,
        password: Option<String>,
    },
}
fn credential(input: CredentialInput) -> Result<SshCredentialChange, AdminApiError> {
    let (pem, passphrase) = match input {
        CredentialInput::Keep => return Ok(SshCredentialChange::Keep),
        CredentialInput::Clear => return Ok(SshCredentialChange::Clear),
        CredentialInput::Generate => (
            mw_ssh_tunnel::generate_ed25519()
                .map_err(|_| AdminApiError::internal("SSH key generation failed"))?,
            None,
        ),
        CredentialInput::Set {
            kind,
            private_key,
            passphrase,
            password,
        } => {
            if kind == "password" && private_key.is_none() && passphrase.is_none() {
                let password = password
                    .filter(|p| !p.is_empty() && p.len() <= 4096)
                    .ok_or_else(|| {
                        invalid(
                            "invalidCredential",
                            "A password up to 4096 bytes is required",
                            "/credential",
                        )
                    })?;
                return Ok(SshCredentialChange::Set {
                    secret: password,
                    kind,
                    public_key: String::new(),
                    algorithm: String::new(),
                });
            }
            if kind != "privateKey" || password.is_some() {
                return Err(invalid(
                    "invalidCredential",
                    "Choose a private key or password",
                    "/credential",
                ));
            }
            (
                private_key.filter(|k| k.len() <= 65536).ok_or_else(|| {
                    invalid(
                        "invalidCredential",
                        "A private key up to 64 KiB is required",
                        "/credential",
                    )
                })?,
                passphrase,
            )
        }
    };
    let (key, public_key, algorithm) = mw_ssh_tunnel::import_private_key(
        &pem,
        passphrase.as_deref(),
    )
    .map_err(|_| {
        invalid(
            "invalidPrivateKey",
            "Use Ed25519, P-256, P-384 or RSA of at least 3072 bits; verify the import passphrase",
            "/credential",
        )
    })?;
    let normalized = mw_ssh_tunnel::normalized_private_key(&key)
        .map_err(|_| AdminApiError::internal("SSH key normalization failed"))?;
    Ok(SshCredentialChange::Set {
        secret: normalized.to_string(),
        kind: "privateKey".into(),
        public_key,
        algorithm,
    })
}
fn parse_write(mut value: Value) -> Result<(Config, CredentialInput, bool), AdminApiError> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| invalid("invalidTunnel", "Expected an object", "/"))?;
    let input = serde_json::from_value(
        object
            .remove("credential")
            .unwrap_or(json!({"action":"keep"})),
    )
    .map_err(|_| {
        invalid(
            "invalidCredential",
            "Invalid credential action",
            "/credential",
        )
    })?;
    let confirm = match object.remove("confirmAuthorityChange") {
        None => false,
        Some(Value::Bool(v)) => v,
        _ => {
            return Err(invalid(
                "invalidConfirmation",
                "Confirmation must be a boolean",
                "/confirmAuthorityChange",
            ));
        }
    };
    let config = serde_json::from_value(value)
        .map_err(|_| invalid("invalidTunnel", "Invalid tunnel fields", "/"))?;
    Ok((config, input, confirm))
}
fn etag(row: &SshTunnelRow) -> String {
    format!("\"{}\"", row.revision)
}
fn require_match(headers: &HeaderMap, row: &SshTunnelRow) -> Result<(), AdminApiError> {
    match headers.get(header::IF_MATCH) {
        None => Err(AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "preconditionRequired",
            "Reload the tunnel before changing it",
        )),
        Some(v) if v.to_str().ok() == Some(etag(row).as_str()) => Ok(()),
        _ => Err(AdminApiError::stale(None, Some(row.updated_at))),
    }
}
async fn saved(state: &AppState, id: &str) -> Result<SshTunnelRow, AdminApiError> {
    state
        .store
        .ssh_tunnel(id)
        .await?
        .ok_or_else(|| AdminApiError::new(StatusCode::NOT_FOUND, "notFound", "Tunnel not found"))
}
fn redacted(row: &SshTunnelRow) -> Result<Value, AdminApiError> {
    let mut value = serde_json::to_value(Config::from_row(row)?).expect("serializable config");
    let o = value.as_object_mut().expect("object");
    o.insert("revision".into(), json!(row.revision));
    o.insert(
        "credentialConfigured".into(),
        json!(row.credential_configured),
    );
    o.insert("authKind".into(), json!(row.auth_kind));
    o.insert(
        "publicKey".into(),
        if row.public_key.is_empty() {
            Value::Null
        } else {
            json!(row.public_key)
        },
    );
    o.insert("createdAt".into(), json!(row.created_at));
    o.insert("updatedAt".into(), json!(row.updated_at));
    Ok(value)
}
fn response(row: &SshTunnelRow) -> Result<Response, AdminApiError> {
    Ok(with_etag(
        (
            [(header::CACHE_CONTROL, "private, no-store")],
            Json(redacted(row)?),
        ),
        &etag(row),
    ))
}
async fn step_up(state: &AppState, ctx: &AdminCtx) -> Result<(), AdminApiError> {
    crate::step_up::require_admin(state, ctx, crate::step_up::StepUpAction::IntegrationManage).await
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::POST,
        "/tunnels/inspect-host-key",
        Capability::TunnelsTest,
        inspect_draft::inspect,
    );
    r.admin_endpoint(Method::GET, "/tunnels", Capability::TunnelsRead, list);
    r.admin_endpoint(Method::POST, "/tunnels", Capability::TunnelsWrite, create);
    r.admin_endpoint(Method::GET, "/tunnels/{id}", Capability::TunnelsRead, get);
    r.admin_endpoint(
        Method::PUT,
        "/tunnels/{id}",
        Capability::TunnelsWrite,
        update,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/tunnels/{id}",
        Capability::TunnelsWrite,
        delete,
    );
    r.admin_endpoint(
        Method::POST,
        "/tunnels/{id}/test",
        Capability::TunnelsTest,
        test,
    );
    r.admin_endpoint(
        Method::POST,
        "/tunnels/{id}/fetch-host-key",
        Capability::TunnelsTest,
        fetch_host_key,
    );
}
async fn list(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::TunnelsRead)?;
    let rows = state.store.ssh_tunnels().await?;
    Ok((
        [(header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"tunnels":rows.iter().map(redacted).collect::<Result<Vec<_>,_>>()?})),
    )
        .into_response())
}
async fn get(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::TunnelsRead)?;
    response(&saved(&state, &id).await?)
}
async fn write(
    state: &AppState,
    ctx: &AdminCtx,
    value: Value,
    old: Option<SshTunnelRow>,
) -> Result<SshTunnelRow, AdminApiError> {
    let (mut config, input, confirm) = parse_write(value)?;
    if let Some(old) = &old {
        if !config.id.is_empty() && config.id != old.id {
            return Err(invalid("idMismatch", "Tunnel ID cannot change", "/id"));
        }
        config.id = old.id.clone();
    } else {
        config.id = uuid::Uuid::new_v4().to_string();
    }
    config.validate()?;
    if matches!(input, CredentialInput::Keep)
        && old.as_ref().is_some_and(|r| r.credential_configured)
        && !confirm
    {
        let mut previous = Config::from_row(old.as_ref().expect("present"))?;
        previous
            .host_key_pins
            .sort_by(|a, b| (&a.algorithm, &a.sha256).cmp(&(&b.algorithm, &b.sha256)));
        if previous.authority() != config.authority() {
            return Err(invalid(
                "credentialReviewRequired",
                "Review the old and new SSH authority before retaining the saved credential",
                "/confirmAuthorityChange",
            ));
        }
    }
    let key_action = match &input {
        CredentialInput::Generate => Some("TunnelKeyGenerated"),
        CredentialInput::Set { kind, .. } if kind == "privateKey" => Some("TunnelKeyImported"),
        CredentialInput::Clear => Some("TunnelCredentialCleared"),
        _ => None,
    };
    let pins_changed = old.as_ref().is_some_and(|row| {
        row.pins_json != serde_json::to_string(&config.host_key_pins).expect("serializable pins")
    });
    let row = match state
        .store
        .put_ssh_tunnel(
            &config.row(ctx),
            old.as_ref().map(|r| r.revision.as_str()),
            credential(input)?,
        )
        .await?
    {
        SshTunnelMutation::Applied(row) => *row,
        SshTunnelMutation::Missing => {
            return Err(AdminApiError::new(
                StatusCode::NOT_FOUND,
                "notFound",
                "Tunnel not found",
            ));
        }
        SshTunnelMutation::Conflict => return Err(AdminApiError::stale(None, None)),
        SshTunnelMutation::CredentialRequired => {
            return Err(invalid(
                "credentialRequired",
                "Configure a credential before enabling the tunnel",
                "/credential",
            ));
        }
    };
    transport::invalidate(&row.id).await;
    crate::admin_accounts::audit(
        state,
        ctx,
        if pins_changed {
            "TunnelPinChanged"
        } else if old.is_some() {
            "TunnelChanged"
        } else {
            "TunnelCreated"
        },
        &format!("tunnel:{}", row.id),
        None,
        Some(json!({"name":row.name})),
    )
    .await;
    if let Some(action) = key_action {
        crate::admin_accounts::audit(
            state,
            ctx,
            action,
            &format!("tunnel:{}", row.id),
            None,
            None,
        )
        .await;
    }
    Ok(row)
}
async fn create(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(value): AdminJson<Value>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::TunnelsWrite)?;
    step_up(&state, &ctx).await?;
    let row = write(&state, &ctx, value, None).await?;
    let mut result = response(&row)?;
    *result.status_mut() = StatusCode::CREATED;
    Ok(result)
}
async fn update(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    headers: HeaderMap,
    AdminJson(value): AdminJson<Value>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::TunnelsWrite)?;
    step_up(&state, &ctx).await?;
    let old = saved(&state, &id).await?;
    require_match(&headers, &old)?;
    response(&write(&state, &ctx, value, Some(old)).await?)
}
async fn delete(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::TunnelsWrite)?;
    step_up(&state, &ctx).await?;
    let old = saved(&state, &id).await?;
    require_match(&headers, &old)?;
    match state.store.delete_ssh_tunnel(&id, &old.revision).await? {
        SshTunnelDelete::Deleted => {}
        SshTunnelDelete::Bound => {
            return Err(AdminApiError::conflict(
                "tunnelBound",
                "Remove saved bindings before deleting this tunnel",
            ));
        }
        SshTunnelDelete::Missing | SshTunnelDelete::Conflict => {
            return Err(AdminApiError::stale(None, None));
        }
    }
    transport::invalidate(&id).await;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "TunnelDeleted",
        &format!("tunnel:{id}"),
        None,
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}
static PROBES: LazyLock<Mutex<BTreeMap<String, VecDeque<Instant>>>> =
    LazyLock::new(Default::default);
fn rate_limit(ctx: &AdminCtx) -> Result<(), AdminApiError> {
    let now = Instant::now();
    let mut map = PROBES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    map.retain(|_, times| {
        while times
            .front()
            .is_some_and(|t| now.duration_since(*t) >= Duration::from_secs(60))
        {
            times.pop_front();
        }
        !times.is_empty()
    });
    if map.len() >= 1024 && !map.contains_key(&ctx.username) {
        return Err(AdminApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "probeBusy",
            "Too many active probes",
        ));
    }
    let times = map.entry(ctx.username.clone()).or_default();
    if times.len() >= 6 {
        return Err(AdminApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "probeRateLimited",
            "At most six SSH probes per minute",
        ));
    }
    times.push_back(now);
    Ok(())
}
async fn fetch_host_key(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Json<HostKeyPin>, AdminApiError> {
    ctx.require_unscoped(Capability::TunnelsTest)?;
    step_up(&state, &ctx).await?;
    rate_limit(&ctx)?;
    let row = saved(&state, &id).await?;
    let config = Config::from_row(&row)?;
    let jump = transport::jump_dialer(&state, &config).await?;
    let pin = mw_ssh_tunnel::observe_host_key(
        jump.as_ref(),
        &config.host,
        config.port,
        Duration::from_millis(u64::from(config.connect_timeout_ms.clamp(100, 60000))),
    )
    .await
    .map_err(|_| {
        AdminApiError::conflict("hostKeyUnavailable", "Unable to inspect the SSH host key")
    })?;
    Ok(Json(pin))
}
async fn test(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Json<Value>, AdminApiError> {
    ctx.require_unscoped(Capability::TunnelsTest)?;
    step_up(&state, &ctx).await?;
    rate_limit(&ctx)?;
    let row = saved(&state, &id).await?;
    let config = Config::from_row(&row)?;
    let target = config
        .targets
        .first()
        .ok_or_else(|| invalid("targetRequired", "Save an allowed target first", "/targets"))?;
    let start = Instant::now();
    let dialer =
        hosting_dialer(&state, &id, &target.host, target.port, target.allow_private).await?;
    let result = tokio::time::timeout(
        Duration::from_millis(u64::from(config.connect_timeout_ms.clamp(100, 60000)) * 4),
        dialer.dial(&target.host, target.port),
    )
    .await;
    let (ok, code) = match result {
        Ok(Ok(stream)) => {
            drop(stream);
            (true, None)
        }
        Ok(Err(e)) => (false, Some(transport::safe_code(&e))),
        Err(_) => (false, Some("ssh_connect_timeout")),
    };
    let elapsed = start.elapsed();
    crate::admin_accounts::audit(
        &state,
        &ctx,
        if code == Some("host_key_mismatch") {
            "TunnelHostKeyRejected"
        } else {
            "TunnelTest"
        },
        &format!("tunnel:{id}"),
        None,
        Some(json!({"ok":ok,"code":code})),
    )
    .await;
    Ok(Json(
        json!({"ok":ok,"phase":if ok{"channel"}else{"connection"},"code":code,"latencyMs":elapsed.as_millis(),"latencyUs":elapsed.as_micros()}),
    ))
}
