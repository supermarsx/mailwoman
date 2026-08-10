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
//! [`embed`](AssistGateway::embed) runs the same pipeline and **refuses** rather than
//! dispatching when no account survives the ceiling clamp. That path carries mail
//! content (a semantic re-rank embeds the documents a search surfaced, not just the
//! query), and before 26.19 it clamped the scope only to name it in the audit row —
//! so the row could record `accounts=0` for a request that had already gone out. An
//! unenforced control is a bug; a control that reports itself enforced is worse,
//! because the audit trail is the compliance artifact. The rate limit is likewise a
//! **per-account** window, not a gateway-wide one.
//!
//! **Safety invariant (§14, plan §6 R4):** no capability transmits/deletes/accepts.
//! The [`AssistCapability`] enum has **no send/delete/accept variant** — a
//! compile-time guarantee. The `Assistant` capability delegates to the existing
//! `mw-mcp` tool registry via [`AssistantTools`] (inheriting its scope + `mail.send`
//! →Outbox gating); it adds no privileged path.

mod adapters;
mod assistant;
pub mod redact;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

pub use adapters::{
    AdapterConfig, Anthropic, ChatPayload, ChatStream, EndpointAdapter, LocalProcess,
    OpenAiCompatible, Provider, SseDecoder, parse_anthropic_message, parse_openai_chat,
    parse_openai_embeddings, parse_openai_transcription,
};
pub use assistant::{ACTION_MARKER, AssistantTools, ProposalFilter, ProposedAction, parse_actions};

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
            // The assistant may PROPOSE tool actions; it can never take one. The
            // proposal block is parsed out of the reply by `ProposalFilter` and shown
            // for human review, so the format is part of the contract (§14.3).
            AssistCapability::Assistant => concat!(
                "You are a mail assistant. You cannot send, delete, or accept anything: ",
                "every action you name is a proposal the user reviews and confirms.\n",
                "To propose actions, end your reply with a line containing exactly ",
                "<<<MW_ACTIONS followed by a JSON array of objects with \"tool\" and ",
                "\"summary\" string fields, and write nothing after that array. ",
                "Omit the line entirely when you are not proposing anything.",
            ),
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

/// What actually left the device on one invocation — the honest, per-call companion
/// to the static "what left the device" copy (§14). Built from the
/// [`redact::RedactionReport`], so it states what redaction really did rather than
/// what the ceiling nominally allows. Counts only; **never content**.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvokeDisclosure {
    /// The host the server-side request reached (the browser never contacts it).
    pub endpoint_host: String,
    /// Content classes that were forwarded.
    pub sent: Vec<String>,
    /// Content classes that were withheld, with the count that was dropped.
    pub withheld: Vec<String>,
}

impl InvokeDisclosure {
    /// Summarize one redaction pass for the user.
    #[must_use]
    pub fn from_report(report: &redact::RedactionReport, endpoint_host: String) -> Self {
        let mut sent = vec!["your prompt".to_string()];
        if report.kept > 0 {
            sent.push(format!("{} message excerpt(s)", report.kept));
        }
        let mut withheld = Vec::new();
        if report.dropped_e2ee > 0 {
            withheld.push(format!(
                "{} end-to-end-encrypted item(s)",
                report.dropped_e2ee
            ));
        }
        if report.dropped_attachment > 0 {
            withheld.push(format!("{} attachment(s)", report.dropped_attachment));
        }
        if report.dropped_scope > 0 {
            withheld.push(format!(
                "{} item(s) outside the permitted accounts or folders",
                report.dropped_scope
            ));
        }
        Self {
            endpoint_host,
            sent,
            withheld,
        }
    }
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

/// Fixed-window rate-limit state for one budget key.
#[derive(Default)]
struct RateWindow {
    start: Option<Instant>,
    count: u32,
}

/// The length of one rate-limit window.
const RATE_WINDOW: Duration = Duration::from_secs(60);

/// The rate-limit budget key for one call: the effective account set, order-normalised
/// so the same accounts always charge the same bucket whatever order the caller listed
/// them in. `\u{1f}` (unit separator) joins them because it cannot occur in an account
/// id, so two different account sets can never collide onto one budget.
fn rate_key(eff: &DataScope) -> String {
    let mut accounts = eff.accounts.clone();
    accounts.sort();
    accounts.join("\u{1f}")
}

/// The Assist gateway (plan §2.4). Enforces capability + data-class scope,
/// redaction, rate-limit, and content-free audit, then dispatches to the adapter.
pub struct AssistGateway {
    config: AssistConfig,
    adapter: Option<Arc<dyn EndpointAdapter>>,
    audit: Arc<dyn AssistAuditSink>,
    /// One fixed window per account set — see [`AssistGateway::check_rate`].
    rate: Mutex<HashMap<String, RateWindow>>,
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
            rate: Mutex::new(HashMap::new()),
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

