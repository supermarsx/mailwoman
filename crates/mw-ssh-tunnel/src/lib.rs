//! SSH is a transport only: exact pinned destinations, verified jump host, no shell.
use mw_net::{BoxFuture, BoxStream, Dialer, SharedDialer};
use russh::{
    client,
    keys::{self, HashAlg, PrivateKey, PrivateKeyWithHashAlg, PublicKeyOrCertificate},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt, io,
    net::IpAddr,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::{Mutex, OwnedSemaphorePermit, Semaphore},
    time::timeout,
};
use zeroize::Zeroizing;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostKeyPin {
    pub algorithm: String,
    pub sha256: String,
}

pub enum Credential {
    PrivateKey(Arc<PrivateKey>),
    Password(Zeroizing<String>),
}
impl fmt::Debug for Credential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Credential([redacted])")
    }
}

#[derive(Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub pins: Vec<HostKeyPin>,
    pub credential: Credential,
    pub connect_timeout: Duration,
    pub keepalive: Duration,
    pub max_channels: usize,
}
struct PinHandler {
    pins: Vec<HostKeyPin>,
    rejected: Arc<AtomicBool>,
}
impl client::Handler for PinHandler {
    type Error = russh::Error;
    async fn check_server_key(
        &mut self,
        key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        let matches = match key {
            PublicKeyOrCertificate::PublicKey { key, .. } => {
                key.key_data()
                    .rsa()
                    .is_none_or(|rsa| rsa.key_size() >= 3072)
                    && self.pins.iter().any(|pin| {
                        pin.algorithm == key.algorithm().to_string()
                            && pin.sha256 == key.fingerprint(HashAlg::Sha256).to_string()
                    })
            }
            _ => false,
        };
        if !matches {
            self.rejected.store(true, Ordering::Relaxed);
        }
        Ok(matches)
    }
}
struct Session {
    handle: Option<client::Handle<PinHandler>>,
    failures: u32,
    retry_at: Instant,
}
pub struct Tunnel {
    config: Config,
    jump: SharedDialer,
    session: Mutex<Session>,
    channels: Arc<Semaphore>,
}
impl fmt::Debug for Tunnel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tunnel")
            .field("host", &self.config.host)
            .field("port", &self.config.port)
            .finish_non_exhaustive()
    }
}
fn error(kind: io::ErrorKind, code: &'static str) -> io::Error {
    io::Error::new(kind, code)
}
impl Tunnel {
    pub fn new(config: Config, jump: SharedDialer) -> io::Result<Arc<Self>> {
        if config.pins.is_empty()
            || config.host.is_empty()
            || config.username.is_empty()
            || config.port == 0
            || !(1..=128).contains(&config.max_channels)
            || config.connect_timeout.is_zero()
            || config.connect_timeout > Duration::from_secs(60)
        {
            return Err(error(io::ErrorKind::InvalidInput, "invalid_tunnel"));
        }
        let channels = Arc::new(Semaphore::new(config.max_channels));
        Ok(Arc::new(Self {
            config,
            jump,
            session: Mutex::new(Session {
                handle: None,
                failures: 0,
                retry_at: Instant::now(),
            }),
            channels,
        }))
    }
    async fn connect(&self) -> io::Result<client::Handle<PinHandler>> {
        let rejected = Arc::new(AtomicBool::new(false));
        let transport = self
            .jump
            .dial(&self.config.host, self.config.port)
            .await
            .map_err(|_| error(io::ErrorKind::ConnectionRefused, "jump_connect_failed"))?;
        let cfg = Arc::new(client::Config {
            keepalive_interval: Some(self.config.keepalive),
            keepalive_max: 3,
            preferred: russh::Preferred {
                key: std::borrow::Cow::Owned(
                    self.config
                        .pins
                        .iter()
                        .flat_map(|pin| match pin.algorithm.as_str() {
                            "ssh-ed25519" => vec![keys::Algorithm::Ed25519],
                            "ecdsa-sha2-nistp256" => vec![keys::Algorithm::Ecdsa {
                                curve: keys::EcdsaCurve::NistP256,
                            }],
                            "ecdsa-sha2-nistp384" => vec![keys::Algorithm::Ecdsa {
                                curve: keys::EcdsaCurve::NistP384,
                            }],
                            "ssh-rsa" => vec![
                                keys::Algorithm::Rsa {
                                    hash: Some(HashAlg::Sha512),
                                },
                                keys::Algorithm::Rsa {
                                    hash: Some(HashAlg::Sha256),
                                },
                            ],
                            _ => Vec::new(),
                        })
                        .collect(),
                ),
                ..Default::default()
            },
            ..Default::default()
        });
        let mut handle = client::connect_stream(
            cfg,
            transport,
            PinHandler {
                pins: self.config.pins.clone(),
                rejected: rejected.clone(),
            },
        )
        .await
        .map_err(|_| {
            error(
                io::ErrorKind::PermissionDenied,
                if rejected.load(Ordering::Relaxed) {
                    "host_key_mismatch"
                } else {
                    "ssh_handshake_failed"
                },
            )
        })?;
        let accepted = match &self.config.credential {
            Credential::Password(password) => {
                handle
                    .authenticate_password(&self.config.username, password.as_str())
                    .await
            }
            Credential::PrivateKey(key) => {
                // Refuse SHA-1 RSA fallback; SHA-2 authentication is mandatory.
                let hash = if key.algorithm().is_rsa() {
                    Some(
                        handle
                            .best_supported_rsa_hash()
                            .await
                            .map_err(|_| {
                                error(io::ErrorKind::PermissionDenied, "rsa_sha2_unavailable")
                            })?
                            .flatten()
                            .ok_or_else(|| {
                                error(io::ErrorKind::PermissionDenied, "rsa_sha2_unavailable")
                            })?,
                    )
                } else {
                    None
                };
                handle
                    .authenticate_publickey(
                        &self.config.username,
                        PrivateKeyWithHashAlg::new(key.clone(), hash),
                    )
                    .await
            }
        }
        .map_err(|_| error(io::ErrorKind::PermissionDenied, "ssh_auth_failed"))?;
        if !accepted.success() {
            return Err(error(io::ErrorKind::PermissionDenied, "ssh_auth_failed"));
        }
        Ok(handle)
    }
    async fn open(&self, address: IpAddr, port: u16) -> io::Result<BoxStream> {
        let deadline = self.config.connect_timeout;
        let permit = timeout(deadline, self.channels.clone().acquire_owned())
            .await
            .map_err(|_| error(io::ErrorKind::TimedOut, "channel_limit"))?
            .map_err(|_| error(io::ErrorKind::BrokenPipe, "tunnel_closed"))?;
        let mut session = timeout(deadline, self.session.lock())
            .await
            .map_err(|_| error(io::ErrorKind::TimedOut, "tunnel_busy"))?;
        if session.handle.as_ref().is_none_or(|h| h.is_closed()) {
            if Instant::now() < session.retry_at {
                return Err(error(io::ErrorKind::WouldBlock, "reconnect_backoff"));
            }
            session.handle = None;
            match timeout(deadline, self.connect()).await {
                Ok(Ok(handle)) => {
                    session.handle = Some(handle);
                    session.failures = 0;
                }
                result => {
                    session.failures = session.failures.saturating_add(1);
                    session.retry_at = Instant::now()
                        + Duration::from_secs(
                            (1u64 << session.failures.saturating_sub(1).min(6)).min(60),
                        );
                    return Err(match result {
                        Ok(Err(e)) => e,
                        _ => error(io::ErrorKind::TimedOut, "ssh_connect_timeout"),
                    });
                }
            }
        }
        let channel = timeout(
            deadline,
            session
                .handle
                .as_ref()
                .expect("connected above")
                .channel_open_direct_tcpip(address.to_string(), u32::from(port), "127.0.0.1", 0),
        )
        .await
        .map_err(|_| error(io::ErrorKind::TimedOut, "channel_timeout"))?
        .map_err(|_| error(io::ErrorKind::ConnectionRefused, "channel_refused"))?;
        Ok(Box::new(PooledStream {
            inner: Box::new(channel.into_stream()),
            _permit: permit,
        }))
    }
}

