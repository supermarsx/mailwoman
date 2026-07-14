//! V7 additive wiring (plan §3 e8; SPEC §6.5/§13/§14/§22): plugin-backed account
//! backends, the **GAL address-book source** for recipient resolution, the
//! **Assist / MCP tool-surface** hooks, and the **bridge optional-capability**
//! preference for reactions/voting/recall/Focused-Inbox-sync (standards fallback).
//!
//! ## Inert by default (hard regression gate)
//! Everything here is **inert** until e14 (MOUNT) calls [`Engine::attach_v7`] and/or
//! [`Engine::register_plugin_backend`]. With nothing attached:
//! - [`Engine::resolve_recipients`] / group-expand / cert-lookup return empty (no
//!   directory) — the composer's existing recipient path is unchanged;
//! - [`Engine::bridge_reactions`] & friends return `None`, so every caller takes its
//!   existing standards fallback (byte-unchanged, per §1.6);
//! - [`Engine::assist`] returns `None`, so the web hides all Assist UI.
//!
//! The non-plugin / non-directory / SQLite-default path is therefore byte-for-byte
//! the V6 behaviour (the e8 hard regression gate).
//!
//! ## Cycle-free injection (plan §1 hard constraint)
//! `mw-plugin` depends on `mw-engine` for the frozen [`AccountBackend`] trait, so the
//! engine **must not** name `mw-plugin` (that would be a dependency cycle). Instead:
//! - **Plugin backends** arrive as a plain [`Arc<dyn AccountBackend>`] (exactly what
//!   `mw_plugin::PluginHandle::as_account_backend()` returns) and are registered
//!   through the ordinary account dispatcher — a plugin backend is *indistinguishable*
//!   from `mw-imap` to the engine. [`Engine::register_plugin_backend`] adds only a
//!   bookkeeping tag so doctor/introspection can report which accounts are bridge-backed.
//! - **Assist / MCP** are reached through the engine-native [`AssistHook`] trait
//!   object; e14 backs it with the real `mw_assist::AssistGateway` + the `mw-mcp`
//!   tool surface (the assistant is an unprivileged MCP client). Keeping the seam
//!   engine-native keeps the Assist/reqwest/streaming tree out of the engine.
//! - **Bridge caps** are engine-native traits injected via [`BridgeCapabilitySource`].
//!
//! Only `mw-directory` is a direct dependency — the mandate is to "use the frozen
//! `Directory` API", and `mw-directory` does not depend on `mw-engine` (no cycle).

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// The frozen GAL surface (plan §2.2) the engine + e9/e14 speak in — re-exported so
// the mount site and the engine cannot drift on the address-book contract.
pub use mw_directory::{DirectorySource, GalEntry};

use crate::backend::{AccountBackend, EngineError, MessageRef, Result};
use crate::engine::Engine;

// ─── Bridge optional capabilities (plan §2.5, §1.6) ──────────────────────────────
//
// The frozen `AccountBackend` trait + `BackendCaps` are unchanged. Bridge-native
// Outlook-parity features are **additive** capability traits the engine PREFERS when
// a backend advertises them; absence ⇒ the existing standards fallback path runs
// unchanged. Each is object-safe (via `async_trait`) so a bridge can hand the engine
// an `Arc<dyn Bridge*>` at mount.

/// One reaction on a message: who reacted and with what emoji.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeReaction {
    pub actor: String,
    pub emoji: String,
}

/// One voting-button tally line for a sent voting message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeVoteTally {
    pub option: String,
    pub count: u32,
}

/// The honest outcome of a bridge message-recall attempt (§10.3 recall matrix —
/// never claims more than the backend actually delivers).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallOutcome {
    /// The recall request was accepted by the server (best-effort; recipients may
    /// still have read the message — the UI must say so honestly).
    Requested,
    /// The server does not support recall for this message.
    Unsupported,
    /// The recall was attempted and refused.
    Failed { reason: String },
}

/// Focused-Inbox classification (Graph/EWS Focused vs Other).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusedState {
    Focused,
    Other,
}

/// Bridge-native reactions (Graph/Teams-style). Preferred over the header-convention
/// fallback when advertised.
#[async_trait]
pub trait BridgeReactions: Send + Sync {
    /// Add (`add=true`) or remove the caller's `emoji` reaction on a message.
    async fn set_reaction(&self, msg: &MessageRef, emoji: &str, add: bool) -> Result<()>;
    /// List the reactions currently on a message.
    async fn get_reactions(&self, msg: &MessageRef) -> Result<Vec<BridgeReaction>>;
}

