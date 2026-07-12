//! rustls client configuration for the ManageSieve transport (ring provider +
//! webpki-roots), built with an explicit `ring` provider so the crate does not
//! depend on a process-global default being installed elsewhere. Mirrors the
//! `mw-imap`/`mw-smtp` setup (plan §1.1). Not exercised by the plaintext mock
//! transcript tests.

use std::sync::Arc;

use rustls::ClientConfig;
use tokio_rustls::TlsConnector;

use crate::SieveError;

/// Build a [`TlsConnector`] trusting the Mozilla webpki root set.
pub(crate) fn connector() -> crate::Result<TlsConnector> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| SieveError::ManageSieve(format!("rustls protocol setup: {e}")))?
        .with_root_certificates(roots)
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(config)))
}