/// Caller resolves and validates all addresses once; SSH receives numeric IPs only.
#[derive(Clone, Debug)]
pub struct PinnedTarget {
    pub host: String,
    pub port: u16,
    pub address: IpAddr,
}
#[derive(Debug)]
pub struct SshDialer {
    tunnel: Arc<Tunnel>,
    targets: BTreeMap<(String, u16), IpAddr>,
}
impl SshDialer {
    pub fn new(tunnel: Arc<Tunnel>, targets: Vec<PinnedTarget>) -> io::Result<Self> {
        if targets.is_empty() || targets.len() > 256 {
            return Err(error(io::ErrorKind::InvalidInput, "invalid_targets"));
        }
        let mut mapping = BTreeMap::new();
        for t in targets {
            if t.host.is_empty()
                || t.port == 0
                || mapping.insert((t.host, t.port), t.address).is_some()
            {
                return Err(error(io::ErrorKind::InvalidInput, "ambiguous_target"));
            }
        }
        Ok(Self {
            tunnel,
            targets: mapping,
        })
    }
}
impl Dialer for SshDialer {
    fn dial<'a>(&'a self, host: &'a str, port: u16) -> BoxFuture<'a, io::Result<BoxStream>> {
        Box::pin(async move {
            let address = self
                .targets
                .get(&(host.to_owned(), port))
                .ok_or_else(|| error(io::ErrorKind::PermissionDenied, "target_not_allowed"))?;
            self.tunnel.open(*address, port).await
        })
    }
}
struct PooledStream {
    inner: BoxStream,
    _permit: OwnedSemaphorePermit,
}
impl AsyncRead for PooledStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}
impl AsyncWrite for PooledStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