/// Bridge-native Outlook voting buttons.
#[async_trait]
pub trait BridgeVoting: Send + Sync {
    /// Cast a voting-button response on a received voting message.
    async fn cast_vote(&self, msg: &MessageRef, option: &str) -> Result<()>;
    /// Tally responses for a sent voting message.
    async fn tally(&self, msg: &MessageRef) -> Result<Vec<BridgeVoteTally>>;
}

/// Bridge-native message recall.
#[async_trait]
pub trait BridgeRecall: Send + Sync {
    /// Attempt to recall a sent message; returns the honest [`RecallOutcome`].
    async fn recall(&self, msg: &MessageRef) -> Result<RecallOutcome>;
}

/// Bridge-native Focused-Inbox sync (bidirectional).
#[async_trait]
pub trait BridgeFocusedSync: Send + Sync {
    /// Read a message's Focused/Other classification from upstream.
    async fn focused_state(&self, msg: &MessageRef) -> Result<FocusedState>;
    /// Move a message between Focused and Other, syncing the choice upstream.
    async fn set_focused(&self, msg: &MessageRef, focused: bool) -> Result<()>;
}

/// Which optional bridge capabilities a backend advertises for an account. All
/// `false` (the default) ⇒ the engine takes every standards fallback.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BridgeCaps {
    pub reactions: bool,
    pub voting: bool,
    pub recall: bool,
    pub focused_sync: bool,
}

/// The seam e14 injects so the engine can discover, per account, which bridge
/// optional capabilities a plugin/bridge backend advertises and obtain the trait
/// object that implements each. Returning `None` for a capability (the default when
/// no source is attached, or for a plain IMAP/POP3 account) tells the engine to use
/// the existing standards fallback — byte-unchanged.
pub trait BridgeCapabilitySource: Send + Sync {
    /// The capabilities advertised for `account_id` (drives the doctor line + UI).
    fn caps(&self, account_id: &str) -> BridgeCaps;
    fn reactions(&self, account_id: &str) -> Option<Arc<dyn BridgeReactions>>;
    fn voting(&self, account_id: &str) -> Option<Arc<dyn BridgeVoting>>;
    fn recall(&self, account_id: &str) -> Option<Arc<dyn BridgeRecall>>;
    fn focused_sync(&self, account_id: &str) -> Option<Arc<dyn BridgeFocusedSync>>;
}

// ─── Assist / MCP tool-surface seam (plan §2.4, SPEC §14) ────────────────────────

/// The engine-side Assist seam. e14 backs this with the real
/// `mw_assist::AssistGateway` (which delegates the `Assistant` capability to the
/// `mw-mcp` tool registry — an unprivileged client, inheriting `mail.send`→Outbox
/// gating). The engine holds it so the assist/JMAP surface reaches Assist *through*
/// the engine, inheriting account scope; **only content-free posture crosses this
/// seam** (no mail body/subject/address — the §21.1 no-content invariant).
pub trait AssistHook: Send + Sync {
    /// Whether Assist is configured + enabled (the web hides all UI when `false`).
    fn is_enabled(&self) -> bool;
    /// The capabilities granted at the active scope, as their kebab-case wire names
    /// (content-free; used only to gate which Assist affordances the UI shows). None
    /// of these is a send/delete/accept capability — Assist can never transmit.
    fn granted_capabilities(&self) -> Vec<String>;
}

// ─── The additive V7 hook bundle ─────────────────────────────────────────────────

/// The additive V7 hook bundle attached by e14 (MOUNT). Cheaply cloneable — every
/// injected seam is an `Arc` trait object and the plugin-backing map is small.
#[derive(Clone, Default)]
pub struct V7Hooks {
    directory: Option<Arc<dyn DirectorySource>>,
    bridge_caps: Option<Arc<dyn BridgeCapabilitySource>>,
    assist: Option<Arc<dyn AssistHook>>,
    /// account_id → plugin/bridge id, for accounts registered via
    /// [`Engine::register_plugin_backend`]. Preserved across [`Engine::attach_v7`].
    plugin_backends: BTreeMap<String, String>,
}

impl V7Hooks {
    /// The inert default: no directory, no bridge caps, no assist, no plugin
    /// backends.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach the GAL address-book source (e14 → `mw_directory::Directory` over the
    /// 0008 `directory_config`).
    #[must_use]
    pub fn with_directory(mut self, dir: Arc<dyn DirectorySource>) -> Self {
        self.directory = Some(dir);
        self
    }

