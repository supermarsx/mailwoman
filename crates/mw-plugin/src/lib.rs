#![forbid(unsafe_code)]
//! `mw-plugin` — the WASM engine-plugin host (plan §2.1, SPEC §22/§7.5).
//!
//! A **wasmtime + WASI-p2 component-model** host that loads manifest-declared,
//! admin-approved, **capability-gated** plugins. It is the milestone's security
//! core: the wasmtime jail is the §7.1 **capability boundary**, not an afterthought.
//!
//! - **Capabilities are deny-by-default.** A plugin's [`PluginManifest`] declares
//!   the [`Capability`]s and `net_allowlist` it needs; the effective grant is the
//!   *intersection* of the manifest and the admin [`Grant`]. Nothing undeclared is
//!   reachable: the guest gets a maximally-restricted WASI ctx (no fs/net/env/stdio)
//!   and every real capability flows through a gated host import ([`host_state`]).
//! - **Resource-limited.** A memory ceiling (a [`wasmtime::ResourceLimiter`]), a
//!   wall-clock CPU deadline (epoch-interruption) + optional fuel; a trip is a typed
//!   [`PluginError::LimitExceeded`], **never a panic** — the host survives.
//! - **Signed registry.** A detached Ed25519 signature over the component bytes is
//!   verified on load ([`signature`]); unsigned components load **only** under an
//!   admin `allow_unsigned` policy and flag the handle so the UI/[`doctor`] raises a
//!   persistent banner + audit signal.
//!
//! The frozen WIT ABI lives in [`wit/`](../wit) — the `mailwoman:plugin` world is
//! the contract e10/e11/e12/e13 build against.
//!
//! This crate depends on `mw-engine` ONLY for the frozen
//! [`mw_engine::AccountBackend`] trait a plugin's `account-backend` export is adapted
//! onto ([`PluginHandle::as_account_backend`]). `mw-engine` never depends on this
//! crate (no cycle); `mw-server` wires a loaded backend into the engine at mount
//! (e14), mirroring how `mw-imap` is constructed today.

use std::collections::BTreeSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wasmtime::Engine;
use wasmtime::component::{Component, Linker};

mod adapter;
mod bindings;
pub mod doctor;
mod engine;
mod host_state;
pub mod signature;

pub use host_state::{
    Clock, HostServices, HttpFetcher, HttpReq, HttpResp, KvStore, OAuthTokenProvider, Rng,
};
pub use signature::{SignatureStatus, TrustRoot};

use host_state::{CapGate, HostState, build_linker};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    /// Linear-memory ceiling (MiB), enforced via a [`wasmtime::ResourceLimiter`].
    pub memory_mb: u32,
    /// CPU deadline (ms), enforced via epoch-interruption.
    pub deadline_ms: u64,
    /// Optional deterministic fuel budget (in addition to the deadline).
    pub fuel: Option<u64>,
}

