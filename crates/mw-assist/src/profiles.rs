//! AI provider profiles (t39 N8): a named endpoint + model + sealed key + options,
//! assigned per deployment, domain or user by the settings registry key
//! `assist.profile`.
//!
//! This module holds what does not need a store:
//!
//! - [`PRESETS`] — starting points for common providers. Each one states how far it
//!   has been checked ([`Verification`]); nothing here claims more than a test ran.
//! - [`ProfileSpec`] — one profile as the gateway uses it, API key opened.
//! - [`ProfileAdapter`] — an [`EndpointAdapter`] over a profile. Every request goes
//!   through the `mw-egress` address gate: the endpoint is resolved once, the
//!   address checked, and the connection pinned to it with no proxy and no
//!   redirects. A private, loopback or LAN address is refused unless the profile's
//!   `host:port` is covered by an administrator's egress grant
//!   ([`EgressAccess::Granted`]); link-local and metadata addresses are refused even
//!   then.
//! - [`test_profile`] — the admin "Test profile" call: one tiny fixed prompt, the
//!   latency, and a classified error ([`ProfileErrorKind`]) that never carries the
//!   key or the upstream body.
//!
//! The API key lives in [`ProfileSpec::api_key`] only in memory. `Debug` on every
//! type here prints `<redacted>` in its place, and no error string is built from a
//! request header or an upstream body.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::adapters::{ChatPayload, ChatStream, EndpointAdapter, Provider, sse_stream};
use crate::{AssistError, Result};

// ---------------------------------------------------------------------------
// Wire dialects and presets
// ---------------------------------------------------------------------------

/// The request format a profile speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dialect {
    /// `POST {endpoint}/chat/completions`, `Authorization: Bearer`.
    Openai,
    /// Azure OpenAI: `POST {endpoint}/chat/completions?api-version=…`, `api-key:`
    /// header. `endpoint` is the deployment URL
    /// (`https://<resource>.openai.azure.com/openai/deployments/<deployment>`).
    AzureOpenai,
    /// Anthropic Messages API: `POST {endpoint}/v1/messages`, `x-api-key:`.
    Anthropic,
}

impl Dialect {
    /// The storage and wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Dialect::Openai => "openai",
            Dialect::AzureOpenai => "azure-openai",
            Dialect::Anthropic => "anthropic",
        }
    }

    /// Parse the storage spelling.
    #[must_use]
    pub fn parse(s: &str) -> Option<Dialect> {
        [Dialect::Openai, Dialect::AzureOpenai, Dialect::Anthropic]
            .into_iter()
            .find(|d| d.as_str() == s)
    }
}

/// How far a preset has been checked. Shown next to the preset in the admin UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verification {
    /// Run against a local mock that speaks the documented wire format of the
    /// dialect (the tests in `crates/mw-server/tests/t39_assist_profiles.rs`). Not a
    /// run against the provider's live service.
    MockWire,
    /// Not run against the provider or a mock of it. The dialect may be shared with
    /// a checked preset, but the endpoint, model name and the provider's handling of
    /// the request are not confirmed.
    Unverified,
}

/// A starting point for a profile. Every field is editable once a profile is made.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: &'static str,
    pub label: &'static str,
    pub dialect: Dialect,
    /// The endpoint the profile starts with. May contain a `<placeholder>` the
    /// administrator replaces (Azure).
    pub endpoint: &'static str,
    /// Suggested models; the first is the default. Empty: the administrator names
    /// the model the server has loaded.
    pub models: &'static [&'static str],
    /// Whether the provider requires an API key.
    pub key_required: bool,
    /// Whether the endpoint is normally on this host or the local network, so an
    /// egress grant is needed before it can be reached.
    pub local: bool,
    pub verification: Verification,
}