    /// The capabilities granted at this scope. A disabled gateway grants **none**, so
    /// a caller can render the surface straight from this list.
    #[must_use]
    pub fn granted_capabilities(&self) -> Vec<AssistCapability> {
        if self.is_enabled() {
            self.config.capability_grants.clone()
        } else {
            Vec::new()
        }
    }

    /// The admin data-class ceiling every per-call scope is clamped to.
    #[must_use]
    pub fn data_ceiling(&self) -> &DataScope {
        &self.config.data_ceiling
    }

    /// The configured per-account request budget (requests/min; `None` ⇒ unlimited).
    ///
    /// Exposed so the mount site can assert that an operator's configured value
    /// actually reached the gateway. It was hardcoded `None` there until 26.19, which
    /// is the kind of thing only an observable value catches.
    #[must_use]
    pub fn rate_limit_per_min(&self) -> Option<u32> {
        self.config.rate_limit_per_min
    }

    /// The endpoint host content would be proxied to, for the disclosure. `None` when
    /// the gateway is disabled (nothing can leave).
    #[must_use]
    pub fn endpoint_host(&self) -> Option<String> {
        if self.is_enabled() {
            self.adapter.as_ref().map(|a| a.host())
        } else {
            None
        }
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

    /// Fixed-window rate limit, **per effective account set**. Trips on the request
    /// that would exceed `rate_limit_per_min` within the current 60-second window.
    ///
    /// Keyed per account rather than being one gateway-wide window because a single
    /// user's semantic search is up to `1 + LAZY_FILL_MAX` = 33 outbound requests on a
    /// cold embedding cache. Under one shared bucket that user could exhaust the
    /// deployment's whole minute and lock every other account out of Assist — a
    /// self-inflicted denial of service created by the control meant to prevent one.
    /// Per-account, the account that spends the budget is the account it bounds.
    ///
    /// The unit is one outbound request, deliberately: the harms this bounds are
    /// third-party API cost and bulk egress volume, and charging a 33-request pass as
    /// a single unit would leave both unbounded.
    fn check_rate(&self, eff: &DataScope) -> Result<()> {
        let Some(limit) = self.config.rate_limit_per_min else {
            return Ok(());
        };
        let key = rate_key(eff);
        let now = Instant::now();
        let mut windows = self.rate.lock().expect("rate lock");
        // Expired windows carry no budget, so dropping them is free and keeps the map
        // bounded by the accounts active in the last minute rather than by every
        // account the process has ever served.
        windows.retain(|_, w| matches!(w.start, Some(s) if now.duration_since(s) < RATE_WINDOW));
        let w = windows.entry(key).or_default();
        if w.start.is_none() {
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
        self.invoke_disclosed(cap, scope, input)
            .await
            .map(|(stream, _)| stream)
    }

    /// [`invoke`](Self::invoke) plus the per-call [`InvokeDisclosure`] — the same
    /// single pipeline, with the redaction outcome reported so the caller can tell the
    /// user what actually left the device (§14).
    ///
    /// # Errors
    /// Identical to [`invoke`](Self::invoke).
    pub async fn invoke_disclosed(
        &self,
        cap: AssistCapability,
        scope: DataScope,
        input: &AssistInput,
    ) -> Result<(ChatStream, InvokeDisclosure)> {
        let adapter = self.require_adapter()?; // enabled + adapter present
        self.check_capability(cap)?; // 1. capability granted
        let eff = scope.clamp(&self.config.data_ceiling); // 2. data-class ceiling
        let (payload, report) = redact::redact_chat_reported(input, &eff, cap); // 3. redaction
        self.check_rate(&eff)?; // 4. rate-limit
        let host = adapter.host();
        self.audit(cap, &eff, host.clone()); // 5. content-free audit
        let stream = adapter.chat(&payload).await?; // 6. dispatch (streaming)
        Ok((stream, InvokeDisclosure::from_report(&report, host)))
    }

    /// Embeddings for the SearchSemantic re-rank slot (§14). Same enforcement
    /// pipeline; `text` is either the user's search string or a document the search
    /// surfaced, so this path carries mail content and the ceiling is enforced on it.
    ///
    /// # Errors
    /// [`AssistError::Disabled`] when unconfigured, [`AssistError::CapabilityDenied`]
    /// when `search-semantic` is not granted, [`AssistError::ScopeExceeded`] when no
    /// account in `scope` survives the deployment ceiling,
    /// [`AssistError::RateLimited`], or an [`AssistError::Endpoint`] transport error.
    pub async fn embed(&self, scope: DataScope, text: &str) -> Result<Vec<f32>> {
        let adapter = self.require_adapter()?; // enabled + adapter present
        self.check_capability(AssistCapability::SearchSemantic)?; // 1. capability granted
        let eff = scope.clamp(&self.config.data_ceiling); // 2. data-class ceiling
        // ...and the ceiling is ENFORCED here, not merely recorded. `accounts` is a
        // strict allowlist (`intersect_ids`), so an empty effective set means every
        // account this call names is outside the deployment ceiling. Dispatching
        // anyway — as this path did before 26.19 — sends mail content the ceiling
        // excludes and writes an audit row saying `accounts=0`, which reads as
        // "nothing was in scope" rather than "the request went anyway". Refusing
        // before the audit keeps the audit trail a record of what actually left.
        if eff.accounts.is_empty() {
            return Err(AssistError::ScopeExceeded(
                "no account in this call is within the deployment data-class ceiling".to_string(),
            ));
        }
        self.check_rate(&eff)?; // 3. rate-limit
        self.audit(AssistCapability::SearchSemantic, &eff, adapter.host()); // 4. audit
        adapter.embed(text).await // 5. dispatch
    }

    /// Speech-to-text for the Dictation slot (§14, Whisper-compatible). Audio is
    /// user-provided, not mailbox content; same enforcement pipeline.
    pub async fn transcribe(&self, scope: DataScope, audio: &[u8], mime: &str) -> Result<String> {
        let adapter = self.require_adapter()?;
        self.check_capability(AssistCapability::Dictation)?;
        let eff = scope.clamp(&self.config.data_ceiling);
        self.check_rate(&eff)?;
        self.audit(AssistCapability::Dictation, &eff, adapter.host());
        adapter.transcribe(audio, mime).await
    }
}

#[cfg(test)]
mod tests;

/// 26.19 (t19-e16): the ceiling-enforcement and per-account rate-limit fixes.
///
/// Inline rather than appended to `tests.rs` because that file is outside this lane's
/// file locks. The coverage belongs with the code it pins either way.
#[cfg(test)]
mod enforcement_tests {
    use super::*;
    use async_trait::async_trait;
    use futures_util::StreamExt;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Counts dispatches. "Was it refused?" and "did it go out anyway?" are different
    /// questions, and only the second one is about egress.
    #[derive(Default)]
    struct CountingAdapter {
        embeds: AtomicUsize,
    }

    #[async_trait]
    impl EndpointAdapter for CountingAdapter {
        async fn chat(&self, _payload: &ChatPayload) -> Result<ChatStream> {
            Ok(futures_util::stream::empty::<Result<StreamChunk>>().boxed())
        }
        async fn embed(&self, _input: &str) -> Result<Vec<f32>> {
            self.embeds.fetch_add(1, Ordering::SeqCst);
            Ok(vec![0.25, 0.5])
        }
        async fn transcribe(&self, _audio: &[u8], _mime: &str) -> Result<String> {
            Ok(String::new())
        }
        fn host(&self) -> String {
            "endpoint.test".into()
        }
    }

    /// A gateway whose ceiling admits exactly `accounts`.
    fn gateway(
        accounts: &[&str],
        rate_limit_per_min: Option<u32>,
    ) -> (AssistGateway, Arc<CountingAdapter>, Arc<InMemoryAudit>) {
        let adapter = Arc::new(CountingAdapter::default());
        let audit = Arc::new(InMemoryAudit::default());
        let gw = AssistGateway::new(AssistConfig {
            enabled: true,
            capability_grants: vec![AssistCapability::SearchSemantic],
            data_ceiling: DataScope {
                accounts: accounts.iter().map(|s| (*s).to_string()).collect(),
                ..DataScope::default()
            },
            adapter: None,
            rate_limit_per_min,
        })
        .with_adapter(adapter.clone())
        .with_audit(audit.clone());
        (gw, adapter, audit)
    }

    fn for_account(id: &str) -> DataScope {
        DataScope {
            accounts: vec![id.to_string()],
            ..DataScope::default()
        }
    }

    /// **Fails against the pre-fix code**, on the first assertion: `embed` clamped the
    /// scope and then used it only to label the audit row, so an account outside the
    /// ceiling returned `Ok` with the text already dispatched, leaving an audit row
    /// reading `accounts=0` — which reads as "nothing was in scope", not as "it went".
    ///
    /// The second half is the paired positive control, in the same test on purpose: a
    /// gateway that refused everything (or was never wired to an adapter) would satisfy
    /// the refusal assertions while proving nothing.
    #[tokio::test]
    async fn embed_refuses_an_account_outside_the_ceiling_and_serves_one_inside_it() {
        let (gw, adapter, audit) = gateway(&["allowed"], None);

        let denied = gw.embed(for_account("intruder"), "quarterly report").await;
        assert!(
            matches!(denied, Err(AssistError::ScopeExceeded(_))),
            "an account outside the ceiling must be refused, got {denied:?}"
        );
        assert_eq!(
            adapter.embeds.load(Ordering::SeqCst),
            0,
            "a refused call must never reach the endpoint"
        );
        assert!(
            audit.rows().is_empty(),
            "nothing left, so nothing is recorded as having left: {:?}",
            audit.rows()
        );

        // Positive control: the permitted account is served, so the assertions above
        // are about enforcement rather than about a gateway that does nothing.
        let allowed = gw.embed(for_account("allowed"), "quarterly report").await;
        assert_eq!(
            allowed.expect("in-ceiling embed dispatches"),
            vec![0.25, 0.5]
        );
        assert_eq!(adapter.embeds.load(Ordering::SeqCst), 1);
        let rows = audit.rows();
        assert_eq!(rows.len(), 1, "exactly one dispatch, exactly one audit row");
        assert!(
            rows[0].scope_summary.contains("accounts=1"),
            "the row describes the call that actually went: {:?}",
            rows[0].scope_summary
        );
    }

    /// An empty per-call scope is refused too — the "forgot to populate the scope"
    /// shape, which the strict-allowlist intersection turns into an empty effective
    /// set exactly like an out-of-ceiling account.
    ///
    /// **Fails against the pre-fix code**: this dispatched.
    #[tokio::test]
    async fn embed_refuses_an_empty_scope() {
        let (gw, adapter, _audit) = gateway(&["allowed"], None);
        assert!(matches!(
            gw.embed(DataScope::default(), "text").await,
            Err(AssistError::ScopeExceeded(_))
        ));
        assert_eq!(adapter.embeds.load(Ordering::SeqCst), 0);
    }

    /// **Fails against the pre-fix code** on the final assertion: the window was a
    /// single gateway-wide bucket, so account `b` inherited account `a`'s spend and was
    /// refused a request it had every right to make. With a cold-cache semantic search
    /// costing up to 33 outbound requests, that is one user locking the whole
    /// deployment out of Assist for the rest of the minute.
    #[tokio::test]
    async fn the_rate_limit_is_per_account_not_gateway_wide() {
        let (gw, adapter, _audit) = gateway(&["a", "b"], Some(1));

        assert!(gw.embed(for_account("a"), "one").await.is_ok());
        assert!(
            matches!(
                gw.embed(for_account("a"), "two").await,
                Err(AssistError::RateLimited)
            ),
            "the account that spent its budget is the account that is bounded"
        );
        assert_eq!(
            adapter.embeds.load(Ordering::SeqCst),
            1,
            "a rate-limited call must never reach the endpoint"
        );

        assert!(
            gw.embed(for_account("b"), "one").await.is_ok(),
            "a second account has its own budget: one user's search must not deny \
             Assist to everyone else"
        );
        assert_eq!(adapter.embeds.load(Ordering::SeqCst), 2);

        // The budget key must not depend on the order the caller listed the accounts
        // in, or the same account set could be charged to two buckets and quietly get
        // double the budget — which would make the per-account window a way to evade
        // the limit rather than to scope it.
        let ba = DataScope {
            accounts: vec!["b".into(), "a".into()],
            ..DataScope::default()
        };
        let ab = DataScope {
            accounts: vec!["a".into(), "b".into()],
            ..DataScope::default()
        };
        assert_eq!(rate_key(&ba), rate_key(&ab));
        assert_ne!(rate_key(&ba), rate_key(&for_account("a")));
    }
}
