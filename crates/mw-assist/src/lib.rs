#![forbid(unsafe_code)]
//! `mw-assist` — the Assist (AI) gateway (plan §2.4, SPEC §14).
//!
//! The engine-side gateway: the **UI never talks to an endpoint directly**.
//! Adapters ([`OpenAiCompatible`], [`Anthropic`], [`LocalProcess`]) are hand-rolled
//! serde over the in-tree `reqwest`(rustls) — **no LLM SDK** (the V6 `rmcp` lesson).
//!
//! [`AssistGateway::invoke`] enforces, IN ORDER (plan §2.4, §14; safety-critical R4):
//! capability granted → data-class ceiling (accounts/folders, `include_e2ee=false`
//! and `include_attachments=false` by **default**) → **redaction** (strip
//! E2EE-decrypted content + attachments unless explicitly granted) → **rate-limit**
//! → **audit** (capability + scope summary + endpoint host — **never content**) →
//! adapter dispatch (streaming chat; also [`embed`](AssistGateway::embed) and
//! [`transcribe`](AssistGateway::transcribe)).
//!
//! **Safety invariant (§14, plan §6 R4):** no capability transmits/deletes/accepts.
//! The [`AssistCapability`] enum has **no send/delete/accept variant** — a
//! compile-time guarantee. The `Assistant` capability delegates to the existing
//! `mw-mcp` tool registry via [`AssistantTools`] (inheriting its scope + `mail.send`
//! →Outbox gating); it adds no privileged path.

mod adapters;
mod assistant;
pub mod redact;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

pub use adapters::{
    AdapterConfig, Anthropic, ChatPayload, ChatStream, EndpointAdapter, LocalProcess,
    OpenAiCompatible, Provider, SseDecoder, parse_anthropic_message, parse_openai_chat,
    parse_openai_embeddings, parse_openai_transcription,
};
pub use assistant::AssistantTools;

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

impl AssistCapability {
    /// Every capability, for enumeration. **There is deliberately no send/delete/
    /// accept entry here or in the enum** (plan §6 R4).
    pub const ALL: [AssistCapability; 8] = [
        AssistCapability::Summarize,
        AssistCapability::Draft,
        AssistCapability::Grammar,
        AssistCapability::Dictation,
        AssistCapability::SearchSemantic,
        AssistCapability::AutoTag,
        AssistCapability::Recap,
        AssistCapability::Assistant,
    ];

    /// A short, content-free system instruction for the chat-shaped capabilities.
    #[must_use]
    pub fn system_prompt(self) -> &'static str {
        match self {
            AssistCapability::Summarize => "Summarize the message(s) concisely.",
            AssistCapability::Draft => "Help draft a reply. Do not send; the user confirms.",
            AssistCapability::Grammar => "Improve grammar and clarity; preserve meaning.",
            AssistCapability::Recap => "Recap the thread's key points and decisions.",
            AssistCapability::AutoTag => "Suggest labels/tags for the message(s).",
            AssistCapability::Assistant => "You are a mail assistant. Any send is human-gated.",
            AssistCapability::Dictation => "Transcribe speech to text.",
            AssistCapability::SearchSemantic => "Produce a semantic representation for re-ranking.",
        }
    }
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
/// accounts (empty) — the safe posture (R4). `folders` empty ⇒ all allowed folders.
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

impl DataScope {
    /// Clamp this per-call scope to the admin `ceiling` — a call can NEVER exceed
    /// the deployment/user ceiling. Booleans are ANDed (E2EE/attachments require
    /// BOTH the call and the ceiling to opt in); accounts intersect; folders use
    /// the "empty ⇒ all" rule so the tighter allowlist wins.
    #[must_use]
    pub fn clamp(&self, ceiling: &DataScope) -> DataScope {
        DataScope {
            accounts: intersect_ids(&self.accounts, &ceiling.accounts),
            folders: intersect_folders(&self.folders, &ceiling.folders),
            include_e2ee: self.include_e2ee && ceiling.include_e2ee,
            include_attachments: self.include_attachments && ceiling.include_attachments,
        }
    }

    /// A content-free summary for the audit row (accounts/folders counts + flags —
    /// never any mail content).
    #[must_use]
    pub fn summary(&self) -> String {
        let folders = if self.folders.is_empty() {
            "*".to_string()
        } else {
            self.folders.len().to_string()
        };
        format!(
            "accounts={} folders={} e2ee={} attach={}",
            self.accounts.len(),
            folders,
            self.include_e2ee,
            self.include_attachments,
        )
    }
}

/// Accounts are strict allowlists: empty ⇒ none, so the intersection of an empty
/// set is empty (the safe default).
fn intersect_ids(a: &[String], b: &[String]) -> Vec<String> {
    a.iter().filter(|x| b.contains(x)).cloned().collect()
}