/// The presets (t39 N8). Two are [`Verification::MockWire`]; the rest are
/// [`Verification::Unverified`]. Model names are the provider's at the time of
/// writing and change; they are suggestions, not checked values.
pub static PRESETS: &[Preset] = &[
    Preset {
        id: "openai-compatible",
        label: "OpenAI-compatible (any endpoint)",
        dialect: Dialect::Openai,
        endpoint: "",
        models: &[],
        key_required: false,
        local: false,
        verification: Verification::MockWire,
    },
    Preset {
        id: "anthropic",
        label: "Anthropic",
        dialect: Dialect::Anthropic,
        endpoint: "https://api.anthropic.com",
        models: &["claude-opus-5-5", "claude-sonnet-5-5", "claude-haiku-5-5"],
        key_required: true,
        local: false,
        verification: Verification::MockWire,
    },
    Preset {
        id: "openai",
        label: "OpenAI",
        dialect: Dialect::Openai,
        endpoint: "https://api.openai.com/v1",
        models: &["gpt-4o-mini", "gpt-4o"],
        key_required: true,
        local: false,
        verification: Verification::Unverified,
    },
    Preset {
        id: "azure-openai",
        label: "Azure OpenAI",
        dialect: Dialect::AzureOpenai,
        endpoint: "https://<resource>.openai.azure.com/openai/deployments/<deployment>",
        models: &[],
        key_required: true,
        local: false,
        verification: Verification::Unverified,
    },
    Preset {
        id: "google",
        label: "Google Gemini (OpenAI-compatible endpoint)",
        dialect: Dialect::Openai,
        endpoint: "https://generativelanguage.googleapis.com/v1beta/openai",
        models: &["gemini-2.5-flash", "gemini-2.5-pro"],
        key_required: true,
        local: false,
        verification: Verification::Unverified,
    },
    Preset {
        id: "mistral",
        label: "Mistral AI",
        dialect: Dialect::Openai,
        endpoint: "https://api.mistral.ai/v1",
        models: &["mistral-small-latest", "mistral-large-latest"],
        key_required: true,
        local: false,
        verification: Verification::Unverified,
    },
    Preset {
        id: "groq",
        label: "Groq",
        dialect: Dialect::Openai,
        endpoint: "https://api.groq.com/openai/v1",
        models: &["llama-3.3-70b-versatile"],
        key_required: true,
        local: false,
        verification: Verification::Unverified,
    },
    Preset {
        id: "together",
        label: "Together AI",
        dialect: Dialect::Openai,
        endpoint: "https://api.together.xyz/v1",
        models: &["meta-llama/Llama-3.3-70B-Instruct-Turbo"],
        key_required: true,
        local: false,
        verification: Verification::Unverified,
    },
    Preset {
        id: "openrouter",
        label: "OpenRouter",
        dialect: Dialect::Openai,
        endpoint: "https://openrouter.ai/api/v1",
        models: &["openrouter/auto"],
        key_required: true,
        local: false,
        verification: Verification::Unverified,
    },
    Preset {
        id: "deepseek",
        label: "DeepSeek",
        dialect: Dialect::Openai,
        endpoint: "https://api.deepseek.com/v1",
        models: &["deepseek-chat"],
        key_required: true,
        local: false,
        verification: Verification::Unverified,
    },
    Preset {
        id: "fireworks",
        label: "Fireworks AI",
        dialect: Dialect::Openai,
        endpoint: "https://api.fireworks.ai/inference/v1",
        models: &["accounts/fireworks/models/llama-v3p3-70b-instruct"],
        key_required: true,
        local: false,
        verification: Verification::Unverified,
    },
    Preset {
        id: "perplexity",
        label: "Perplexity",
        dialect: Dialect::Openai,
        endpoint: "https://api.perplexity.ai",
        models: &["sonar"],
        key_required: true,
        local: false,
        verification: Verification::Unverified,
    },
    Preset {
        id: "ollama",
        label: "Ollama (local)",
        dialect: Dialect::Openai,
        endpoint: "http://127.0.0.1:11434/v1",
        models: &["llama3.2"],
        key_required: false,
        local: true,
        verification: Verification::Unverified,
    },
    Preset {
        id: "lm-studio",
        label: "LM Studio (local)",
        dialect: Dialect::Openai,
        endpoint: "http://127.0.0.1:1234/v1",
        models: &[],
        key_required: false,
        local: true,
        verification: Verification::Unverified,
    },
    Preset {
        id: "vllm",
        label: "vLLM (local)",
        dialect: Dialect::Openai,
        endpoint: "http://127.0.0.1:8000/v1",
        models: &[],
        key_required: false,
        local: true,
        verification: Verification::Unverified,
    },
    Preset {
        id: "llama-cpp",
        label: "llama.cpp server (local)",
        dialect: Dialect::Openai,
        endpoint: "http://127.0.0.1:8080/v1",
        models: &[],
        key_required: false,
        local: true,
        verification: Verification::Unverified,
    },
];

/// The preset with this id.
#[must_use]
pub fn preset(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.id == id)
}

// ---------------------------------------------------------------------------
// Profile parameters and spec
// ---------------------------------------------------------------------------

/// Default whole-request time limit when a profile sets none.
pub const DEFAULT_TIMEOUT_MS: u64 = 60_000;
/// Bounds of `timeoutMs`.
pub const MIN_TIMEOUT_MS: u64 = 1_000;
pub const MAX_TIMEOUT_MS: u64 = 600_000;
/// Default `max_tokens` for the Anthropic dialect (the same ceiling the
/// deployment adapter uses; see `adapters::default_max_tokens`).
pub const DEFAULT_ANTHROPIC_MAX_TOKENS: u32 = 16_000;
/// Default `anthropic-version` header.
pub const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";
/// Default Azure `api-version` query parameter.
pub const DEFAULT_AZURE_API_VERSION: &str = "2024-10-21";

