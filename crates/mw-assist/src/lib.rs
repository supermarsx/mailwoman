#![forbid(unsafe_code)]
// SCAFFOLD (t7-e0): frozen §2.4 types + method shapes as INERT stubs; the adapters,
// scope/redaction/rate-limit enforcement, content-free audit, and streaming are e4.
#![allow(dead_code)]
//! `mw-assist` — the Assist (AI) gateway (plan §2.4, SPEC §14).
//!
//! The engine-side gateway: the **UI never talks to an endpoint directly**.
//! Adapters ([`OpenAiCompatible`], [`Anthropic`], [`LocalProcess`]) are hand-rolled
//! serde over the in-tree `reqwest`(rustls) — **no LLM SDK** (the V6 `rmcp` lesson).
//!
//! [`AssistGateway::invoke`] enforces, in order:
//! capability granted → data-class ceiling (accounts/folders, `include_e2ee=false`
//! and `include_attachments=false` by **default**) → **redaction** → **rate-limit**
//! → **audit** (capability + scope summary + endpoint host — **never content**).
//!
//! **Safety invariant (§14, plan §6 R4):** no capability transmits/deletes/accepts.
//! The [`AssistCapability`] enum has **no send/delete/accept variant** — this is a
//! compile-time guarantee. The `Assistant` capability delegates to the existing
//! `mw-mcp` tool registry (inheriting its scope + `mail.send`→Outbox gating); it
//! adds no privileged path.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// The Assist capabilities (plan §2.4). **Note the absence of any send/delete/
/// accept variant** — Assist can never transmit; that is a structural guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssistCapability {
    Summarize,
    Draft,
    Grammar,
    Dictation,
    SearchSemantic,
    AutoTag,
    Recap,
    /// The assistant chat — a client of the same tool surface as MCP (§14.3).
    Assistant,
}

/// Errors from the gateway (plan §2.4).
#[derive(Debug, thiserror::Error)]
pub enum AssistError {
    /// Assist is unconfigured for this deployment/user ⇒ the web hides all UI.
    #[error("assist disabled")]
    Disabled,
    /// The requested capability is not granted for this scope.
    #[error("assist capability not granted: {0:?}")]
    CapabilityDenied(AssistCapability),
    /// The scope's data-class ceiling forbids the requested content.
    #[error("assist data-class ceiling exceeded: {0}")]
    ScopeExceeded(String),
    /// Rate limit tripped.
    #[error("assist rate limit exceeded")]
    RateLimited,
    /// Endpoint/adapter transport error.
    #[error("assist endpoint error: {0}")]
    Endpoint(String),
    #[error("not implemented")]
    Unimplemented,
}

pub type Result<T> = std::result::Result<T, AssistError>;

/// The data-class ceiling for a single invocation (plan §2.4). The derived default
/// EXCLUDES E2EE-decrypted content and attachments (both `false`) and draws from no
/// accounts (empty) — the safe posture (R4).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DataScope {
    /// Accounts the content may be drawn from (empty ⇒ none).
    pub accounts: Vec<String>,
    /// Folders within those accounts (empty ⇒ all allowed folders).
    pub folders: Vec<String>,
    /// Forward E2EE-decrypted content? **Default false** (never, unless explicit).
    pub include_e2ee: bool,
    /// Forward attachments? **Default false**.
    pub include_attachments: bool,
}

/// A streamed chat token (placeholder; e4 backs this with a real stream type).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub done: bool,
}

/// An endpoint adapter (plan §2.4). Impls are thin serde over the in-tree `reqwest`.
#[async_trait]
pub trait EndpointAdapter: Send + Sync {
    /// Chat completion; e4 returns a real stream — the stub returns a whole reply.
    async fn chat(&self, prompt: &str) -> Result<Vec<StreamChunk>>;
    /// Text embeddings.
    async fn embed(&self, input: &str) -> Result<Vec<f32>>;
    /// Speech-to-text (Whisper-compatible).
    async fn transcribe(&self, audio: &[u8]) -> Result<String>;
}

macro_rules! stub_adapter {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Default)]
        pub struct $name;

        #[async_trait]
        impl EndpointAdapter for $name {
            async fn chat(&self, _prompt: &str) -> Result<Vec<StreamChunk>> {
                Err(AssistError::Unimplemented)
            }
            async fn embed(&self, _input: &str) -> Result<Vec<f32>> {
                Err(AssistError::Unimplemented)
            }
            async fn transcribe(&self, _audio: &[u8]) -> Result<String> {
                Err(AssistError::Unimplemented)
            }
        }
    };
}

stub_adapter!(
    OpenAiCompatible,
    "OpenAI-compatible chat/embeddings/transcriptions."
);
stub_adapter!(Anthropic, "Anthropic Messages API adapter.");
stub_adapter!(
    LocalProcess,
    "Local-process adapter (spawn a binary, stdio JSON)."
);

/// Per-deployment + per-user Assist config (plan §2.4). Admin-lockable; unconfigured
/// ⇒ the gateway returns [`AssistError::Disabled`] and the web hides all Assist UI.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AssistConfig {
    pub enabled: bool,
    /// Capabilities granted at this scope.
    pub capability_grants: Vec<AssistCapability>,
    /// The default data-class ceiling.
    pub data_ceiling: DataScope,
    /// The endpoint host (recorded in the content-free audit; never content).
    pub endpoint_host: Option<String>,
}

/// A content-free audit row (plan §2.4). Carries capability + scope summary +
/// endpoint host — **never mail content** (R4; asserted by e4 + e16).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssistAudit {
    pub capability: AssistCapability,
    pub scope_summary: String,
    pub endpoint_host: String,
}

/// The Assist gateway (plan §2.4). Enforces capability + data-class scope,
/// redaction, rate-limit, and content-free audit. Inert until e4 fills it.
pub struct AssistGateway {
    config: AssistConfig,
}

impl AssistGateway {
    /// Build a gateway over a config. An unconfigured/disabled config makes every
    /// [`invoke`](Self::invoke) return [`AssistError::Disabled`].
    #[must_use]
    pub fn new(config: AssistConfig) -> Self {
        Self { config }
    }

    /// Whether Assist is enabled (the web hides all UI when this is false).
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Invoke a capability. e4 enforces: capability grant → data-class ceiling →
    /// redaction → rate-limit → content-free audit → adapter dispatch (streaming).
    /// `Assistant` delegates to the `mw-mcp` tool registry (inherits scope + send
    /// gating). The stub reports `Disabled`/`Unimplemented`.
    pub async fn invoke(
        &self,
        _cap: AssistCapability,
        _scope: DataScope,
        _input: &str,
    ) -> Result<Vec<StreamChunk>> {
        if !self.config.enabled {
            return Err(AssistError::Disabled);
        }
        Err(AssistError::Unimplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_scope_defaults_are_safe() {
        let s = DataScope::default();
        assert!(!s.include_e2ee, "E2EE content excluded by default (R4)");
        assert!(!s.include_attachments, "attachments excluded by default");
    }

    #[tokio::test]
    async fn unconfigured_gateway_is_disabled() {
        let gw = AssistGateway::new(AssistConfig::default());
        assert!(!gw.is_enabled());
        assert!(matches!(
            gw.invoke(AssistCapability::Summarize, DataScope::default(), "hi")
                .await,
            Err(AssistError::Disabled)
        ));
    }
}
