//! The transport seam + a tiny authorized Microsoft Graph client.
//!
//! Everything here is target-independent pure Rust: the mapping modules
//! (`mail`/`calendar`/`contacts`/`todo`/`caps`) drive a [`GraphClient`] over a
//! [`Transport`], and BOTH the `wasm32` guest (`HostTransport`, over the gated host
//! imports) and the host-side unit tests (`FixtureTransport`, over recorded
//! fixtures) implement the same trait. A guest never opens a socket and never holds
//! a long-lived credential: it asks the host for a short-lived token per call
//! ([`Transport::token`]) and hands every request to the host ([`Transport::fetch`]).

use serde::de::DeserializeOwned;

/// The Graph v1.0 base URL. The ONLY host the bridge's own requests target
/// (`graph.microsoft.com`); OAuth (`login.microsoftonline.com`) is host-side.
pub const GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";

/// A coarse bridge error mirroring the WIT `plugin-error` variants so the guest can
/// map it 1:1 across the ABI without loss.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    /// A protocol/parse problem (malformed Graph JSON, unexpected shape).
    Protocol(String),
    /// Authentication/authorization failure (token acquisition, 401/403).
    Auth(String),
    /// A transport failure (host fetch failed, non-2xx where fatal).
    Transport(String),
    /// The operation is not expressible/supported over Graph.
    Unsupported(String),
    /// A referenced mailbox/folder was not found (404 on a folder path).
    MailboxNotFound(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::Protocol(m) => write!(f, "protocol: {m}"),
            BridgeError::Auth(m) => write!(f, "auth: {m}"),
            BridgeError::Transport(m) => write!(f, "transport: {m}"),
            BridgeError::Unsupported(m) => write!(f, "unsupported: {m}"),
            BridgeError::MailboxNotFound(m) => write!(f, "mailbox not found: {m}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, BridgeError>;

/// A host-mediated outbound HTTP request (plain, matches the WIT `http-request`).
#[derive(Debug, Clone)]
pub struct HttpRequestSpec {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

/// The response returned by the host (plain, matches the WIT `http-response`).
#[derive(Debug, Clone)]
pub struct HttpResponseData {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// The seam every Graph operation runs over. Implemented by the `wasm32` guest (host
/// imports) and by the host-side fixture replayer.
pub trait Transport {
    /// Acquire a short-lived bearer token for the bound account. The host holds and
    /// refreshes the long-lived secret; the guest sees only a transient access token
    /// it attaches to the very next request (never persisted, never logged).
    fn token(&self, account: &str) -> Result<String>;
    /// Perform one host-mediated HTTP request.
    fn fetch(&self, req: HttpRequestSpec) -> Result<HttpResponseData>;
}

/// A thin authorized Graph client bound to one account.
pub struct GraphClient<'a, T: Transport> {
    transport: &'a T,
    account: &'a str,
}

impl<'a, T: Transport> GraphClient<'a, T> {
    pub fn new(transport: &'a T, account: &'a str) -> Self {
        Self { transport, account }
    }

    fn authed(
        &self,
        method: &str,
        url: String,
        body: Option<Vec<u8>>,
        content_type: Option<&str>,
    ) -> Result<HttpResponseData> {
        // Acquire the token immediately before the call and attach it transiently.
        let token = self.transport.token(self.account)?;
        let mut headers = vec![
            ("Authorization".to_string(), format!("Bearer {token}")),
            ("Accept".to_string(), "application/json".to_string()),
        ];
        if let Some(ct) = content_type {
            headers.push(("Content-Type".to_string(), ct.to_string()));
        }
        let resp = self.transport.fetch(HttpRequestSpec {
            method: method.to_string(),
            url,
            headers,
            body,
        })?;
        if resp.status == 401 || resp.status == 403 {
            return Err(BridgeError::Auth(format!("Graph returned {}", resp.status)));
        }
        if resp.status == 404 {
            return Err(BridgeError::MailboxNotFound(format!(
                "Graph returned 404 for {method}"
            )));
        }
        if resp.status >= 400 {
            return Err(BridgeError::Transport(format!(
                "Graph returned {}",
                resp.status
            )));
        }
        Ok(resp)
    }

    /// Resolve a URL against `GRAPH_BASE` unless it is already absolute (a
    /// `@odata.deltaLink`/`nextLink` is absolute and used verbatim).
    fn absolute(path: &str) -> String {
        if path.starts_with("https://") || path.starts_with("http://") {
            path.to_string()
        } else {
            format!("{GRAPH_BASE}{path}")
        }
    }

    /// GET a path and deserialize the JSON body.
    pub fn get_json<D: DeserializeOwned>(&self, path: &str) -> Result<D> {
        let resp = self.authed("GET", Self::absolute(path), None, None)?;
        serde_json::from_slice(&resp.body)
            .map_err(|e| BridgeError::Protocol(format!("decode GET {path}: {e}")))
    }

    /// GET a path and return the raw response body (e.g. `$value` MIME).
    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        Ok(self.authed("GET", Self::absolute(path), None, None)?.body)
    }

    /// POST a JSON value; deserialize the JSON response (or ignore if empty).
    pub fn post_json<D: DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<D> {
        let raw = serde_json::to_vec(body)
            .map_err(|e| BridgeError::Protocol(format!("encode POST {path}: {e}")))?;
        let resp = self.authed(
            "POST",
            Self::absolute(path),
            Some(raw),
            Some("application/json"),
        )?;
        if resp.body.is_empty() {
            // 202/204 with no content — synthesize `null` for `Option`/unit targets.
            return serde_json::from_slice(b"null")
                .map_err(|e| BridgeError::Protocol(format!("decode empty POST {path}: {e}")));
        }
        serde_json::from_slice(&resp.body)
            .map_err(|e| BridgeError::Protocol(format!("decode POST {path}: {e}")))
    }

    /// POST a JSON value; ignore the response body (fire-and-forget: move/send).
    pub fn post_ignore(&self, path: &str, body: &serde_json::Value) -> Result<()> {
        let raw = serde_json::to_vec(body)
            .map_err(|e| BridgeError::Protocol(format!("encode POST {path}: {e}")))?;
        self.authed(
            "POST",
            Self::absolute(path),
            Some(raw),
            Some("application/json"),
        )?;
        Ok(())
    }

    /// POST a raw body with an explicit content-type (base64 MIME → `/sendMail`).
    pub fn post_raw(&self, path: &str, body: Vec<u8>, content_type: &str) -> Result<()> {
        self.authed("POST", Self::absolute(path), Some(body), Some(content_type))?;
        Ok(())
    }

    /// PATCH a JSON value; ignore the response body (flag/category updates).
    pub fn patch_ignore(&self, path: &str, body: &serde_json::Value) -> Result<()> {
        let raw = serde_json::to_vec(body)
            .map_err(|e| BridgeError::Protocol(format!("encode PATCH {path}: {e}")))?;
        self.authed(
            "PATCH",
            Self::absolute(path),
            Some(raw),
            Some("application/json"),
        )?;
        Ok(())
    }
}
