//! Saved panel authority. Credentials are stored separately and never serialized here.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PanelProvider {
    Virtualmin,
    Hestia,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum PanelConnection {
    Direct,
    Ssh {
        #[serde(rename = "tunnelId")]
        tunnel_id: String,
    },
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PanelTls {
    pub server_name: Option<String>,
    pub ca_pem: Option<String>,
    pub min_version: String,
}
impl Default for PanelTls {
    fn default() -> Self {
        Self {
            server_name: None,
            ca_pem: None,
            min_version: "1.2".into(),
        }
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PanelConfig {
    pub id: String,
    pub name: String,
    pub provider: PanelProvider,
    pub base_url: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub access_key: String,
    #[serde(default)]
    pub allow_private: bool,
    pub connection: PanelConnection,
    #[serde(default)]
    pub tls: PanelTls,
    /// Virtualmin domain lifecycle affects the whole virtual server, not just mail.
    #[serde(default)]
    pub allow_domain_lifecycle: bool,
}

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
use mw_store::{HostingConnectionRow, HostingCredentialChange, HostingMutation};
use serde_json::{Value, json};

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/integrations/hosting/connections",
        Capability::IntegrationsRead,
        list,
    );
    r.admin_endpoint(
        Method::POST,
        "/integrations/hosting/connections",
        Capability::IntegrationsWrite,
        create,
    );
    r.admin_endpoint(
        Method::GET,
        "/integrations/hosting/connections/{id}",
        Capability::IntegrationsRead,
        get,
    );
    r.admin_endpoint(
        Method::PUT,
        "/integrations/hosting/connections/{id}",
        Capability::IntegrationsWrite,
        update,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/integrations/hosting/connections/{id}",
        Capability::IntegrationsWrite,
        delete,
    );
    r.admin_endpoint(
        Method::POST,
        "/integrations/hosting/connections/{id}/actions",
        Capability::IntegrationsRead,
        action,
    );
}
fn invalid(detail: &str, field: &str) -> AdminApiError {
    AdminApiError::invalid("invalidHostingConnection", detail, field)
}
fn store_error(_: mw_store::StoreError) -> AdminApiError {
    AdminApiError::internal("Hosting connection storage failed")
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}
fn bounded(value: &str, max: usize) -> bool {
    value.len() <= max && !value.chars().any(char::is_control)
}
impl PanelConfig {
    pub(crate) fn validate(&mut self) -> Result<(), AdminApiError> {
        if !identifier(&self.id) {
            return Err(invalid("Invalid connection ID", "/config/id"));
        }
        if self.name.trim().is_empty() || !bounded(&self.name, 120) {
            return Err(invalid(
                "Connection name must contain 1 to 120 bytes without control characters",
                "/config/name",
            ));
        }
        let url = reqwest::Url::parse(&self.base_url)
            .map_err(|_| invalid("An HTTPS panel origin is required", "/config/baseUrl"))?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
            || url.port() == Some(0)
        {
            return Err(invalid(
                "Use only an HTTPS origin, without credentials, path, query or fragment",
                "/config/baseUrl",
            ));
        }
        self.base_url = url.origin().ascii_serialization();
        if !bounded(&self.username, 128)
            || !bounded(&self.owner, 128)
            || !bounded(&self.access_key, 512)
        {
            return Err(invalid(
                "Panel identity fields are too long or contain control characters",
                "/config",
            ));
        }
        match self.provider {
            PanelProvider::Virtualmin
                if self.username.is_empty() || self.username.contains(':') =>
            {
                return Err(invalid(
                    "A Virtualmin login name without a colon is required",
                    "/config/username",
                ));
            }
            PanelProvider::Hestia
                if self.owner.is_empty()
                    || self.access_key.is_empty()
                    || self.access_key.contains(':') =>
            {
                return Err(invalid(
                    "A Hestia owner and public access key are required",
                    "/config/owner",
                ));
            }
            _ => {}
        }
        if let PanelConnection::Ssh { tunnel_id } = &self.connection
            && !identifier(tunnel_id)
        {
            return Err(invalid(
                "Select a saved SSH tunnel",
                "/config/connection/tunnelId",
            ));
        }
        if !matches!(self.tls.min_version.as_str(), "1.2" | "1.3") {
            return Err(invalid(
                "Minimum TLS must be 1.2 or 1.3",
                "/config/tls/minVersion",
            ));
        }
        if let Some(name) = &self.tls.server_name
            && (name.is_empty()
                || name.len() > 253
                || rustls::pki_types::ServerName::try_from(name.clone()).is_err())
        {
            return Err(invalid("Invalid TLS server name", "/config/tls/serverName"));
        }
        if let Some(pem) = &self.tls.ca_pem {
            use rustls::pki_types::pem::PemObject;
            if pem.is_empty() || pem.len() > 65536 || pem.contains("PRIVATE KEY") {
                return Err(invalid(
                    "Provide CA certificates only, at most 64 KiB",
                    "/config/tls/caPem",
                ));
            }
            let certs = rustls::pki_types::CertificateDer::pem_slice_iter(pem.as_bytes())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| invalid("Invalid CA certificate PEM", "/config/tls/caPem"))?;
            if certs.is_empty() {
                return Err(invalid("No CA certificate was found", "/config/tls/caPem"));
            }
            let mut roots = rustls::RootCertStore::empty();
            for cert in certs {
                roots
                    .add(cert)
                    .map_err(|_| invalid("Invalid CA certificate", "/config/tls/caPem"))?;
            }
        }
        Ok(())
    }
    fn authority(&self) -> Value {
        json!([
            self.provider,
            self.base_url,
            self.username,
            self.owner,
            self.access_key,
            self.connection,
            self.tls
        ])
    }
}
fn etag(row: &HostingConnectionRow) -> String {
    format!("\"{}\"", row.revision)
}
fn redacted(row: &HostingConnectionRow) -> Result<Value, AdminApiError> {
    let config: PanelConfig = serde_json::from_str(&row.config_json)
        .map_err(|_| AdminApiError::internal("Saved hosting configuration is invalid"))?;
    let mut value = serde_json::to_value(config)
        .map_err(|_| AdminApiError::internal("Hosting configuration encoding failed"))?;
    let obj = value
        .as_object_mut()
        .expect("PanelConfig serializes as object");
    obj.insert("credential".into(), json!({"set":row.credential_set}));
    obj.insert("etag".into(), json!(etag(row)));
    obj.insert("updatedAt".into(), json!(row.updated_at));
    Ok(value)
}
fn require_match(headers: &HeaderMap, row: &HostingConnectionRow) -> Result<(), AdminApiError> {
    let raw = headers.get(header::IF_MATCH).ok_or_else(|| {
        AdminApiError::new(
            StatusCode::PRECONDITION_REQUIRED,
            "preconditionRequired",
            "Reload this connection before changing it",
        )
    })?;
    if raw.to_str().ok() != Some(etag(row).as_str()) {
        return Err(AdminApiError::stale(None, Some(row.updated_at)));
    }
    Ok(())
}
async fn saved(state: &AppState, id: &str) -> Result<HostingConnectionRow, AdminApiError> {
    state
        .store
        .hosting_connection(id)
        .await
        .map_err(store_error)?
        .ok_or_else(|| {
            AdminApiError::new(
                StatusCode::NOT_FOUND,
                "notFound",
                "Hosting connection not found",
            )
        })
}
fn response(row: &HostingConnectionRow) -> Result<Response, AdminApiError> {
    Ok(with_etag(
        (
            [(header::CACHE_CONTROL, "private, no-store")],
            Json(redacted(row)?),
        ),
        &etag(row),
    ))
}
async fn list(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsRead)?;
    let rows = state
        .store
        .hosting_connections()
        .await
        .map_err(store_error)?;
    let entries = rows.iter().map(redacted).collect::<Result<Vec<_>, _>>()?;
    Ok((
        [(header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"connections":entries})),
    )
        .into_response())
}
async fn get(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsRead)?;
    response(&saved(&state, &id).await?)
}
#[derive(Default, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase", deny_unknown_fields)]
enum CredentialInput {
    #[default]
    Keep,
    Set {
        value: String,
    },
    Clear,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WriteInput {
    config: PanelConfig,
    #[serde(default)]
    credential: CredentialInput,
}
async fn write(
    state: &AppState,
    ctx: &AdminCtx,
    mut input: WriteInput,
    old: Option<HostingConnectionRow>,
) -> Result<HostingConnectionRow, AdminApiError> {
    input.config.validate()?;
    let previous_connection = old
        .as_ref()
        .map(|row| {
            serde_json::from_str::<PanelConfig>(&row.config_json)
                .map(|config| config.connection)
                .map_err(|_| AdminApiError::internal("Saved hosting configuration is invalid"))
        })
        .transpose()?;
    if previous_connection.as_ref() != Some(&input.config.connection)
        && (matches!(input.config.connection, PanelConnection::Ssh { .. })
            || matches!(previous_connection, Some(PanelConnection::Ssh { .. })))
    {
        ctx.require_unscoped(Capability::TunnelsBind)?;
    }
    let retaining = matches!(input.credential, CredentialInput::Keep)
        && old.as_ref().is_some_and(|row| row.credential_set);
    if retaining && let Some(old) = &old {
        let previous: PanelConfig = serde_json::from_str(&old.config_json)
            .map_err(|_| AdminApiError::internal("Saved hosting configuration is invalid"))?;
        if previous.authority() != input.config.authority() {
            return Err(invalid(
                "Replace or clear the credential when changing its destination, identity, TLS authority or SSH tunnel",
                "/credential",
            ));
        }
    }
    if input.config.enabled
        && !(retaining || matches!(input.credential, CredentialInput::Set { .. }))
    {
        return Err(invalid(
            "A configured credential is required before enabling the connection",
            "/credential",
        ));
    }
    let credential = match input.credential {
        CredentialInput::Keep => HostingCredentialChange::Keep,
        CredentialInput::Clear => HostingCredentialChange::Clear,
        CredentialInput::Set { value } => {
            if value.is_empty() || !bounded(&value, 4096) {
                return Err(invalid(
                    "Credential must contain 1 to 4096 bytes without control characters",
                    "/credential",
                ));
            }
            HostingCredentialChange::Set(value)
        }
    };
    let encoded = serde_json::to_string(&input.config)
        .map_err(|_| AdminApiError::internal("Hosting configuration encoding failed"))?;
    let row = match state
        .store
        .put_hosting_connection(
            &input.config.id,
            &encoded,
            old.as_ref().map(|row| row.revision.as_str()),
            credential,
            chrono::Utc::now().timestamp(),
        )
        .await
        .map_err(store_error)?
    {
        HostingMutation::Applied(row) => row,
        HostingMutation::Conflict => return Err(AdminApiError::stale(None, None)),
        HostingMutation::Missing => {
            return Err(AdminApiError::new(
                StatusCode::NOT_FOUND,
                "notFound",
                "Hosting connection not found",
            ));
        }
    };
    crate::admin_accounts::audit(
        state,
        ctx,
        if old.is_some() {
            "hosting-connection-updated"
        } else {
            "hosting-connection-created"
        },
        &format!("hosting:{}", row.id),
        None,
        Some(json!({"name":input.config.name,"provider":input.config.provider})),
    )
    .await;
    Ok(row)
}
async fn create(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(mut input): AdminJson<WriteInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsWrite)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    input.config.id = uuid::Uuid::new_v4().to_string();
    let row = write(&state, &ctx, input, None).await?;
    let mut result = response(&row)?;
    *result.status_mut() = StatusCode::CREATED;
    Ok(result)
}
async fn update(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    headers: HeaderMap,
    AdminJson(input): AdminJson<WriteInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsWrite)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let old = saved(&state, &id).await?;
    require_match(&headers, &old)?;
    if input.config.id != id {
        return Err(invalid(
            "Connection ID does not match the path",
            "/config/id",
        ));
    }
    response(&write(&state, &ctx, input, Some(old)).await?)
}
async fn delete(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsWrite)?;
    crate::step_up::require_admin(
        &state,
        &ctx,
        crate::step_up::StepUpAction::IntegrationManage,
    )
    .await?;
    let old = saved(&state, &id).await?;
    require_match(&headers, &old)?;
    if !state
        .store
        .delete_hosting_connection(&id, &old.revision)
        .await
        .map_err(store_error)?
    {
        return Err(AdminApiError::stale(None, None));
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "hosting-connection-deleted",
        &format!("hosting:{id}"),
        None,
        None,
    )
    .await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionInput {
    operation: crate::hosting_transport::PanelOperation,
    confirmation: Option<String>,
}
fn action_name(operation: &crate::hosting_transport::PanelOperation) -> &'static str {
    use crate::hosting_transport::PanelOperation::*;
    match operation {
        ListDomains => "list-domains",
        ListMailboxes { .. } => "list-mailboxes",
        CreateDomain { .. } => "create-domain",
        SetDomainEnabled { .. } => "set-domain-enabled",
        DeleteDomain { .. } => "delete-domain",
        CreateMailbox { .. } => "create-mailbox",
        SetMailboxQuota { .. } => "set-mailbox-quota",
        ResetMailboxPassword { .. } => "reset-mailbox-password",
        SetMailboxEnabled { .. } => "set-mailbox-enabled",
        DeleteMailbox { .. } => "delete-mailbox",
        AddAlias { .. } => "add-alias",
        RemoveAlias { .. } => "remove-alias",
    }
}
fn confirmation_target(
    operation: &crate::hosting_transport::PanelOperation,
    provider: PanelProvider,
) -> Option<String> {
    use crate::hosting_transport::PanelOperation::*;
    match operation {
        DeleteDomain { domain } => Some(domain.clone()),
        CreateDomain { domain, .. } | SetDomainEnabled { domain, .. }
            if provider == PanelProvider::Virtualmin =>
        {
            Some(domain.clone())
        }
        DeleteMailbox { domain, account } => Some(format!("{domain}/{account}")),
        RemoveAlias { domain, alias, .. } => Some(format!("{alias}@{domain}")),
        _ => None,
    }
}
async fn action(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    headers: HeaderMap,
    AdminJson(input): AdminJson<ActionInput>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::IntegrationsRead)?;
    if input.operation.is_mutation() {
        ctx.require_unscoped(Capability::IntegrationsWrite)?;
        crate::step_up::require_admin(
            &state,
            &ctx,
            crate::step_up::StepUpAction::IntegrationManage,
        )
        .await?;
    }
    let (row, secret) = state
        .store
        .hosting_connection_authority(&id)
        .await
        .map_err(store_error)?
        .ok_or_else(|| {
            AdminApiError::new(
                StatusCode::NOT_FOUND,
                "notFound",
                "Hosting connection not found",
            )
        })?;
    if input.operation.is_mutation() || headers.contains_key(header::IF_MATCH) {
        require_match(&headers, &row)?;
    }
    let mut config: PanelConfig = serde_json::from_str(&row.config_json)
        .map_err(|_| AdminApiError::internal("Saved hosting configuration is invalid"))?;
    config.validate()?;
    if !config.enabled {
        return Err(AdminApiError::conflict(
            "connection-disabled",
            "This hosting connection is disabled",
        ));
    }
    input.operation.validate(&config).map_err(|error| {
        AdminApiError::invalid(
            error.code,
            "The requested panel operation is not allowed for this connection",
            "/operation",
        )
    })?;
    if let Some(target) = confirmation_target(&input.operation, config.provider)
        && input.confirmation.as_deref() != Some(&target)
    {
        return Err(AdminApiError::invalid(
            "confirmationRequired",
            "Confirm the exact remote target before performing this operation",
            "/confirmation",
        ));
    }
    let secret = secret.ok_or_else(|| {
        AdminApiError::conflict(
            "credential-missing",
            "This hosting connection has no saved credential",
        )
    })?;
    if state
        .store
        .active_egress_proxy()
        .await
        .map_err(store_error)?
        .is_some()
    {
        return Err(AdminApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "unsupported-route",
            "The active proxy cannot carry this panel connection; direct fallback is refused",
        ));
    }
    let url = reqwest::Url::parse(&config.base_url)
        .map_err(|_| invalid("Invalid HTTPS origin", "/config/baseUrl"))?;
    let host = url
        .host_str()
        .expect("validated hostname")
        .trim_matches(['[', ']']);
    let port = url.port_or_known_default().expect("HTTPS has a port");
    let dialer = match &config.connection {
        PanelConnection::Direct => {
            crate::domain_transport::GuardedDialer::shared(host, port, config.allow_private)
        }
        PanelConnection::Ssh { tunnel_id } => {
            crate::ssh_tunnels::hosting_dialer(&state, tunnel_id, host, port, config.allow_private)
                .await?
        }
    };
    let result =
        crate::hosting_transport::perform(&config, &secret, &input.operation, dialer).await;
    if input.operation.is_mutation() {
        crate::admin_accounts::audit(&state,&ctx,&format!("hosting-{}",action_name(&input.operation)),&format!("hosting:{id}"),None,Some(json!({"domain":input.operation.domain(),"confirmed":result.is_ok(),"resultUncertain":result.as_ref().err().is_some_and(|error|error.result_uncertain)}))).await;
    }
    match result {
        Ok(result)=>Ok(([(header::CACHE_CONTROL,"private, no-store")],Json(result)).into_response()),
        Err(error)=>Err(AdminApiError::new(StatusCode::BAD_GATEWAY,error.code,if error.result_uncertain{"The panel request was dispatched, but its result is uncertain. Inspect the remote state before retrying."}else{"The panel request could not be completed. Check the saved endpoint, credential and transport policy."}).with("resultUncertain",error.result_uncertain)),
    }
}