    /// Attach the bridge optional-capability source (e14 → the loaded bridge plugins).
    #[must_use]
    pub fn with_bridge_caps(mut self, caps: Arc<dyn BridgeCapabilitySource>) -> Self {
        self.bridge_caps = Some(caps);
        self
    }

    /// Attach the Assist hook (e14 → `mw_assist::AssistGateway` + the MCP tool surface).
    #[must_use]
    pub fn with_assist(mut self, assist: Arc<dyn AssistHook>) -> Self {
        self.assist = Some(assist);
        self
    }
}

impl Engine {
    /// Attach the V7 hooks (GAL directory / bridge caps / assist). Called by e14
    /// (MOUNT) after the engine is built. Idempotent-by-replacement for the injected
    /// seams; the plugin-backing registry (populated by
    /// [`Engine::register_plugin_backend`]) is **preserved** across calls.
    pub fn attach_v7(&self, hooks: V7Hooks) {
        let mut w = self.v7.write().expect("v7 hooks lock");
        w.directory = hooks.directory;
        w.bridge_caps = hooks.bridge_caps;
        w.assist = hooks.assist;
        // Any plugin backings the caller pre-seeded on the bundle are merged in;
        // existing registrations always win and are never dropped by a re-attach.
        for (acct, plugin) in hooks.plugin_backends {
            w.plugin_backends.entry(acct).or_insert(plugin);
        }
    }

    /// A cheap clone of the current V7 hook bundle.
    pub(crate) fn v7_hooks(&self) -> V7Hooks {
        self.v7.read().expect("v7 hooks lock").clone()
    }

    // ── Plugin-backed accounts (plan §3 e8) ──────────────────────────────────────

    /// Register a **plugin/bridge-provided** account backend into the ordinary
    /// account dispatcher (plan §6.5): a plugin backend is `Arc<dyn AccountBackend>`
    /// exactly like `mw-imap`, so it flows through [`Engine::register_backend`] and
    /// is served by the same sync/JMAP paths — indistinguishable to the engine. The
    /// `plugin_id` is recorded only for doctor/introspection.
    pub fn register_plugin_backend(
        &self,
        account_id: impl Into<String>,
        plugin_id: impl Into<String>,
        runtime: crate::account::AccountRuntime,
    ) {
        let account_id = account_id.into();
        self.register_backend(&account_id, runtime);
        self.v7
            .write()
            .expect("v7 hooks lock")
            .plugin_backends
            .insert(account_id, plugin_id.into());
    }

    /// Whether `account_id` is served by a plugin/bridge backend.
    #[must_use]
    pub fn is_plugin_backed(&self, account_id: &str) -> bool {
        self.v7
            .read()
            .expect("v7 hooks lock")
            .plugin_backends
            .contains_key(account_id)
    }

    /// The plugin/bridge id backing `account_id`, if any (doctor/introspection).
    #[must_use]
    pub fn plugin_backend_id(&self, account_id: &str) -> Option<String> {
        self.v7
            .read()
            .expect("v7 hooks lock")
            .plugin_backends
            .get(account_id)
            .cloned()
    }

    // ── GAL address-book source (plan §2.2 / SPEC §13) ───────────────────────────

    /// Whether a GAL directory is attached (drives the doctor line + tests).
    #[must_use]
    pub fn directory_attached(&self) -> bool {
        self.v7.read().expect("v7 hooks lock").directory.is_some()
    }

    /// Resolve recipients from the GAL for the composer's address-book (SPEC §13):
    /// search every recipient field for `query`, page `page` (0-based). Returns
    /// `Ok(vec![])` when **no directory is attached** or none is configured — the
    /// non-directory default path, byte-unchanged (the composer keeps its existing
    /// local-contact resolution; GAL only *adds* entries).
    pub async fn resolve_recipients(&self, query: &str, page: u32) -> Result<Vec<GalEntry>> {
        let Some(dir) = self.v7_hooks().directory else {
            return Ok(Vec::new());
        };
        match dir.search_gal(query, page).await {
            Ok(entries) => Ok(entries),
            Err(mw_directory::DirectoryError::NotConfigured) => Ok(Vec::new()),
            Err(e) => Err(dir_err(e)),
        }
    }