/// Folders use the "empty ⇒ all" rule (per `DataScope::folders` docs); the tighter
/// (non-empty) list wins, and two non-empty lists intersect.
fn intersect_folders(a: &[String], b: &[String]) -> Vec<String> {
    match (a.is_empty(), b.is_empty()) {
        (true, _) => b.to_vec(),
        (_, true) => a.to_vec(),
        _ => intersect_ids(a, b),
    }
}

/// The classification of one piece of context handed to Assist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentKind {
    /// Ordinary (non-E2EE, non-attachment) mail content.
    Plain,
    /// E2EE-decrypted plaintext — **never** forwarded unless `include_e2ee`.
    E2eeDecrypted,
    /// Attachment content — **not** forwarded unless `include_attachments`.
    Attachment,
}

/// One context item drawn from the mailbox for an Assist invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextItem {
    /// Owning account id (checked against the data-class ceiling).
    pub account: String,
    /// Owning folder id (checked against the ceiling; empty ⇒ unfiled).
    #[serde(default)]
    pub folder: String,
    /// The content text.
    pub text: String,
    /// The privacy classification driving redaction.
    pub kind: ContentKind,
}

/// The input to an Assist chat invocation: the user's instruction plus the mailbox
/// context items to (selectively) forward. Redaction happens on `context` before
/// anything reaches an adapter.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssistInput {
    /// The user's own instruction (not mail content — always forwarded).
    pub prompt: String,
    /// Mailbox context items, redacted per scope before send.
    #[serde(default)]
    pub context: Vec<ContextItem>,
}

impl AssistInput {
    /// Convenience: a prompt with no mailbox context.
    #[must_use]
    pub fn prompt(text: impl Into<String>) -> Self {
        Self {
            prompt: text.into(),
            context: Vec::new(),
        }
    }
}

/// A streamed chat token (plan §2.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: String,
    pub done: bool,
}

/// Per-deployment + per-user Assist config (plan §2.4). Admin-lockable; unconfigured
/// (or `enabled=false`, or no `adapter`) ⇒ the gateway returns
/// [`AssistError::Disabled`] and the web hides all Assist UI.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AssistConfig {
    pub enabled: bool,
    /// Capabilities granted at this scope.
    pub capability_grants: Vec<AssistCapability>,
    /// The default data-class ceiling (per-call scope is clamped to this).
    pub data_ceiling: DataScope,
    /// The configured endpoint adapter (unset ⇒ Disabled).
    pub adapter: Option<AdapterConfig>,
    /// Per-key request rate limit (requests/min; None ⇒ unlimited).
    pub rate_limit_per_min: Option<u32>,
}

/// A content-free audit row (plan §2.4). Carries capability + scope summary +
/// endpoint host — **never mail content** (R4; asserted in tests + e16). The struct
/// is deliberately shaped so there is **no field that could hold content**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssistAudit {
    pub capability: AssistCapability,
    pub scope_summary: String,
    pub endpoint_host: String,
}

/// The append-only sink Assist audit rows are written to. e9/e14 back this with the
/// `assist_audit` table (0008); tests use [`InMemoryAudit`].
pub trait AssistAuditSink: Send + Sync {
    fn record(&self, row: AssistAudit);
}

/// A no-op audit sink (default when none is configured).
#[derive(Debug, Default)]
pub struct NoopAudit;
impl AssistAuditSink for NoopAudit {
    fn record(&self, _row: AssistAudit) {}
}

/// An in-memory audit sink for tests + local dev.
#[derive(Debug, Default)]
pub struct InMemoryAudit {
    rows: Mutex<Vec<AssistAudit>>,
}
impl InMemoryAudit {
    #[must_use]
    pub fn rows(&self) -> Vec<AssistAudit> {
        self.rows.lock().expect("audit lock").clone()
    }
}
impl AssistAuditSink for InMemoryAudit {
    fn record(&self, row: AssistAudit) {
        self.rows.lock().expect("audit lock").push(row);
    }
}

/// Fixed-window rate-limit state.
#[derive(Default)]
struct RateWindow {
    start: Option<Instant>,
    count: u32,
}

/// The Assist gateway (plan §2.4). Enforces capability + data-class scope,
/// redaction, rate-limit, and content-free audit, then dispatches to the adapter.
pub struct AssistGateway {
    config: AssistConfig,
    adapter: Option<Arc<dyn EndpointAdapter>>,
    audit: Arc<dyn AssistAuditSink>,
    rate: Mutex<RateWindow>,
}

