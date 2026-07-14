#![forbid(unsafe_code)]
// SCAFFOLD (t7-e0): frozen §2.1 types + method shapes as INERT stubs. The wasmtime
// WASI-p2 host, capability enforcement, resource limits, and signed-registry
// verification are filled by e1. Stub fields/methods read as dead code until then;
// the crate-level allow keeps `clippy -D warnings` green (plan e0 acceptance).
#![allow(dead_code)]
//! `mw-plugin` — the WASM engine-plugin host (plan §2.1, SPEC §22/§7.5).
//!
//! A **wasmtime + WASI-p2 component-model** host that loads manifest-declared,
//! admin-approved, **capability-gated** plugins. It is the milestone's security
//! core: the wasmtime jail is the §7.1 **capability boundary**, not an afterthought.
//!
//! - **Capabilities are deny-by-default.** A plugin's [`PluginManifest`] declares
//!   the [`Capability`]s and `net_allowlist` it needs; nothing undeclared is
//!   granted (no ambient WASI filesystem/clock/random/net).
//! - **Resource-limited.** A memory ceiling (`StoreLimits`), a CPU deadline
//!   (epoch-interruption) + optional fuel; a trip is a typed
//!   [`PluginError::LimitExceeded`], **never a panic** — the host survives.
//! - **Signed registry.** A detached signature over the component bytes is
//!   verified on load; unsigned components load **only** under `allow_unsigned`
//!   admin policy and raise a persistent banner + audit signal.
//!
//! The WIT ABI lives in [`wit/`](../wit) — the `mailwoman:plugin` world is the
//! contract e1/e10/e11/e12/e13 build against. **Frozen at approval**; changes
//! require the coordinator to re-broadcast.
//!
//! This crate depends on `mw-engine` ONLY for the frozen
//! [`mw_engine::AccountBackend`] trait it adapts a plugin's `account-backend`
//! export onto ([`PluginHandle::as_account_backend`]). `mw-engine` never depends on
//! this crate (no cycle); `mw-server` wires a loaded backend into the engine at
//! mount (e14), mirroring how `mw-imap` is constructed today.

use std::sync::Arc;

use mw_engine::AccountBackend;
use serde::{Deserialize, Serialize};

/// Errors at the plugin-host boundary (plan §2.1). Resource-limit / capability
/// violations are TYPED — the host maps a wasmtime trap onto these, never panics.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// The component tripped a resource limit (deadline / memory / fuel). The host
    /// survives; the instance is discarded.
    #[error("plugin resource limit exceeded: {0}")]
    LimitExceeded(String),
    /// The plugin attempted an action outside its granted capabilities
    /// (e.g. a `http-fetch` to a host outside its `net_allowlist`).
    #[error("plugin capability denied: {0}")]
    CapabilityDenied(String),
    /// The detached signature over the component bytes failed to verify, and
    /// `allow_unsigned` is not set.
    #[error("plugin signature invalid or missing (allow_unsigned not set): {0}")]
    SignatureInvalid(String),
    /// The manifest could not be parsed / is inconsistent.
    #[error("plugin manifest error: {0}")]
    Manifest(String),
    /// The component bytes could not be parsed/instantiated by the host.
    #[error("plugin load error: {0}")]
    Load(String),
    /// A host import / guest export call failed at the ABI boundary.
    #[error("plugin runtime error: {0}")]
    Runtime(String),
}

pub type Result<T> = std::result::Result<T, PluginError>;

/// A capability a plugin may declare in its manifest and an admin may grant
/// (plan §2.1). **Deny-by-default**: nothing here is granted implicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    /// Implement the engine account-backend seam (the bridge role, §6.5).
    AccountBackend,
    /// Outbound HTTP restricted to the manifest `net_allowlist` (host-mediated).
    Net,
    /// A DLP detector hook (§10.8).
    DlpDetector,
    /// A spam-action hook (§10.8).
    SpamAction,
    /// An address-book source (§13).
    AddrbookSource,
    /// An autoconfig source.
    AutoconfigSource,
    /// A message in/out pipeline hook (§22).
    MessagePipeline,
    /// A scoped KV scratch namespace in the store.
    StoreKvScoped,
}

/// Resource limits enforced by the host (plan §2.1). A trip → [`PluginError::LimitExceeded`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginLimits {
    /// Linear-memory ceiling (MiB), enforced via wasmtime `StoreLimits`.
    pub memory_mb: u32,
    /// CPU deadline (ms), enforced via epoch-interruption.
    pub deadline_ms: u64,
    /// Optional deterministic fuel budget (in addition to the deadline).
    pub fuel: Option<u64>,
}

impl Default for PluginLimits {
    fn default() -> Self {
        // Conservative deny-leaning defaults; e1 tunes per-hook.
        Self {
            memory_mb: 64,
            deadline_ms: 5_000,
            fuel: None,
        }
    }
}