/// The optional request options of a profile. Unset fields are not sent (the
/// provider's default applies), except `maxTokens` on the Anthropic dialect, which
/// the Messages API requires.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct ProfileParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Whole-request time limit in milliseconds, connect to last byte.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// Azure `api-version`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
    /// Anthropic `anthropic-version` header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anthropic_version: Option<String>,
    /// Model for `/audio/transcriptions` (dictation); unset: dictation is not
    /// offered through this profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription_model: Option<String>,
}

impl ProfileParams {
    /// Refuse values a provider would refuse, before anything is stored.
    ///
    /// # Errors
    /// A message naming the field.
    pub fn validate(&self) -> std::result::Result<(), String> {
        if let Some(n) = self.max_tokens
            && !(1..=1_000_000).contains(&n)
        {
            return Err("params.maxTokens must be between 1 and 1000000".into());
        }
        if let Some(t) = self.temperature
            && !(t.is_finite() && (0.0..=2.0).contains(&t))
        {
            return Err("params.temperature must be between 0 and 2".into());
        }
        if let Some(ms) = self.timeout_ms
            && !(MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&ms)
        {
            return Err(format!(
                "params.timeoutMs must be between {MIN_TIMEOUT_MS} and {MAX_TIMEOUT_MS}"
            ));
        }
        for (name, v) in [
            ("apiVersion", &self.api_version),
            ("anthropicVersion", &self.anthropic_version),
            ("transcriptionModel", &self.transcription_model),
        ] {
            if let Some(s) = v
                && (s.is_empty() || s.len() > 128 || s.chars().any(char::is_control))
            {
                return Err(format!("params.{name} must be 1 to 128 characters"));
            }
        }
        Ok(())
    }

    /// The whole-request time limit.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(
            self.timeout_ms
                .unwrap_or(DEFAULT_TIMEOUT_MS)
                .clamp(MIN_TIMEOUT_MS, MAX_TIMEOUT_MS),
        )
    }
}

/// One profile as the gateway uses it. `api_key` is in the clear here and only
/// here; `Debug` prints `<redacted>`.
#[derive(Clone)]
pub struct ProfileSpec {
    pub id: String,
    pub dialect: Dialect,
    /// The API root, no trailing slash.
    pub endpoint: String,
    pub model: String,
    pub api_key: String,
    pub params: ProfileParams,
}

impl std::fmt::Debug for ProfileSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfileSpec")
            .field("id", &self.id)
            .field("dialect", &self.dialect)
            .field("endpoint", &self.endpoint)
            .field("model", &self.model)
            .field("api_key", &"<redacted>")
            .field("params", &self.params)
            .finish()
    }
}

/// Check an endpoint URL: `http://` or `https://`, a host, no credentials, no
/// query or fragment, no `<placeholder>` left in it.
///
/// # Errors
/// A message naming the problem.
pub fn validate_endpoint(endpoint: &str) -> std::result::Result<reqwest::Url, String> {
    if endpoint.len() > 2048 {
        return Err("endpoint is longer than 2048 characters".into());
    }
    if endpoint.contains('<') || endpoint.contains('>') {
        return Err("endpoint still contains a <placeholder>; replace it".into());
    }
    let url = reqwest::Url::parse(endpoint).map_err(|_| "endpoint is not a URL".to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("endpoint must be an http:// or https:// URL".into());
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err("endpoint has no host".into());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("endpoint must not contain credentials; use the API key field".into());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("endpoint must not contain a query or fragment".into());
    }
    Ok(url)
}

/// The `host:port` an endpoint connects to, in the spelling egress grants use:
/// lower-case host, IPv6 in brackets, the scheme's default port when none is
/// given.
#[must_use]
pub fn destination_of(endpoint: &str) -> Option<String> {
    let url = reqwest::Url::parse(endpoint).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    let port = url.port_or_known_default()?;
    Some(format!("{host}:{port}"))
}

/// Normalise an egress grant destination: `host:port` with an explicit port, a
/// hostname or IP literal (IPv6 in brackets). Refuses addresses no grant may open:
/// link-local (including the cloud metadata address), multicast and unspecified.
///
/// # Errors
/// A message naming the problem.
pub fn parse_grant(input: &str) -> std::result::Result<String, String> {
    let s = input.trim();
    if s.is_empty() || s.len() > 300 || s.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err("destination must be host:port".into());
    }
    if s.contains("://") || s.contains('/') || s.contains('@') {
        return Err("destination is host:port, not a URL".into());
    }
    let (host, port) = s
        .rsplit_once(':')
        .ok_or_else(|| "destination needs a port (host:port)".to_string())?;
    let port: u16 = port
        .parse()
        .ok()
        .filter(|p| *p != 0)
        .ok_or_else(|| "port must be 1 to 65535".to_string())?;
    let host = host.to_ascii_lowercase();
    let ip = match host.strip_prefix('[').and_then(|h| h.strip_suffix(']')) {
        Some(v6) => Some(
            v6.parse::<std::net::Ipv6Addr>()
                .map(IpAddr::V6)
                .map_err(|_| "bracketed host is not an IPv6 address".to_string())?,
        ),
        None => {
            if host.contains(':') {
                return Err("an IPv6 address must be in brackets: [::1]:11434".into());
            }
            host.parse::<std::net::Ipv4Addr>().ok().map(IpAddr::V4)
        }
    };
    if host.is_empty() {
        return Err("destination has no host".into());
    }
    if ip.is_none()
        && !host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return Err("host is not a valid hostname".into());
    }
    if let Some(ip) = ip
        && !granted_ip_allowed(&ip)
    {
        return Err(
            "link-local, metadata, multicast and unspecified addresses can never be granted".into(),
        );
    }
    Ok(format!("{host}:{port}"))
}

