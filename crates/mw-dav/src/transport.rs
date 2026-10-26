//! Injectable DAV transport. Managed accounts supply an implementation that
//! revalidates authority and credentials for every operation, including writes.

use std::{future::Future, pin::Pin};

use crate::{DavError, Result};

/// Maximum document size accepted by the DAV parsers and transport.
pub const MAX_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;

/// A DAV operation without credentials or arbitrary request headers.
/// Credentials belong to the transport's current account authority.
#[derive(Clone)]
pub struct TransportRequest {
    /// Application data family, used by account policy guards.
    pub kind: Option<&'static str>,
    pub method: String,
    pub url: String,
    pub depth: Option<String>,
    pub content_type: Option<String>,
    pub body: Option<String>,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
}

impl TransportRequest {
    /// Validate the bounded protocol surface before any network operation.
    pub fn validate(&self) -> Result<()> {
        let url = reqwest::Url::parse(&self.url)
            .map_err(|_| DavError::Transport("invalid DAV URL".into()))?;
        if self.url.len() > 8192
            || !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(DavError::Transport("invalid DAV URL".into()));
        }
        if !matches!(
            self.method.as_str(),
            "GET" | "PROPFIND" | "REPORT" | "PUT" | "DELETE" | "MKCALENDAR" | "POST"
        ) {
            return Err(DavError::Transport("unsupported DAV method".into()));
        }
        if self
            .body
            .as_ref()
            .is_some_and(|b| b.len() > MAX_DOCUMENT_BYTES)
        {
            return Err(DavError::Transport(
                "DAV document exceeds size limit".into(),
            ));
        }
        if self
            .depth
            .as_deref()
            .is_some_and(|d| !matches!(d, "0" | "1"))
        {
            return Err(DavError::Transport("unsupported DAV depth".into()));
        }
        for value in [&self.content_type, &self.if_match, &self.if_none_match]
            .into_iter()
            .flatten()
        {
            if value.len() > 4096 || value.bytes().any(|b| b < 0x20 || b == 0x7f) {
                return Err(DavError::Transport("invalid DAV header".into()));
            }
        }
        Ok(())
    }
}

/// Preserve status and entity tags so callers can distinguish conflicts from
/// successful mutations. Error bodies must not contain transport credentials.
pub struct TransportResponse {
    pub status: u16,
    pub etag: Option<String>,
    pub body: String,
}

pub type TransportFuture<'a> = Pin<Box<dyn Future<Output = Result<TransportResponse>> + Send + 'a>>;

/// An account-scoped transport; failure never permits a direct-network retry.
pub trait DavTransport: Send + Sync {
    fn send(&self, request: TransportRequest) -> TransportFuture<'_>;
}

/// Existing standalone DAV transport. Managed accounts replace this entirely;
/// they never fall back to it after a policy or upstream failure.
pub struct DirectTransport {
    client: reqwest::Client,
    config: crate::DavConfig,
}

impl DirectTransport {
    pub fn new(config: crate::DavConfig) -> Result<Self> {
        Ok(Self {
            client: crate::http_client(&config)?,
            config,
        })
    }
}

impl DavTransport for DirectTransport {
    fn send(&self, request: TransportRequest) -> TransportFuture<'_> {
        Box::pin(async move {
            request.validate()?;
            if !crate::url_allowed(&self.config, &request.url) {
                return Err(DavError::Transport(
                    "refused: the DAV server named a URL on another host".into(),
                ));
            }
            let method = reqwest::Method::from_bytes(request.method.as_bytes())
                .map_err(|_| DavError::Transport("invalid DAV method".into()))?;
            let mut pending = self
                .client
                .request(method, &request.url)
                .timeout(std::time::Duration::from_secs(15))
                .basic_auth(&self.config.username, Some(&self.config.password));
            for (name, value) in [
                ("Depth", request.depth),
                ("Content-Type", request.content_type),
                ("If-Match", request.if_match),
                ("If-None-Match", request.if_none_match),
            ] {
                if let Some(value) = value {
                    pending = pending.header(name, value);
                }
            }
            if let Some(body) = request.body {
                pending = pending.body(body);
            }
            let mut response = pending
                .send()
                .await
                .map_err(|_| DavError::Transport("DAV request failed".into()))?;
            let status = response.status().as_u16();
            let etag = response
                .headers()
                .get(reqwest::header::ETAG)
                .and_then(|value| value.to_str().ok())
                .filter(|value| value.len() <= 4096)
                .map(str::to_owned);
            if response
                .content_length()
                .is_some_and(|size| size > MAX_DOCUMENT_BYTES as u64)
            {
                return Err(DavError::Transport(
                    "DAV response exceeds size limit".into(),
                ));
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|_| DavError::Transport("DAV response failed".into()))?
            {
                if chunk.len() > MAX_DOCUMENT_BYTES.saturating_sub(bytes.len()) {
                    return Err(DavError::Transport(
                        "DAV response exceeds size limit".into(),
                    ));
                }
                bytes.extend_from_slice(&chunk);
            }
            let body = String::from_utf8(bytes)
                .map_err(|_| DavError::Transport("DAV response is not UTF-8".into()))?;
            Ok(TransportResponse { status, etag, body })
        })
    }
}
