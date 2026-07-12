//! rustls client configuration (ring provider + webpki-roots).
//!
//! Built with an explicit `ring` [`CryptoProvider`] rather than relying on a
//! process-global default, so the crate is self-contained and does not depend
//! on install order elsewhere in the workspace.

use std::sync::Arc;

use rustls::ClientConfig;
use tokio_rustls::TlsConnector;

use crate::error::{ImapError, ImapResult};

/// Build a [`TlsConnector`] trusting the Mozilla webpki root set.
pub(crate) fn connector() -> ImapResult<TlsConnector> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| ImapError::Tls(format!("rustls protocol setup: {e}")))?
        .with_root_certificates(roots)
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(config)))
}