/// The address policy for an endpoint an administrator has granted: loopback, or
/// anything [`mw_egress::on_prem_allowed`] admits (private ranges and public
/// addresses; never link-local, metadata, multicast, unspecified or their IPv6
/// embeddings).
#[must_use]
pub fn granted_ip_allowed(ip: &IpAddr) -> bool {
    let loopback = match ip {
        IpAddr::V4(v4) => v4.is_loopback(),
        IpAddr::V6(v6) => {
            v6.is_loopback() || v6.to_ipv4_mapped().is_some_and(|v4| v4.is_loopback())
        }
    };
    loopback || mw_egress::on_prem_allowed(ip)
}

/// Which address policy a profile's requests run under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EgressAccess {
    /// [`mw_egress::ip_allowed`]: public addresses only. The default.
    Public,
    /// [`granted_ip_allowed`]: the profile's `host:port` matches an egress grant.
    Granted,
}

impl EgressAccess {
    /// The access a profile's endpoint gets under `grants` (normalised with
    /// [`parse_grant`]).
    #[must_use]
    pub fn for_endpoint(endpoint: &str, grants: &[String]) -> EgressAccess {
        match destination_of(endpoint) {
            Some(d) if grants.contains(&d) => EgressAccess::Granted,
            _ => EgressAccess::Public,
        }
    }

    fn policy(self) -> fn(&IpAddr) -> bool {
        match self {
            EgressAccess::Public => mw_egress::ip_allowed,
            EgressAccess::Granted => granted_ip_allowed,
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Why a profile request failed. The codes are stable wire values for the admin
/// UI; none carries the key, a header or the upstream body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileErrorKind {
    /// The endpoint resolved only to addresses the egress policy refuses (a
    /// private or loopback address without a grant, or one no grant opens), or did
    /// not resolve.
    EgressRefused,
    /// The time limit passed.
    Timeout,
    /// No connection or TLS session could be made.
    Connect,
    /// `401` or `403`: the key is missing, wrong or lacks access.
    Unauthorized,
    /// `404`: wrong endpoint path, deployment or model.
    NotFound,
    /// `429`.
    RateLimited,
    /// Another `4xx`: the provider refused the request (often an unknown model or
    /// option).
    Rejected,
    /// `5xx`.
    Upstream,
    /// The answer was not the dialect's format.
    BadResponse,
    /// The provider declined to answer (a refusal or content filter).
    Declined,
    /// The profile cannot do this (for example dictation without a transcription
    /// model).
    Unsupported,
}

impl ProfileErrorKind {
    /// The wire code.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            ProfileErrorKind::EgressRefused => "egress-refused",
            ProfileErrorKind::Timeout => "timeout",
            ProfileErrorKind::Connect => "connect",
            ProfileErrorKind::Unauthorized => "unauthorized",
            ProfileErrorKind::NotFound => "not-found",
            ProfileErrorKind::RateLimited => "rate-limited",
            ProfileErrorKind::Rejected => "rejected",
            ProfileErrorKind::Upstream => "upstream",
            ProfileErrorKind::BadResponse => "bad-response",
            ProfileErrorKind::Declined => "declined",
            ProfileErrorKind::Unsupported => "unsupported",
        }
    }

    fn from_status(status: u16) -> ProfileErrorKind {
        match status {
            401 | 403 => ProfileErrorKind::Unauthorized,
            404 => ProfileErrorKind::NotFound,
            408 => ProfileErrorKind::Timeout,
            429 => ProfileErrorKind::RateLimited,
            500..=599 => ProfileErrorKind::Upstream,
            _ => ProfileErrorKind::Rejected,
        }
    }
}

/// A failed profile request: the kind, the HTTP status when there was one, and the
/// provider's own error code when it sent a short machine-readable one
/// (`invalid_api_key`, `authentication_error`, `model_not_found`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileError {
    pub kind: ProfileErrorKind,
    pub status: Option<u16>,
    pub upstream_code: Option<String>,
}

impl ProfileError {
    fn of(kind: ProfileErrorKind) -> Self {
        Self {
            kind,
            status: None,
            upstream_code: None,
        }
    }

