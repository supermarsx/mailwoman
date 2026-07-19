use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;

use crate::types::{Request, Response, Session};

#[derive(Debug, thiserror::Error)]
pub enum JmapError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("upstream returned status {0}")]
    Status(reqwest::StatusCode),
    #[error("invalid JMAP url: {0}")]
    InvalidUrl(String),
}

/// Thin async JMAP client with HTTP Basic auth (upstream servers; the
/// browser never sees these credentials — SPEC §4.1).
#[derive(Debug, Clone)]
pub struct JmapClient {
    http: reqwest::Client,
    authorization: String,
}

/// Normalize a user-supplied server URL to its JMAP session endpoint.
pub fn session_url(input: &str) -> Result<String, JmapError> {
    let trimmed = input.trim().trim_end_matches('/');
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(JmapError::InvalidUrl(input.to_string()));
    }
    if trimmed.contains("/.well-known/jmap") || trimmed.ends_with("/jmap/session") {
        Ok(trimmed.to_string())
    } else {
        Ok(format!("{trimmed}/.well-known/jmap"))
    }
}

impl JmapClient {
    pub fn new(username: &str, password: &str) -> Result<Self, JmapError> {
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(3))
            .build()?;
        let authorization = format!("Basic {}", B64.encode(format!("{username}:{password}")));
        Ok(Self {
            http,
            authorization,
        })
    }

    pub fn authorization(&self) -> &str {
        &self.authorization
    }

    /// Fetch the JMAP Session resource (validates credentials).
    pub async fn session(&self, server_url: &str) -> Result<Session, JmapError> {
        let url = session_url(server_url)?;
        let resp = self
            .http
            .get(&url)
            .header("Authorization", &self.authorization)
            .header("Accept", "application/json")
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(JmapError::Status(resp.status()));
        }
        Ok(resp.json::<Session>().await?)
    }

    /// Execute a JMAP API request against the session's `apiUrl`.
    pub async fn request(&self, api_url: &str, request: &Request) -> Result<Response, JmapError> {
        let resp = self
            .http
            .post(api_url)
            .header("Authorization", &self.authorization)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(JmapError::Status(resp.status()));
        }
        Ok(resp.json::<Response>().await?)
    }

    /// Forward a raw JMAP request body, returning `(status, body)` verbatim.
    /// Used by the proxy so unknown methods/extensions pass through untouched.
    pub async fn request_raw(
        &self,
        api_url: &str,
        body: bytes::Bytes,
    ) -> Result<(reqwest::StatusCode, bytes::Bytes), JmapError> {
        let resp = self
            .http
            .post(api_url)
            .header("Authorization", &self.authorization)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        Ok((status, bytes))
    }

    /// POST raw bytes to an upload URL with injected auth and a caller-supplied
    /// `Content-Type`, returning `(status, content_type, body)` so the proxy can
    /// relay the upstream `{accountId, blobId, type, size}` upload response back
    /// to the browser verbatim (RFC 8620 §6.1) — the symmetric counterpart of
    /// [`JmapClient::get_bytes`].
    pub async fn post_bytes(
        &self,
        url: &str,
        content_type: &str,
        body: bytes::Bytes,
    ) -> Result<(reqwest::StatusCode, Option<String>, bytes::Bytes), JmapError> {
        let resp = self
            .http
            .post(url)
            .header("Authorization", &self.authorization)
            .header("Content-Type", content_type)
            .body(body)
            .send()
            .await?;
        let status = resp.status();
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let bytes = resp.bytes().await?;
        Ok((status, content_type, bytes))
    }

    /// GET a blob/download URL with injected auth, returning
    /// `(status, content_type, content_disposition, body)` so the proxy can
    /// stream an upstream attachment/message download back to the browser
    /// verbatim (RFC 8620 §6.2).
    #[allow(clippy::type_complexity)]
    pub async fn get_bytes(
        &self,
        url: &str,
    ) -> Result<
        (
            reqwest::StatusCode,
            Option<String>,
            Option<String>,
            bytes::Bytes,
        ),
        JmapError,
    > {
        let resp = self
            .http
            .get(url)
            .header("Authorization", &self.authorization)
            .send()
            .await?;
        let status = resp.status();
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let content_disposition = resp
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let bytes = resp.bytes().await?;
        Ok((status, content_type, content_disposition, bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_url_normalization() {
        assert_eq!(
            session_url("https://mail.example.org").unwrap(),
            "https://mail.example.org/.well-known/jmap"
        );
        assert_eq!(
            session_url("https://mail.example.org/ ").unwrap(),
            "https://mail.example.org/.well-known/jmap"
        );
        assert_eq!(
            session_url("http://mock:8181/.well-known/jmap").unwrap(),
            "http://mock:8181/.well-known/jmap"
        );
        assert!(session_url("ftp://mail.example.org").is_err());
        assert!(session_url("mail.example.org").is_err());
    }

    #[test]
    fn basic_auth_header() {
        let c = JmapClient::new("user", "pass").unwrap();
        assert_eq!(c.authorization(), "Basic dXNlcjpwYXNz");
    }
}