impl AssistGateway {
    /// Build a gateway over a config, constructing the configured adapter. A
    /// disabled/unconfigured config makes every call return [`AssistError::Disabled`].
    #[must_use]
    pub fn new(config: AssistConfig) -> Self {
        let adapter = config.adapter.as_ref().and_then(AdapterConfig::build);
        Self {
            config,
            adapter,
            audit: Arc::new(NoopAudit),
            rate: Mutex::new(RateWindow::default()),
        }
    }

    /// Attach an audit sink (e9/e14 back this with the 0008 `assist_audit` table).
    #[must_use]
    pub fn with_audit(mut self, audit: Arc<dyn AssistAuditSink>) -> Self {
        self.audit = audit;
        self
    }

    /// Inject an adapter directly (dependency injection for tests / custom wiring).
    #[must_use]
    pub fn with_adapter(mut self, adapter: Arc<dyn EndpointAdapter>) -> Self {
        self.adapter = Some(adapter);
        self
    }

    /// Whether Assist is enabled (the web hides all UI when this is false).
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.adapter.is_some()
    }

    fn require_adapter(&self) -> Result<&Arc<dyn EndpointAdapter>> {
        if !self.config.enabled {
            return Err(AssistError::Disabled);
        }
        self.adapter.as_ref().ok_or(AssistError::Disabled)
    }

    fn check_capability(&self, cap: AssistCapability) -> Result<()> {
        if self.config.capability_grants.contains(&cap) {
            Ok(())
        } else {
            Err(AssistError::CapabilityDenied(cap))
        }
    }

    /// Fixed-window rate limit (per gateway). Trips on the request that would
    /// exceed `rate_limit_per_min` within the current 60-second window.
    fn check_rate(&self) -> Result<()> {
        let Some(limit) = self.config.rate_limit_per_min else {
            return Ok(());
        };
        let mut w = self.rate.lock().expect("rate lock");
        let now = Instant::now();
        let fresh =
            !matches!(w.start, Some(start) if now.duration_since(start) < Duration::from_secs(60));
        if fresh {
            w.start = Some(now);
            w.count = 0;
        }
        if w.count >= limit {
            return Err(AssistError::RateLimited);
        }
        w.count += 1;
        Ok(())
    }

    fn audit(&self, cap: AssistCapability, eff: &DataScope, host: String) {
        self.audit.record(AssistAudit {
            capability: cap,
            scope_summary: eff.summary(),
            endpoint_host: host,
        });
    }

    /// Invoke a **chat-shaped** capability (Summarize/Draft/Grammar/Recap/AutoTag/
    /// Assistant). Enforces the full §2.4 pipeline, then streams the reply.
    ///
    /// # Errors
    /// [`AssistError::Disabled`] when unconfigured, [`AssistError::CapabilityDenied`]
    /// when the capability is not granted, [`AssistError::RateLimited`] when the
    /// per-minute limit trips, or an [`AssistError::Endpoint`] transport error.
    pub async fn invoke(
        &self,
        cap: AssistCapability,
        scope: DataScope,
        input: &AssistInput,
    ) -> Result<ChatStream> {
        let adapter = self.require_adapter()?; // enabled + adapter present
        self.check_capability(cap)?; // 1. capability granted
        let eff = scope.clamp(&self.config.data_ceiling); // 2. data-class ceiling
        let payload = redact::redact_chat(input, &eff, cap); // 3. redaction
        self.check_rate()?; // 4. rate-limit
        self.audit(cap, &eff, adapter.host()); // 5. content-free audit
        adapter.chat(&payload).await // 6. dispatch (streaming)
    }

    /// Embeddings for the SearchSemantic re-rank slot (§14). Same enforcement
    /// pipeline; the query is the search text (not mailbox content).
    pub async fn embed(&self, scope: DataScope, query: &str) -> Result<Vec<f32>> {
        let adapter = self.require_adapter()?;
        self.check_capability(AssistCapability::SearchSemantic)?;
        let eff = scope.clamp(&self.config.data_ceiling);
        self.check_rate()?;
        self.audit(AssistCapability::SearchSemantic, &eff, adapter.host());
        adapter.embed(query).await
    }

    /// Speech-to-text for the Dictation slot (§14, Whisper-compatible). Audio is
    /// user-provided, not mailbox content; same enforcement pipeline.
    pub async fn transcribe(&self, scope: DataScope, audio: &[u8], mime: &str) -> Result<String> {
        let adapter = self.require_adapter()?;
        self.check_capability(AssistCapability::Dictation)?;
        let eff = scope.clamp(&self.config.data_ceiling);
        self.check_rate()?;
        self.audit(AssistCapability::Dictation, &eff, adapter.host());
        adapter.transcribe(audio, mime).await
    }
}

#[cfg(test)]
mod tests;