    /// Expand a distribution group DN to its members before send (§13
    /// "expand-before-send" — "who is actually in this?"). Empty when no directory.
    pub async fn expand_group(&self, dn: &str) -> Result<Vec<GalEntry>> {
        let Some(dir) = self.v7_hooks().directory else {
            return Ok(Vec::new());
        };
        match dir.expand_group(dn).await {
            Ok(entries) => Ok(entries),
            Err(mw_directory::DirectoryError::NotConfigured) => Ok(Vec::new()),
            Err(e) => Err(dir_err(e)),
        }
    }

    /// S/MIME certificate lookup for a recipient via the GAL (feeds `mw-crypto`'s
    /// cert path, §8.2). Empty when no directory is attached.
    pub async fn gal_lookup_cert(&self, email: &str) -> Result<Vec<Vec<u8>>> {
        let Some(dir) = self.v7_hooks().directory else {
            return Ok(Vec::new());
        };
        match dir.lookup_cert(email).await {
            Ok(ders) => Ok(ders),
            Err(mw_directory::DirectoryError::NotConfigured) => Ok(Vec::new()),
            Err(e) => Err(dir_err(e)),
        }
    }

    // ── Bridge optional-capability preference (plan §2.5, §1.6) ──────────────────

    /// The bridge capabilities advertised for `account_id` (all `false` ⇒ standards
    /// fallback everywhere). Used for the doctor line + UI affordance gating.
    #[must_use]
    pub fn bridge_caps(&self, account_id: &str) -> BridgeCaps {
        self.v7
            .read()
            .expect("v7 hooks lock")
            .bridge_caps
            .as_ref()
            .map(|s| s.caps(account_id))
            .unwrap_or_default()
    }

    /// The bridge-native reactions impl for `account_id` when advertised, else
    /// `None` → the caller uses the existing header-convention fallback (byte-unchanged).
    #[must_use]
    pub fn bridge_reactions(&self, account_id: &str) -> Option<Arc<dyn BridgeReactions>> {
        self.v7
            .read()
            .expect("v7 hooks lock")
            .bridge_caps
            .as_ref()
            .and_then(|s| s.reactions(account_id))
    }

    /// The bridge-native voting impl for `account_id` when advertised, else `None`.
    #[must_use]
    pub fn bridge_voting(&self, account_id: &str) -> Option<Arc<dyn BridgeVoting>> {
        self.v7
            .read()
            .expect("v7 hooks lock")
            .bridge_caps
            .as_ref()
            .and_then(|s| s.voting(account_id))
    }

    /// The bridge-native recall impl for `account_id` when advertised, else `None`.
    #[must_use]
    pub fn bridge_recall(&self, account_id: &str) -> Option<Arc<dyn BridgeRecall>> {
        self.v7
            .read()
            .expect("v7 hooks lock")
            .bridge_caps
            .as_ref()
            .and_then(|s| s.recall(account_id))
    }

    /// The bridge-native Focused-sync impl for `account_id` when advertised, else `None`.
    #[must_use]
    pub fn bridge_focused_sync(&self, account_id: &str) -> Option<Arc<dyn BridgeFocusedSync>> {
        self.v7
            .read()
            .expect("v7 hooks lock")
            .bridge_caps
            .as_ref()
            .and_then(|s| s.focused_sync(account_id))
    }

    // ── Assist hook (plan §2.4, SPEC §14) ────────────────────────────────────────

    /// The attached Assist hook, if any. `None` ⇒ Assist unconfigured (the web hides
    /// all Assist UI).
    #[must_use]
    pub fn assist(&self) -> Option<Arc<dyn AssistHook>> {
        self.v7.read().expect("v7 hooks lock").assist.clone()
    }

    /// Whether Assist is attached **and** enabled (content-free UI gate).
    #[must_use]
    pub fn assist_enabled(&self) -> bool {
        self.assist().is_some_and(|a| a.is_enabled())
    }
}

/// Map a [`mw_directory::DirectoryError`] onto the engine's error type so GAL
/// operations degrade under the engine's uniform retry/error policy.
fn dir_err(e: mw_directory::DirectoryError) -> EngineError {
    use mw_directory::DirectoryError as D;
    match e {
        D::Protocol(m) => EngineError::Protocol(m),
        D::Auth(m) => EngineError::Auth(m),
        D::Transport(m) => EngineError::Transport(m),
        D::NotConfigured => EngineError::Unsupported("no directory configured".into()),
    }
}

// A compile-time proof that a plugin backend is nothing more than the frozen
// `AccountBackend` trait object — the same type `mw-imap` produces — so the engine
// literally cannot tell them apart (plan §6.5).
const _: fn(Arc<dyn AccountBackend>) = |_b| {};
