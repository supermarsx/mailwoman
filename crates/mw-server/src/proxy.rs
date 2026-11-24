//! Server socket adapter for the shared trusted-proxy model.
use axum::{
    extract::ConnectInfo,
    http::{Extensions, HeaderMap},
};
pub(crate) use mw_protection::proxy::*;
use std::net::{IpAddr, SocketAddr};

/// Resolve the client IP for a request: the peer address from `ConnectInfo`, refined
/// by a forwarded header only when the configuration permits it.
///
/// `None` means the serve path did not install `ConnectInfo` — there is then no
/// address any check may rely on, and callers treat that as "no source IP" rather
/// than falling back to a header.
pub(crate) fn client_ip(headers: &HeaderMap, ext: &Extensions) -> Option<IpAddr> {
    let peer = peer_ip(ext)?;
    let cfg = ProxyConfig::from_env();
    // Report-only (t56): warns when a private peer's forwarded header goes unused.
    crate::proxy_posture::observe(&cfg, peer, headers);
    Some(resolve(&cfg, peer, headers))
}

/// The address actually on the other end of the socket, canonicalised — no header is
/// consulted, and no proxy configuration can change it.
///
/// Use this, not [`client_ip`], for any check about **who is connecting** rather than
/// who the connection is on behalf of. Header authentication is the case in point: a
/// proxy trusted to report a client address is not thereby trusted to assert an
/// identity, so `MW_HEADER_AUTH_TRUSTED_IPS` must gate on the peer.
///
/// `None` means the serve path installed no `ConnectInfo`; a gate that requires an
/// address must refuse rather than proceed.
pub(crate) fn peer_ip(ext: &Extensions) -> Option<IpAddr> {
    Some(ext.get::<ConnectInfo<SocketAddr>>()?.0.ip().to_canonical())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }
    fn hdrs(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.append(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                v.parse().unwrap(),
            );
        }
        h
    }
    #[test]
    fn client_ip_requires_connect_info() {
        let h = hdrs(&[("x-forwarded-for", "8.8.8.8")]);
        // No ConnectInfo → no client IP, whatever the header says.
        assert!(client_ip(&h, &Extensions::new()).is_none());

        let mut ext = Extensions::new();
        ext.insert(ConnectInfo(
            "198.51.100.7:52000".parse::<SocketAddr>().unwrap(),
        ));
        // Default posture (no env set in this process): the peer address.
        assert_eq!(client_ip(&h, &ext).unwrap(), ip("198.51.100.7"));
    }
    #[test]
    fn peer_ip_reads_only_connect_info() {
        assert!(peer_ip(&Extensions::new()).is_none());

        let mut ext = Extensions::new();
        ext.insert(ConnectInfo(
            "198.51.100.7:52000".parse::<SocketAddr>().unwrap(),
        ));
        assert_eq!(peer_ip(&ext).unwrap(), ip("198.51.100.7"));

        // A mapped v4 peer is canonicalised, so a v4 allowlist matches it.
        let mut ext = Extensions::new();
        ext.insert(ConnectInfo(
            "[::ffff:10.1.2.3]:52000".parse::<SocketAddr>().unwrap(),
        ));
        assert_eq!(peer_ip(&ext).unwrap(), ip("10.1.2.3"));
    }
}