    fn transport(e: &reqwest::Error) -> Self {
        Self::of(if e.is_timeout() {
            ProfileErrorKind::Timeout
        } else if e.is_connect() {
            ProfileErrorKind::Connect
        } else if e.is_decode() || e.is_body() {
            ProfileErrorKind::BadResponse
        } else {
            ProfileErrorKind::Connect
        })
    }

    /// The gateway error for this failure. The text is the wire code only.
    #[must_use]
    pub fn into_assist(self) -> AssistError {
        AssistError::Endpoint(format!("profile: {}", self.kind.code()))
    }
}

/// The provider's error code from an error body, when it is a short token. The
/// message is never read: OpenAI's echoes part of the key it refused.
fn upstream_code(body: &str) -> Option<String> {
    let v: Value = serde_json::from_str(body).ok()?;
    let candidates = [
        v.pointer("/error/code"),
        v.pointer("/error/type"),
        v.pointer("/error/status"),
        v.pointer("/type").filter(|t| t.as_str() != Some("error")),
    ];
    candidates
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .find(|s| {
            !s.is_empty()
                && s.len() <= 64
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        })
        .map(str::to_string)
}

// ---------------------------------------------------------------------------
// The adapter
// ---------------------------------------------------------------------------

/// An [`EndpointAdapter`] over one profile, egress-gated per request.
pub struct ProfileAdapter {
    spec: ProfileSpec,
    access: EgressAccess,
}

impl std::fmt::Debug for ProfileAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfileAdapter")
            .field("spec", &self.spec)
            .field("access", &self.access)
            .finish()
    }
}

/// Upper bound on an error body read for its code.
const ERROR_BODY_LIMIT: usize = 8 * 1024;

impl ProfileAdapter {
    /// An adapter for `spec` under `access` (see [`EgressAccess::for_endpoint`]).
    #[must_use]
    pub fn new(mut spec: ProfileSpec, access: EgressAccess) -> Self {
        spec.endpoint = spec.endpoint.trim_end_matches('/').to_string();
        Self { spec, access }
    }

    /// As an `Arc<dyn EndpointAdapter>` for the gateway.
    #[must_use]
    pub fn shared(self) -> Arc<dyn EndpointAdapter> {
        Arc::new(self)
    }

