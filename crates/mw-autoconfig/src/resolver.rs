//! Live DNS SRV resolver behind the [`Fetcher::srv`] seam (plan §2.4, SPEC §6.3).
//!
//! RFC 6186 discovery needs real SRV lookups. The DNS I/O sits behind the
//! [`SrvResolver`] trait so the record mapping and the ladder above it stay
//! unit-testable against a stub with no live network. [`HickoryResolver`] is the
//! live implementation over `hickory-resolver` (pure-Rust DNS over UDP/TCP with
//! `rustls` — no `-sys`/C/openssl, within the license floor).

use async_trait::async_trait;

use crate::{DiscoverError, SrvRecord};

/// The DNS SRV lookup primitive. Injecting it lets the discovery ladder run
/// against a stub resolver with no live network (see the crate tests).
#[async_trait]
pub trait SrvResolver: Send + Sync {
    /// Resolve SRV records for a service name (e.g. `_imaps._tcp.example.org`).
    /// Returns an empty vec when the record is absent (NXDOMAIN / no records) so
    /// the ladder simply moves on to the next rung.
    async fn lookup_srv(&self, service: &str) -> Result<Vec<SrvRecord>, DiscoverError>;
}

/// A resolver that never returns records. Used as the fallback when the live
/// resolver cannot be constructed, so discovery still proceeds via the HTTP
/// rungs instead of hard-failing.
pub(crate) struct NoopResolver;

#[async_trait]
impl SrvResolver for NoopResolver {
    async fn lookup_srv(&self, _service: &str) -> Result<Vec<SrvRecord>, DiscoverError> {
        Ok(Vec::new())
    }
}

/// Live SRV resolver over `hickory-resolver`, built from the host's system DNS
/// configuration.
pub struct HickoryResolver {
    inner: hickory_resolver::TokioResolver,
}

impl HickoryResolver {
    /// Build a resolver from the host's system DNS configuration
    /// (`/etc/resolv.conf` on Unix, the network registry on Windows).
    pub fn new() -> Result<Self, DiscoverError> {
        let inner = hickory_resolver::TokioResolver::builder_tokio()
            .map_err(|e| DiscoverError::Lookup(e.to_string()))?
            .build()
            .map_err(|e| DiscoverError::Lookup(e.to_string()))?;
        Ok(Self { inner })
    }
}

#[async_trait]
impl SrvResolver for HickoryResolver {
    async fn lookup_srv(&self, service: &str) -> Result<Vec<SrvRecord>, DiscoverError> {
        use hickory_resolver::proto::rr::RData;
        // A trailing dot makes it a fully-qualified name (skips the search list).
        let fqdn = if service.ends_with('.') {
            service.to_string()
        } else {
            format!("{service}.")
        };
        match self.inner.srv_lookup(fqdn).await {
            Ok(lookup) => Ok(lookup
                .answers()
                .iter()
                .filter_map(|rec| match &rec.data {
                    RData::SRV(srv) => Some(SrvRecord {
                        target: srv.target.to_utf8(),
                        port: srv.port,
                        priority: srv.priority,
                        weight: srv.weight,
                    }),
                    _ => None,
                })
                .collect()),
            // Absent record: an empty result is "rung missed", not an error.
            Err(e) if e.is_no_records_found() => Ok(Vec::new()),
            Err(e) => Err(DiscoverError::Lookup(e.to_string())),
        }
    }
}

/// A fixed-response resolver used to exercise the SRV seam without live network.
#[cfg(test)]
pub(crate) struct StubResolver(pub Vec<SrvRecord>);

#[cfg(test)]
#[async_trait]
impl SrvResolver for StubResolver {
    async fn lookup_srv(&self, _service: &str) -> Result<Vec<SrvRecord>, DiscoverError> {
        Ok(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_resolver_returns_empty() {
        let recs = NoopResolver
            .lookup_srv("_imaps._tcp.example.org")
            .await
            .unwrap();
        assert!(recs.is_empty());
    }

    #[tokio::test]
    async fn stub_resolver_returns_fixture_records() {
        let stub = StubResolver(vec![SrvRecord {
            target: "imap.example.org.".into(),
            port: 993,
            priority: 10,
            weight: 5,
        }]);
        let recs = stub.lookup_srv("_imaps._tcp.example.org").await.unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].target, "imap.example.org.");
        assert_eq!(recs[0].port, 993);
    }
}