/// The frozen `plugin.toml` manifest (plan §2.1). **Host defaults DENY everything
/// undeclared.**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    /// Detached signature over the component bytes (base64/hex); `None` ⇒ unsigned.
    #[serde(default)]
    pub signature: Option<String>,
    /// Capabilities the plugin requires (granted only after admin approval).
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    /// Hosts the plugin's `http-fetch` may reach (empty ⇒ no outbound net).
    #[serde(default)]
    pub net_allowlist: Vec<String>,
    /// Resource limits.
    #[serde(default)]
    pub limits: PluginLimits,
}

/// An admin's decision to run a plugin with a set of granted capabilities
/// (plan §2.1). The host intersects this with the manifest — never widening it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
    pub plugin_id: String,
    /// The subset of the manifest's capabilities the admin approved.
    pub capabilities: Vec<Capability>,
    /// Admin identity that approved the grant (for the audit row).
    pub granted_by: String,
    /// Whether an unsigned component may load (⇒ persistent banner + audit).
    #[serde(default)]
    pub allow_unsigned: bool,
}

/// A registry row (mirrors the 0008 `plugins` table, plan §2.7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub manifest: PluginManifest,
    pub approved_by: Option<String>,
    pub enabled: bool,
}

/// A loaded, instantiated plugin (plan §2.1). Owns the wasmtime store/instance in
/// e1; here it is an inert handle.
pub struct PluginHandle {
    id: String,
    granted: Vec<Capability>,
}

impl PluginHandle {
    /// Adapt a plugin's `account-backend` WIT export onto the engine's frozen
    /// [`AccountBackend`] trait, if the plugin declares & was granted that
    /// capability. `mw-server` hands the returned `Arc` to the engine at mount.
    #[must_use]
    pub fn as_account_backend(&self) -> Option<Arc<dyn AccountBackend>> {
        // e1: build a WIT↔trait adapter over the async/streams account-backend
        // interface (plan §2.1 / §6 R1). Stub advertises nothing.
        None
    }

    /// The plugin id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// The plugin host + registry (plan §2.1). Consumed by e8 (engine) + e14 (mount).
#[derive(Default)]
pub struct PluginHost {
    registry: Vec<RegistryEntry>,
}

impl PluginHost {
    /// Construct an empty host. e1 configures the wasmtime `Engine`
    /// (component-model, epoch-interruption, memory `StoreLimits`, Pulley-vs-
    /// Cranelift by build flag) and the signed-registry root key here.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load + instantiate a component under a capability grant (plan §2.1).
    ///
    /// e1: verify the detached signature (or require `grant.allow_unsigned`),
    /// intersect granted ⊆ manifest capabilities, apply resource limits, and
    /// instantiate in the WASI-p2 jail with NO ambient authority.
    pub fn load(
        &self,
        _component_bytes: &[u8],
        _manifest: &PluginManifest,
        _grant: &Grant,
    ) -> Result<PluginHandle> {
        Err(PluginError::Load(
            "mw-plugin host not yet wired (e1)".into(),
        ))
    }

    /// Registry: enumerate known plugins.
    #[must_use]
    pub fn list(&self) -> &[RegistryEntry] {
        &self.registry
    }

    /// Registry: admin-approve a plugin.
    pub fn approve(&mut self, _plugin_id: &str, _admin: &str) -> Result<()> {
        Err(PluginError::Runtime("registry not yet wired (e1)".into()))
    }

    /// Registry: enable a plugin.
    pub fn enable(&mut self, _plugin_id: &str) -> Result<()> {
        Err(PluginError::Runtime("registry not yet wired (e1)".into()))
    }

    /// Registry: disable a plugin.
    pub fn disable(&mut self, _plugin_id: &str) -> Result<()> {
        Err(PluginError::Runtime("registry not yet wired (e1)".into()))
    }

    /// Registry: grant a capability for a plugin (optionally scoped to an account).
    pub fn grant(&mut self, _grant: Grant) -> Result<()> {
        Err(PluginError::Runtime("registry not yet wired (e1)".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_round_trips_and_defaults_deny() {
        let m = PluginManifest {
            id: "bridge-graph".into(),
            name: "Microsoft Graph bridge".into(),
            version: "26.8.0".into(),
            signature: None,
            capabilities: vec![Capability::AccountBackend, Capability::Net],
            net_allowlist: vec!["graph.microsoft.com".into()],
            limits: PluginLimits::default(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: PluginManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
        // Deny-by-default: an empty manifest grants nothing.
        let empty: PluginManifest =
            serde_json::from_str(r#"{"id":"x","name":"x","version":"0"}"#).unwrap();
        assert!(empty.capabilities.is_empty());
        assert!(empty.net_allowlist.is_empty());
    }

    #[test]
    fn host_is_inert_until_e1() {
        let host = PluginHost::new();
        assert!(host.list().is_empty());
        let m = PluginManifest {
            id: "x".into(),
            name: "x".into(),
            version: "0".into(),
            signature: None,
            capabilities: vec![],
            net_allowlist: vec![],
            limits: PluginLimits::default(),
        };
        let g = Grant {
            plugin_id: "x".into(),
            capabilities: vec![],
            granted_by: "admin".into(),
            allow_unsigned: false,
        };
        assert!(host.load(b"", &m, &g).is_err());
    }
}