    /// The profile id.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.spec.id
    }

    /// Resolve `url` once under the profile's address policy and build a client
    /// pinned to that address: no proxy, no redirects, the profile's time limits.
    async fn pinned_client(
        &self,
        url: &reqwest::Url,
    ) -> std::result::Result<reqwest::Client, ProfileError> {
        let target = mw_egress::validate_and_resolve_with(url.clone(), self.access.policy())
            .await
            .map_err(|_| ProfileError::of(ProfileErrorKind::EgressRefused))?;
        let total = self.spec.params.timeout();
        // The connect bound of the t56 egress timeouts, never longer than the
        // whole-request limit.
        let connect = mw_egress::FetchTimeouts::REMOTE_IMAGE.connect.min(total);
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .resolve(&target.host, target.addr)
            .connect_timeout(connect)
            .timeout(total)
            .user_agent(mw_egress::class::CLASS_UA)
            .build()
            .map_err(|_| ProfileError::of(ProfileErrorKind::Connect))
    }

    fn url(&self, path: &str) -> std::result::Result<reqwest::Url, ProfileError> {
        let base = &self.spec.endpoint;
        let mut url = reqwest::Url::parse(&format!("{base}{path}"))
            .map_err(|_| ProfileError::of(ProfileErrorKind::Unsupported))?;
        if self.spec.dialect == Dialect::AzureOpenai {
            let version = self
                .spec
                .params
                .api_version
                .as_deref()
                .unwrap_or(DEFAULT_AZURE_API_VERSION);
            url.query_pairs_mut().append_pair("api-version", version);
        }
        Ok(url)
    }

    fn authorize(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let key = &self.spec.api_key;
        match self.spec.dialect {
            Dialect::Openai if key.is_empty() => req,
            Dialect::Openai => req.bearer_auth(key),
            Dialect::AzureOpenai => req.header("api-key", key),
            Dialect::Anthropic => req.header("x-api-key", key).header(
                "anthropic-version",
                self.spec
                    .params
                    .anthropic_version
                    .as_deref()
                    .unwrap_or(DEFAULT_ANTHROPIC_VERSION),
            ),
        }
    }

    fn chat_body(&self, payload: &ChatPayload, stream: bool) -> Value {
        let p = &self.spec.params;
        match self.spec.dialect {
            Dialect::Openai | Dialect::AzureOpenai => {
                let mut messages = Vec::new();
                if let Some(sys) = &payload.system {
                    messages.push(json!({ "role": "system", "content": sys }));
                }
                messages.push(json!({ "role": "user", "content": payload.prompt }));
                let mut body = json!({ "stream": stream, "messages": messages });
                // Azure routes by deployment; a model in the body is ignored there,
                // so it is sent only when the profile names one.
                if !self.spec.model.is_empty() {
                    body["model"] = json!(self.spec.model);
                }
                if let Some(n) = p.max_tokens {
                    body["max_tokens"] = json!(n);
                }
                if let Some(t) = p.temperature {
                    body["temperature"] = json!(t);
                }
                body
            }
            Dialect::Anthropic => {
                let mut body = json!({
                    "model": self.spec.model,
                    "max_tokens": p.max_tokens.unwrap_or(DEFAULT_ANTHROPIC_MAX_TOKENS),
                    "stream": stream,
                    "messages": [{ "role": "user", "content": payload.prompt }],
                });
                if let Some(sys) = &payload.system {
                    body["system"] = json!(sys);
                }
                if let Some(t) = p.temperature {
                    body["temperature"] = json!(t);
                }
                body
            }
        }
    }

    fn chat_path(&self) -> &'static str {
        match self.spec.dialect {
            Dialect::Openai | Dialect::AzureOpenai => "/chat/completions",
            Dialect::Anthropic => "/v1/messages",
        }
    }

    /// Send one request and turn a non-success status into a classified error.
    async fn send(
        &self,
        url: reqwest::Url,
        build: impl FnOnce(&reqwest::Client, reqwest::Url) -> reqwest::RequestBuilder,
    ) -> std::result::Result<reqwest::Response, ProfileError> {
        let client = self.pinned_client(&url).await?;
        let resp = self
            .authorize(build(&client, url))
            .send()
            .await
            .map_err(|e| ProfileError::transport(&e))?;
        let status = resp.status();
        if status.is_success() {
            return Ok(resp);
        }
        if status.is_redirection() {
            // Redirects are not followed: a key must not be replayed to a second
            // host, and a redirect is not a reply.
            return Err(ProfileError {
                kind: ProfileErrorKind::BadResponse,
                status: Some(status.as_u16()),
                upstream_code: None,
            });
        }
        let body = read_limited(resp, ERROR_BODY_LIMIT).await;
        Err(ProfileError {
            kind: ProfileErrorKind::from_status(status.as_u16()),
            status: Some(status.as_u16()),
            upstream_code: upstream_code(&body),
        })
    }

    /// Streaming chat with a classified error (the gateway path maps it to
    /// [`AssistError::Endpoint`]).
    ///
    /// # Errors
    /// See [`ProfileErrorKind`].
    pub async fn chat_classified(
        &self,
        payload: &ChatPayload,
    ) -> std::result::Result<ChatStream, ProfileError> {
        let url = self.url(self.chat_path())?;
        let body = self.chat_body(payload, true);
        let resp = self.send(url, |c, u| c.post(u).json(&body)).await?;
        let provider = match self.spec.dialect {
            Dialect::Anthropic => Provider::Anthropic,
            Dialect::Openai | Dialect::AzureOpenai => Provider::OpenAi,
        };
        Ok(sse_stream(resp, provider))
    }
}

