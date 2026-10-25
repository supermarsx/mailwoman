//! Pinned DNS policy around the reusable SSH transport. No direct fallback.
use super::{Config, normalize_host};
use crate::{AppState, admin_ctx::AdminApiError};
use mw_net::{BoxFuture, BoxStream, Dialer, SharedDialer};
use mw_ssh_tunnel::{Credential, PinnedTarget, SshDialer, Tunnel};
use std::{
    collections::BTreeMap,
    fmt, io,
    net::{IpAddr, SocketAddr},
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::sync::Mutex;
use zeroize::Zeroizing;

type TunnelPool = BTreeMap<(String, String), Arc<Tunnel>>;
static POOL: LazyLock<Mutex<TunnelPool>> = LazyLock::new(Default::default);
pub(super) async fn invalidate(id: &str) {
    POOL.lock().await.retain(|(saved_id, _), _| saved_id != id);
}
fn denied(code: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, code)
}
async fn resolve(host: &str, port: u16, allow_private: bool) -> io::Result<Vec<SocketAddr>> {
    let addresses = if let Ok(ip) = host.parse::<IpAddr>() {
        vec![SocketAddr::new(ip, port)]
    } else {
        tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| denied("dns_failed"))?
            .collect::<Vec<_>>()
    };
    if addresses.is_empty()
        || addresses.len() > 32
        || addresses
            .iter()
            .any(|a| !mw_egress::operator_mail_ip_allowed(&a.ip(), allow_private))
    {
        return Err(denied("address_not_allowed"));
    }
    let mut addresses = addresses;
    addresses.sort();
    addresses.dedup();
    Ok(addresses)
}
#[derive(Debug)]
struct JumpDialer {
    host: String,
    port: u16,
    allow_private: bool,
    deadline: Duration,
}
impl Dialer for JumpDialer {
    fn dial<'a>(&'a self, host: &'a str, port: u16) -> BoxFuture<'a, io::Result<BoxStream>> {
        Box::pin(async move {
            if host != self.host || port != self.port {
                return Err(denied("jump_target_not_allowed"));
            }
            tokio::time::timeout(self.deadline, async {
                let addresses = resolve(host, port, self.allow_private).await?;
                for address in addresses {
                    if let Ok(stream) = tokio::net::TcpStream::connect(address).await {
                        stream.set_nodelay(true).ok();
                        return Ok(Box::new(stream) as BoxStream);
                    }
                }
                Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "jump_connect_failed",
                ))
            })
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "jump_connect_timeout"))?
        })
    }
}
pub(super) async fn jump_dialer(
    state: &AppState,
    config: &Config,
) -> Result<SharedDialer, AdminApiError> {
    if state.store.active_egress_proxy().await?.is_some() {
        return Err(AdminApiError::conflict(
            "sshProxyUnsupported",
            "SSH through an active global proxy is not configured; no direct fallback is allowed",
        ));
    }
    let host = normalize_host(&config.host)?;
    Ok(Arc::new(JumpDialer {
        host,
        port: config.port,
        allow_private: config.allow_private,
        deadline: Duration::from_millis(u64::from(config.connect_timeout_ms.clamp(100, 60000))),
    }))
}
struct GuardedDialer {
    store: mw_store::Store,
    id: String,
    revision: String,
    inner: SshDialer,
}
impl fmt::Debug for GuardedDialer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GuardedSshDialer")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}
impl Dialer for GuardedDialer {
    fn dial<'a>(&'a self, host: &'a str, port: u16) -> BoxFuture<'a, io::Result<BoxStream>> {
        Box::pin(async move {
            let current = self
                .store
                .ssh_tunnel(&self.id)
                .await
                .map_err(|_| denied("tunnel_unavailable"))?
                .ok_or_else(|| denied("tunnel_unavailable"))?;
            if !current.enabled
                || !current.credential_configured
                || current.revision != self.revision
            {
                return Err(denied("tunnel_authority_changed"));
            }
            if self
                .store
                .active_egress_proxy()
                .await
                .map_err(|_| denied("egress_unavailable"))?
                .is_some()
            {
                return Err(denied("ssh_proxy_unsupported"));
            }
            let canonical = normalize_host(host).map_err(|_| denied("target_not_allowed"))?;
            let result = self.inner.dial(&canonical, port).await;
            if result
                .as_ref()
                .is_err_and(|error| safe_code(error) == "host_key_mismatch")
            {
                let audit = mw_store::AuditRow {
                    id: uuid::Uuid::new_v4().to_string(),
                    ts: chrono::Utc::now().to_rfc3339(),
                    actor: "ssh-transport".into(),
                    actor_kind: "system".into(),
                    action: "TunnelHostKeyRejected".into(),
                    target: Some(format!("tunnel:{}", self.id)),
                    detail_json: "{}".into(),
                    ip: None,
                };
                if self
                    .store
                    .append_categorized_audit(&audit, "security", None)
                    .await
                    .is_err()
                {
                    tracing::warn!("SSH host-key rejection audit could not be persisted");
                }
            }
            result
        })
    }
}
pub(crate) async fn hosting_dialer(
    state: &AppState,
    id: &str,
    host: &str,
    port: u16,
    allow_private: bool,
) -> Result<SharedDialer, AdminApiError> {
    let (row, secret) = state
        .store
        .ssh_tunnel_authority(id)
        .await?
        .ok_or_else(|| AdminApiError::conflict("tunnelUnavailable", "SSH tunnel is missing"))?;
    let mut config = Config::from_row(&row)?;
    config.validate()?;
    if !row.enabled {
        return Err(AdminApiError::conflict(
            "tunnelDisabled",
            "SSH tunnel is disabled",
        ));
    }
    let host = normalize_host(host)?;
    let target = config
        .targets
        .iter()
        .find(|target| target.host == host && target.port == port)
        .ok_or_else(|| {
            AdminApiError::conflict(
                "targetNotAllowed",
                "This exact host and port are not allowed by the selected tunnel",
            )
        })?;
    let deadline = Duration::from_millis(u64::from(config.connect_timeout_ms));
    let addresses = tokio::time::timeout(
        deadline,
        resolve(&host, port, allow_private && target.allow_private),
    )
    .await
    .map_err(|_| AdminApiError::conflict("targetDnsTimeout", "Target resolution timed out"))?
    .map_err(|_| {
        AdminApiError::conflict(
            "targetNotAllowed",
            "The target resolved to a forbidden address",
        )
    })?;
    let jump = jump_dialer(state, &config).await?;
    let key = (row.id.clone(), row.revision.clone());
    let mut pool = POOL.lock().await;
    let tunnel = if let Some(tunnel) = pool.get(&key) {
        tunnel.clone()
    } else {
        let secret = Zeroizing::new(secret.ok_or_else(|| {
            AdminApiError::conflict("credentialMissing", "SSH credential is not configured")
        })?);
        let credential = match row.auth_kind.as_deref() {
            Some("password") => Credential::Password(secret),
            Some("privateKey") => {
                mw_ssh_tunnel::import_private_key(&secret, None)
                    .map_err(|_| {
                        AdminApiError::conflict(
                            "credentialInvalid",
                            "Saved SSH credential is invalid",
                        )
                    })?
                    .0
            }
            _ => {
                return Err(AdminApiError::conflict(
                    "credentialInvalid",
                    "Saved SSH credential kind is unsupported",
                ));
            }
        };
        let tunnel = Tunnel::new(
            mw_ssh_tunnel::Config {
                host: config.host.clone(),
                port: config.port,
                username: config.username.clone(),
                pins: config.host_key_pins.clone(),
                credential,
                connect_timeout: deadline,
                keepalive: Duration::from_secs(u64::from(config.keepalive_seconds)),
                max_channels: config.max_channels as usize,
            },
            jump,
        )
        .map_err(|_| AdminApiError::conflict("tunnelInvalid", "Saved SSH tunnel is invalid"))?;
        pool.retain(|(saved_id, _), _| saved_id != id);
        if pool.len() >= 128 {
            pool.pop_first();
        }
        pool.insert(key, tunnel.clone());
        tunnel
    };
    drop(pool);
    let inner = SshDialer::new(
        tunnel,
        vec![PinnedTarget {
            host,
            port,
            address: addresses[0].ip(),
        }],
    )
    .map_err(|_| AdminApiError::conflict("targetNotAllowed", "Invalid SSH target"))?;
    Ok(Arc::new(GuardedDialer {
        store: state.store.clone(),
        id: row.id,
        revision: row.revision,
        inner,
    }))
}
pub(super) fn safe_code(error: &io::Error) -> &'static str {
    match error.to_string().as_str() {
        "host_key_mismatch" => "host_key_mismatch",
        "ssh_auth_failed" => "ssh_auth_failed",
        "rsa_sha2_unavailable" => "rsa_sha2_unavailable",
        "channel_limit" => "channel_limit",
        "channel_timeout" => "channel_timeout",
        "channel_refused" => "channel_refused",
        "reconnect_backoff" => "reconnect_backoff",
        "target_not_allowed" => "target_not_allowed",
        "tunnel_authority_changed" => "tunnel_authority_changed",
        "ssh_proxy_unsupported" => "ssh_proxy_unsupported",
        _ => "ssh_connection_failed",
    }
}