impl Default for PluginLimits {
    fn default() -> Self {
        // Conservative deny-leaning defaults; per-hook tuning at grant time.
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
    /// Hex-encoded detached Ed25519 signature over the component bytes; `None` ⇒
    /// unsigned (loads only under `allow_unsigned`).
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

/// Compiled + verified plugin context shared by a handle and its adapters.
struct PluginCtx {
    engine: Engine,
    component: Component,
    linker: Linker<HostState>,
    gate: CapGate,
    services: Arc<HostServices>,
    limits: PluginLimits,
    plugin_id: String,
    granted: BTreeSet<Capability>,
    signature: SignatureStatus,
}

/// A loaded, verified plugin. Compilation + signature verification happen at
/// [`PluginHost::load`]; a wasmtime *instance* is created lazily per session/call
/// (deny-by-default + instance recycling, plan §2.1).
pub struct PluginHandle {
    ctx: Arc<PluginCtx>,
}

impl PluginHandle {
    /// The plugin id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.ctx.plugin_id
    }

    /// Whether this plugin loaded **unsigned** (⇒ the UI must show a persistent
    /// banner and the host emits an audit signal, §7.5). Never silent.
    #[must_use]
    pub fn is_unsigned(&self) -> bool {
        self.ctx.signature == SignatureStatus::UnsignedAllowed
    }

    /// The effective granted capabilities (`manifest ∩ admin grant`).
    #[must_use]
    pub fn granted(&self) -> Vec<Capability> {
        self.ctx.granted.iter().copied().collect()
    }

    /// Adapt this plugin's `account-backend` export onto the engine's frozen
    /// [`mw_engine::AccountBackend`] trait — but only if `account-backend` was
    /// granted. `mw-server` hands the returned `Arc` to the engine at mount (e14),
    /// where a plugin backend is indistinguishable from `mw-imap`.
    #[must_use]
    pub fn as_account_backend(&self) -> Option<Arc<dyn mw_engine::AccountBackend>> {
        if !self.ctx.granted.contains(&Capability::AccountBackend) {
            return None;
        }
        Some(Arc::new(adapter::AccountBackendAdapter::new(
            self.ctx.clone(),
        )))
    }

    /// Invoke the `dlp-detect` hook (§10.8). Requires the `dlp-detector` capability.
    pub async fn call_dlp_detect(&self, body: Vec<u8>) -> Result<Vec<String>> {
        self.require(Capability::DlpDetector)?;
        let mut s = self.ctx.instantiate().await?;
        let g = s.plugin.mailwoman_plugin_dlp_detect();
        match g.call_detect(&mut s.store, &body).await {
            Ok(inner) => inner.map_err(adapter::wit_to_plugin_err),
            Err(e) => Err(host_state::map_call_err(&s.store, e)),
        }
    }

    /// Invoke the `addrbook-source` hook (§13). Requires `addrbook-source`; the
    /// guest's outbound HTTP (if any) is separately gated by `net` + `net_allowlist`.
    pub async fn call_addrbook_search(&self, query: String) -> Result<Vec<String>> {
        self.require(Capability::AddrbookSource)?;
        let mut s = self.ctx.instantiate().await?;
        let g = s.plugin.mailwoman_plugin_addrbook_source();
        match g.call_search(&mut s.store, &query).await {
            Ok(inner) => inner.map_err(adapter::wit_to_plugin_err),
            Err(e) => Err(host_state::map_call_err(&s.store, e)),
        }
    }

    /// Invoke the `message-out` pipeline hook (§22). Requires `message-pipeline`.
    pub async fn call_message_out(&self, raw: Vec<u8>) -> Result<Vec<u8>> {
        self.require(Capability::MessagePipeline)?;
        let mut s = self.ctx.instantiate().await?;
        let g = s.plugin.mailwoman_plugin_message_pipeline();
        match g.call_message_out(&mut s.store, &raw).await {
            Ok(inner) => inner.map_err(adapter::wit_to_plugin_err),
            Err(e) => Err(host_state::map_call_err(&s.store, e)),
        }
    }

    fn require(&self, cap: Capability) -> Result<()> {
        if self.ctx.granted.contains(&cap) {
            Ok(())
        } else {
            Err(PluginError::CapabilityDenied(format!(
                "{cap:?} not granted"
            )))
        }
    }
}

impl PluginCtx {
    /// Instantiate a fresh, resource-limited store + component instance (async).
    async fn instantiate(self: &Arc<Self>) -> Result<adapter::GuestSession> {
        let mut store = host_state::new_store(
            &self.engine,
            self.gate.clone(),
            self.services.clone(),
            &self.limits,
            self.plugin_id.clone(),
        )?;
        let plugin = bindings::Plugin::instantiate_async(&mut store, &self.component, &self.linker)
            .await
            .map_err(|e| host_state::map_call_err(&store, e))?;
        Ok(adapter::GuestSession { store, plugin })
    }
}

/// The plugin host + in-memory registry (plan §2.1). Consumed by e8 (engine) +
/// e14 (mount). Owns the wasmtime `Engine`, the epoch ticker, the injected host
/// services, and the signing trust root.
pub struct PluginHost {
    engine: Engine,
    _epoch: engine::EpochTicker,
    services: Arc<HostServices>,
    trust: TrustRoot,
    registry: Vec<RegistryEntry>,
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginHost {
    /// Construct a host with default (deny-leaning) services and an empty trust
    /// root. Panics only if the wasmtime engine cannot initialize (effectively
    /// never); use [`PluginHost::try_new`] to handle that explicitly.
    #[must_use]
    pub fn new() -> Self {
        Self::try_new(HostServices::default(), TrustRoot::empty())
            .expect("wasmtime engine initialization")
    }

    /// Fallible constructor with explicit services + trust root.
    pub fn try_new(services: HostServices, trust: TrustRoot) -> Result<Self> {
        let engine = engine::build_engine()?;
        let epoch = engine::EpochTicker::spawn(engine.clone());
        Ok(Self {
            engine,
            _epoch: epoch,
            services: Arc::new(services),
            trust,
            registry: Vec::new(),
        })
    }

    /// Replace the injected host services (HTTP/OAuth/KV/clock/rng). `mw-server`
    /// calls this at mount to wire its `reqwest`(rustls) client + OAuth store (e14).
    pub fn set_services(&mut self, services: HostServices) {
        self.services = Arc::new(services);
    }

    /// Set the signing trust root used to verify component signatures.
    pub fn set_trust_root(&mut self, trust: TrustRoot) {
        self.trust = trust;
    }

    /// Load + verify a component under a capability grant (plan §2.1).
    ///
    /// Verifies the detached signature (or requires `grant.allow_unsigned`),
    /// intersects `granted ⊆ manifest` capabilities, compiles the component, and
    /// prepares the linker. **No instantiation happens here** — a resource-limited
    /// instance is created lazily per session/call (deny-by-default + recycling).
    pub fn load(
        &self,
        component_bytes: &[u8],
        manifest: &PluginManifest,
        grant: &Grant,
    ) -> Result<PluginHandle> {
        // 1. Signed-registry verification (fails closed).
        let signature = signature::decide(
            component_bytes,
            manifest.signature.as_deref(),
            &self.trust,
            grant.allow_unsigned,
        )?;

        // 2. Effective grant = manifest ∩ admin grant (never widened).
        let manifest_caps: BTreeSet<Capability> = manifest.capabilities.iter().copied().collect();
        let admin_caps: BTreeSet<Capability> = grant.capabilities.iter().copied().collect();
        let granted: BTreeSet<Capability> =
            manifest_caps.intersection(&admin_caps).copied().collect();

        // 3. Compile the component (sync; the expensive step).
        let component = Component::new(&self.engine, component_bytes)
            .map_err(|e| PluginError::Load(format!("component compile: {e}")))?;
        let linker = build_linker(&self.engine)?;

        let gate = CapGate::new(granted.clone(), manifest.net_allowlist.clone());
        let ctx = Arc::new(PluginCtx {
            engine: self.engine.clone(),
            component,
            linker,
            gate,
            services: self.services.clone(),
            limits: manifest.limits,
            plugin_id: manifest.id.clone(),
            granted,
            signature,
        });
        Ok(PluginHandle { ctx })
    }

    /// Registry: enumerate known plugins.
    #[must_use]
    pub fn list(&self) -> &[RegistryEntry] {
        &self.registry
    }

    /// Registry: register a plugin manifest (enabled=false, unapproved).
    pub fn register(&mut self, manifest: PluginManifest) {
        if self.registry.iter().any(|e| e.manifest.id == manifest.id) {
            return;
        }
        self.registry.push(RegistryEntry {
            manifest,
            approved_by: None,
            enabled: false,
        });
    }

    /// Registry: admin-approve a plugin.
    pub fn approve(&mut self, plugin_id: &str, admin: &str) -> Result<()> {
        let e = self.entry_mut(plugin_id)?;
        e.approved_by = Some(admin.to_string());
        Ok(())
    }

    /// Registry: enable a plugin (must be approved first).
    pub fn enable(&mut self, plugin_id: &str) -> Result<()> {
        let e = self.entry_mut(plugin_id)?;
        if e.approved_by.is_none() {
            return Err(PluginError::CapabilityDenied(
                "cannot enable an unapproved plugin".into(),
            ));
        }
        e.enabled = true;
        Ok(())
    }

    /// Registry: disable a plugin.
    pub fn disable(&mut self, plugin_id: &str) -> Result<()> {
        self.entry_mut(plugin_id)?.enabled = false;
        Ok(())
    }

    fn entry_mut(&mut self, plugin_id: &str) -> Result<&mut RegistryEntry> {
        self.registry
            .iter_mut()
            .find(|e| e.manifest.id == plugin_id)
            .ok_or_else(|| PluginError::Manifest(format!("unknown plugin '{plugin_id}'")))
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
        let empty: PluginManifest =
            serde_json::from_str(r#"{"id":"x","name":"x","version":"0"}"#).unwrap();
        assert!(empty.capabilities.is_empty());
        assert!(empty.net_allowlist.is_empty());
    }

    #[test]
    fn registry_approve_enable_flow() {
        let mut host = PluginHost::new();
        let m = PluginManifest {
            id: "lt".into(),
            name: "LanguageTool".into(),
            version: "1".into(),
            signature: None,
            capabilities: vec![],
            net_allowlist: vec![],
            limits: PluginLimits::default(),
        };
        host.register(m);
        assert!(host.enable("lt").is_err(), "must approve before enable");
        host.approve("lt", "admin@x").unwrap();
        host.enable("lt").unwrap();
        assert!(host.list()[0].enabled);
    }
}