/// Read at most `limit` bytes of a body as text; errors read as empty.
async fn read_limited(resp: reqwest::Response, limit: usize) -> String {
    let mut out = Vec::new();
    let mut stream = resp.bytes_stream();
    while let Some(Ok(chunk)) = stream.next().await {
        let room = limit.saturating_sub(out.len());
        out.extend_from_slice(&chunk[..chunk.len().min(room)]);
        if out.len() >= limit {
            break;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[async_trait]
impl EndpointAdapter for ProfileAdapter {
    async fn chat(&self, payload: &ChatPayload) -> Result<ChatStream> {
        self.chat_classified(payload)
            .await
            .map_err(ProfileError::into_assist)
    }

    async fn embed(&self, _input: &str) -> Result<Vec<f32>> {
        // Semantic search keeps the deployment endpoint: one index needs one
        // embedding model, and a per-user profile would mix vector spaces.
        Err(ProfileError::of(ProfileErrorKind::Unsupported).into_assist())
    }

    async fn transcribe(&self, audio: &[u8], mime: &str) -> Result<String> {
        let unsupported = || ProfileError::of(ProfileErrorKind::Unsupported).into_assist();
        let Some(model) = self.spec.params.transcription_model.clone() else {
            return Err(unsupported());
        };
        if self.spec.dialect == Dialect::Anthropic {
            return Err(unsupported());
        }
        let url = self
            .url("/audio/transcriptions")
            .map_err(ProfileError::into_assist)?;
        let part = reqwest::multipart::Part::bytes(audio.to_vec())
            .file_name("audio")
            .mime_str(mime)
            .map_err(|_| ProfileError::of(ProfileErrorKind::Rejected).into_assist())?;
        let form = reqwest::multipart::Form::new()
            .text("model", model)
            .part("file", part);
        let resp = self
            .send(url, |c, u| c.post(u).multipart(form))
            .await
            .map_err(ProfileError::into_assist)?;
        let text = resp
            .text()
            .await
            .map_err(|e| ProfileError::transport(&e).into_assist())?;
        crate::parse_openai_transcription(&text)
            .map_err(|_| ProfileError::of(ProfileErrorKind::BadResponse).into_assist())
    }

    fn host(&self) -> String {
        reqwest::Url::parse(&self.spec.endpoint)
            .ok()
            .and_then(|u| {
                u.host_str().map(|h| match u.port() {
                    Some(p) => format!("{h}:{p}"),
                    None => h.to_string(),
                })
            })
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Test profile
// ---------------------------------------------------------------------------

/// The fixed instruction of a profile test. Carries no mail content.
pub const TEST_SYSTEM: &str = "You are checking a connection. Reply with the single word OK.";
pub const TEST_PROMPT: &str = "Connection test from Mailwoman. Reply with OK.";
/// How much of the reply a test returns.
pub const TEST_REPLY_PREVIEW: usize = 200;

/// The result of [`test_profile`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestOutcome {
    pub ok: bool,
    /// Milliseconds from the start of the request to the end of the reply (or the
    /// failure).
    pub latency_ms: u64,
    /// The first [`TEST_REPLY_PREVIEW`] characters of the reply, when it succeeded.
    pub reply: Option<String>,
    pub error: Option<ProfileError>,
}

/// Send the fixed test prompt through `adapter` and read the whole reply.
pub async fn test_profile(adapter: &ProfileAdapter) -> TestOutcome {
    let start = Instant::now();
    let payload = ChatPayload {
        system: Some(TEST_SYSTEM.to_string()),
        prompt: TEST_PROMPT.to_string(),
    };
    let elapsed = |s: Instant| u64::try_from(s.elapsed().as_millis()).unwrap_or(u64::MAX);
    let fail = |error: ProfileError| TestOutcome {
        ok: false,
        latency_ms: elapsed(start),
        reply: None,
        error: Some(error),
    };
    let mut stream = match adapter.chat_classified(&payload).await {
        Ok(s) => s,
        Err(e) => return fail(e),
    };
    let mut reply = String::new();
    let mut finished = false;
    while let Some(item) = stream.next().await {
        match item {
            Ok(chunk) => {
                if reply.chars().count() < TEST_REPLY_PREVIEW {
                    reply.push_str(&chunk.delta);
                }
                if chunk.done {
                    finished = true;
                    break;
                }
            }
            Err(AssistError::Endpoint(msg)) => {
                // The SSE decoder reports a mid-stream refusal and a mid-stream
                // error frame with fixed texts; a transport failure carries
                // reqwest's text, which names no key.
                let kind = if msg.contains("declined") {
                    ProfileErrorKind::Declined
                } else if msg.contains("timed out") || msg.contains("timeout") {
                    ProfileErrorKind::Timeout
                } else if msg.contains("stream error") {
                    ProfileErrorKind::Upstream
                } else {
                    ProfileErrorKind::BadResponse
                };
                return fail(ProfileError::of(kind));
            }
            Err(_) => return fail(ProfileError::of(ProfileErrorKind::BadResponse)),
        }
    }
    if !finished && reply.is_empty() {
        return fail(ProfileError::of(ProfileErrorKind::BadResponse));
    }
    TestOutcome {
        ok: true,
        latency_ms: elapsed(start),
        reply: Some(reply.chars().take(TEST_REPLY_PREVIEW).collect()),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_unique_and_honest() {
        let mut ids: Vec<&str> = PRESETS.iter().map(|p| p.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), PRESETS.len(), "preset ids are unique");
        assert!(PRESETS.len() >= 15, "about fifteen presets");
        // Only the two dialect mocks in the server tests are claimed.
        let checked: Vec<&str> = PRESETS
            .iter()
            .filter(|p| p.verification != Verification::Unverified)
            .map(|p| p.id)
            .collect();
        assert_eq!(checked, ["openai-compatible", "anthropic"]);
        let anthropic = preset("anthropic").unwrap();
        assert_eq!(
            anthropic.models,
            ["claude-opus-5-5", "claude-sonnet-5-5", "claude-haiku-5-5"]
        );
        for p in PRESETS {
            if p.local {
                let d = destination_of(p.endpoint).unwrap();
                assert!(d.starts_with("127.0.0.1:"), "{}: {d}", p.id);
            }
        }
    }

    #[test]
    fn grants_name_a_host_and_port_and_never_open_metadata() {
        assert_eq!(parse_grant("127.0.0.1:11434").unwrap(), "127.0.0.1:11434");
        assert_eq!(parse_grant("Ollama.LAN:11434").unwrap(), "ollama.lan:11434");
        assert_eq!(parse_grant("[::1]:1234").unwrap(), "[::1]:1234");
        assert_eq!(parse_grant("10.0.0.5:8000").unwrap(), "10.0.0.5:8000");
        for bad in [
            "169.254.169.254:80",
            "[fe80::1]:80",
            "0.0.0.0:11434",
            "224.0.0.1:80",
            "127.0.0.1",
            "127.0.0.1:0",
            "http://127.0.0.1:11434",
            "::1:1234",
            "host name:1",
            "",
        ] {
            assert!(parse_grant(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_grant_admits_loopback_and_lan_but_not_link_local() {
        for ok in ["127.0.0.1", "::1", "10.1.2.3", "192.168.1.9", "8.8.8.8"] {
            assert!(granted_ip_allowed(&ok.parse().unwrap()), "{ok}");
        }
        for no in [
            "169.254.169.254",
            "fe80::1",
            "0.0.0.0",
            "::ffff:169.254.169.254",
        ] {
            assert!(!granted_ip_allowed(&no.parse().unwrap()), "{no}");
        }
        // Without a grant the strict policy applies.
        assert!(!mw_egress::ip_allowed(&"127.0.0.1".parse().unwrap()));
    }

    #[test]
    fn access_follows_the_exact_destination() {
        let grants = vec!["127.0.0.1:11434".to_string()];
        assert_eq!(
            EgressAccess::for_endpoint("http://127.0.0.1:11434/v1", &grants),
            EgressAccess::Granted
        );
        assert_eq!(
            EgressAccess::for_endpoint("http://127.0.0.1:11435/v1", &grants),
            EgressAccess::Public
        );
        assert_eq!(
            EgressAccess::for_endpoint("https://api.anthropic.com", &grants),
            EgressAccess::Public
        );
    }

    #[test]
    fn endpoints_are_checked() {
        assert!(validate_endpoint("https://api.openai.com/v1").is_ok());
        for bad in [
            "ftp://x/",
            "https://user:pw@x/",
            "https://x/v1?key=1",
            "https://<resource>.openai.azure.com/",
            "not a url",
        ] {
            assert!(validate_endpoint(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn params_are_bounded() {
        assert!(ProfileParams::default().validate().is_ok());
        let p = |f: fn(&mut ProfileParams)| {
            let mut p = ProfileParams::default();
            f(&mut p);
            p.validate()
        };
        assert!(p(|p| p.temperature = Some(2.5)).is_err());
        assert!(p(|p| p.temperature = Some(f64::NAN)).is_err());
        assert!(p(|p| p.timeout_ms = Some(10)).is_err());
        assert!(p(|p| p.max_tokens = Some(0)).is_err());
        assert!(p(|p| p.api_version = Some(String::new())).is_err());
        assert_eq!(
            ProfileParams::default().timeout(),
            Duration::from_millis(DEFAULT_TIMEOUT_MS)
        );
        let parsed: std::result::Result<ProfileParams, _> =
            serde_json::from_value(json!({ "maxTokens": 5, "bogus": 1 }));
        assert!(parsed.is_err(), "unknown params are refused");
    }

    #[test]
    fn the_upstream_code_is_a_token_never_the_message() {
        let openai = r#"{"error":{"message":"Incorrect API key provided: sk-abc***xyz","type":"invalid_request_error","code":"invalid_api_key"}}"#;
        assert_eq!(upstream_code(openai).as_deref(), Some("invalid_api_key"));
        let anthropic = r#"{"type":"error","error":{"type":"authentication_error","message":"invalid x-api-key"}}"#;
        assert_eq!(
            upstream_code(anthropic).as_deref(),
            Some("authentication_error")
        );
        assert_eq!(
            upstream_code(r#"{"error":{"code":"has spaces in it"}}"#),
            None
        );
        assert_eq!(upstream_code("<html>"), None);
    }

    #[test]
    fn debug_never_prints_the_key() {
        let spec = ProfileSpec {
            id: "p".into(),
            dialect: Dialect::Openai,
            endpoint: "https://x".into(),
            model: "m".into(),
            api_key: "sk-SECRET-e11".into(),
            params: ProfileParams::default(),
        };
        let adapter = ProfileAdapter::new(spec.clone(), EgressAccess::Public);
        assert!(!format!("{spec:?}").contains("SECRET"));
        assert!(!format!("{adapter:?}").contains("SECRET"));
    }

    #[test]
    fn status_codes_map_to_kinds() {
        assert_eq!(
            ProfileErrorKind::from_status(401),
            ProfileErrorKind::Unauthorized
        );
        assert_eq!(
            ProfileErrorKind::from_status(404),
            ProfileErrorKind::NotFound
        );
        assert_eq!(
            ProfileErrorKind::from_status(429),
            ProfileErrorKind::RateLimited
        );
        assert_eq!(
            ProfileErrorKind::from_status(400),
            ProfileErrorKind::Rejected
        );
        assert_eq!(
            ProfileErrorKind::from_status(503),
            ProfileErrorKind::Upstream
        );
    }
}
