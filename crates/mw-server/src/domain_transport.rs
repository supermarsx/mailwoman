//! Operator-owned mail targets: all DNS answers are gated once, then sockets
//! connect only to those checked addresses. TLS remains the protocol's job.
use mw_net::{BoxFuture, BoxStream, Dialer, SharedDialer};
use std::{
    io,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::net::TcpStream;

#[derive(Debug, Clone, Copy)]
pub(crate) enum DialPhase {
    Dns,
    Connect,
}
#[derive(Debug)]
pub(crate) struct DialFailure {
    pub phase: DialPhase,
    source: io::Error,
}
impl std::fmt::Display for DialFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.phase {
            DialPhase::Dns => "DNS lookup failed",
            DialPhase::Connect => "upstream connection failed",
        })
    }
}
impl std::error::Error for DialFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
fn stage(phase: DialPhase, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        DialFailure {
            phase,
            source: error,
        },
    )
}

#[derive(Debug, Clone)]
pub struct GuardedDialer {
    host: String,
    port: u16,
    allow_private: bool,
}
impl GuardedDialer {
    pub fn new(host: impl Into<String>, port: u16, allow_private: bool) -> Self {
        Self {
            host: host.into(),
            port,
            allow_private,
        }
    }
    pub fn shared(host: impl Into<String>, port: u16, allow_private: bool) -> SharedDialer {
        Arc::new(Self::new(host, port, allow_private))
    }
}
/// The operator opt-in allows private LANs, including same-host loopback, never metadata/link-local.
pub(crate) fn address_allowed(ip: &IpAddr, allow_private: bool) -> bool {
    mw_egress::operator_mail_ip_allowed(ip, allow_private)
}

fn checked_addresses(
    addresses: Vec<SocketAddr>,
    allow_private: bool,
) -> io::Result<Vec<SocketAddr>> {
    if addresses.is_empty()
        || !addresses
            .iter()
            .all(|a| address_allowed(&a.ip(), allow_private))
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "upstream address policy refused the DNS answer set",
        ));
    }
    Ok(addresses)
}
impl Dialer for GuardedDialer {
    fn dial<'a>(&'a self, host: &'a str, port: u16) -> BoxFuture<'a, io::Result<BoxStream>> {
        Box::pin(async move {
            if !host.eq_ignore_ascii_case(&self.host) || port != self.port {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "dial target differs from the configured endpoint",
                ));
            }
            tokio::time::timeout(Duration::from_secs(5), async {
                let host = host.trim_matches(['[', ']']);
                let addresses = checked_addresses(
                    tokio::net::lookup_host((host, port))
                        .await
                        .map_err(|e| stage(DialPhase::Dns, e))?
                        .collect(),
                    self.allow_private,
                )?;
                let mut last =
                    io::Error::new(io::ErrorKind::NotConnected, "upstream connection failed");
                for addr in addresses {
                    match TcpStream::connect(addr).await {
                        Ok(stream) => {
                            stream.set_nodelay(true).ok();
                            return Ok(Box::new(stream) as BoxStream);
                        }
                        Err(error) => last = error,
                    }
                }
                Err(stage(DialPhase::Connect, last))
            })
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "upstream connection timed out"))?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_dns_answer_must_pass_and_private_needs_explicit_opt_in() {
        let addr = |ip: &str| SocketAddr::new(ip.parse().unwrap(), 993);
        assert!(checked_addresses(vec![addr("8.8.8.8"), addr("1.1.1.1")], false).is_ok());
        assert!(checked_addresses(vec![addr("8.8.8.8"), addr("10.0.0.1")], false).is_err());
        assert!(checked_addresses(vec![addr("8.8.8.8"), addr("10.0.0.1")], true).is_ok());
        assert!(checked_addresses(vec![], true).is_err());
        for ip in [
            "169.254.169.254",
            "fd00:ec2::254",
            "fe80::1",
            "0.0.0.0",
            "224.0.0.1",
            "::",
        ] {
            assert!(
                checked_addresses(vec![addr("8.8.8.8"), addr(ip)], true).is_err(),
                "{ip}"
            );
        }
    }
    #[tokio::test]
    async fn refuses_loopback_without_opening_a_socket_and_cannot_be_retargeted() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let dialer = GuardedDialer::new("127.0.0.1", port, false);
        assert!(dialer.dial("127.0.0.1", port).await.is_err());
        assert!(dialer.dial("8.8.8.8", port).await.is_err());
        assert!(
            tokio::time::timeout(Duration::from_millis(30), listener.accept())
                .await
                .is_err()
        );
        let opted_in = GuardedDialer::new("127.0.0.1", port, true);
        let _client = opted_in.dial("127.0.0.1", port).await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_secs(1), listener.accept())
                .await
                .is_ok()
        );
    }
}