pub fn import_private_key(
    pem: &str,
    passphrase: Option<&str>,
) -> io::Result<(Credential, String, String)> {
    let key = keys::decode_secret_key(pem, passphrase)
        .map_err(|_| error(io::ErrorKind::InvalidInput, "invalid_private_key"))?;
    match key.algorithm() {
        keys::Algorithm::Ed25519
        | keys::Algorithm::Ecdsa {
            curve: keys::EcdsaCurve::NistP256 | keys::EcdsaCurve::NistP384,
        }
        | keys::Algorithm::Rsa { .. } => {}
        _ => {
            return Err(error(
                io::ErrorKind::InvalidInput,
                "unsupported_key_algorithm",
            ));
        }
    }
    if key
        .public_key()
        .key_data()
        .rsa()
        .is_some_and(|rsa| rsa.key_size() < 3072)
    {
        return Err(error(io::ErrorKind::InvalidInput, "rsa_key_too_small"));
    }
    let public = key
        .public_key()
        .to_openssh()
        .map_err(|_| error(io::ErrorKind::InvalidInput, "invalid_private_key"))?;
    let algorithm = key.algorithm().to_string();
    Ok((Credential::PrivateKey(Arc::new(key)), public, algorithm))
}
/// Remove an import passphrase before sealing; never store that passphrase.
pub fn normalized_private_key(credential: &Credential) -> io::Result<Zeroizing<String>> {
    match credential {
        Credential::PrivateKey(key) => key
            .to_openssh(Default::default())
            .map_err(|_| error(io::ErrorKind::InvalidInput, "invalid_private_key")),
        Credential::Password(_) => Err(error(io::ErrorKind::InvalidInput, "not_a_private_key")),
    }
}
pub fn generate_ed25519() -> io::Result<String> {
    PrivateKey::random(&mut rand10::rng(), keys::Algorithm::Ed25519)
        .and_then(|key| key.to_openssh(Default::default()))
        .map(|s| s.to_string())
        .map_err(|_| error(io::ErrorKind::Other, "key_generation_failed"))
}

struct ObserveKey(Arc<std::sync::Mutex<Option<HostKeyPin>>>);
impl client::Handler for ObserveKey {
    type Error = russh::Error;
    async fn check_server_key(
        &mut self,
        key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        if let PublicKeyOrCertificate::PublicKey { key, .. } = key {
            *self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(HostKeyPin {
                algorithm: key.algorithm().to_string(),
                sha256: key.fingerprint(HashAlg::Sha256).to_string(),
            });
        }
        // Deliberately stop before authentication. Observation is never trust.
        Ok(false)
    }
}
pub async fn observe_host_key(
    jump: &dyn Dialer,
    host: &str,
    port: u16,
    deadline: Duration,
) -> io::Result<HostKeyPin> {
    timeout(deadline, async {
        let observed = Arc::new(std::sync::Mutex::new(None));
        let stream = jump
            .dial(host, port)
            .await
            .map_err(|_| error(io::ErrorKind::ConnectionRefused, "jump_connect_failed"))?;
        let _ = client::connect_stream(
            Arc::new(client::Config::default()),
            stream,
            ObserveKey(observed.clone()),
        )
        .await;
        let result = observed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        result.ok_or_else(|| error(io::ErrorKind::InvalidData, "host_key_unavailable"))
    })
    .await
    .map_err(|_| error(io::ErrorKind::TimedOut, "ssh_connect_timeout"))?
}
