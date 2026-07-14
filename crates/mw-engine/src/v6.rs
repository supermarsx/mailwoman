//! V6 additive wiring (plan §3 e10): cache-aside on the header-window /
//! message-body read paths, zero-access ciphertext opacity, and an audit +
//! webhook feed off the existing `StateChange` broadcast.
//!
//! ## Inert by default (hard regression gate)
//! Everything here is **inert** until e11 (MOUNT) attaches a [`Cache`], a
//! posture source, and/or an audit feed via [`Engine::attach_v6`]. With nothing
//! attached the engine's read paths call the store directly (byte-for-byte the
//! V5 behavior) and emit no audit events — the non-zero-access + SQLite-default
//! path is unchanged (plan §3 e10 acceptance).
//!
//! ## Zero-access opacity
//! The engine never decrypts a zero-access row: for a zero-access account the
//! store already holds ciphertext (e1 metadata columns + e6 AAD) and the engine
//! treats it as an opaque blob. The cache-aside helpers route header windows +
//! message bodies through [`Cache::get_derived`] with the account's
//! [`AccountPosture`]; for a [`AccountPosture::ZeroAccess`] account that
//! *structurally* forces the value to per-request scope — it is never placed in
//! the memory, Redis, or store cache tiers (mw-cache enforces this by the
//! [`PlaintextDerived`] type, not operator diligence, plan §1.2).
//!
//! ## No new broadcast plumbing
//! Webhooks are a *second consumer of the existing engine `StateChange`
//! broadcast* (e9, in `mw-server`). This module only adds a synchronous
//! [`AuditFeed`] sink for the semantic events a raw `StateChange` cannot express
//! (which rule fired, which submission was recalled). The engine builds no
//! second broadcast channel of its own (plan §3 e10 scope).

use std::sync::Arc;

use serde::{Deserialize, Serialize};

// Re-export the frozen mw-cache surface the engine + e11 speak in (single source
// of truth, so the mount site and the engine cannot drift).
pub use mw_cache::{AccountPosture, Cache, CacheClass, CacheError, PlaintextDerived};

use crate::backend::{EngineError, Result};
use crate::engine::Engine;

/// Source of an account's cache posture (standard vs zero-access). e11 backs
/// this with the `zeroaccess_accounts` table (§2.1); the default
/// ([`StandardPosture`]) treats every account as [`AccountPosture::Standard`] so
/// the existing path is unchanged.
pub trait AccountPostureSource: Send + Sync {
    /// The posture the cache-aside helpers apply for `account_id`.
    fn posture(&self, account_id: &str) -> AccountPosture;
}

/// The default posture source: every account is a conventional (standard)
/// account. Used until e11 attaches a zero-access-aware source.
#[derive(Debug, Default, Clone, Copy)]
pub struct StandardPosture;

impl AccountPostureSource for StandardPosture {
    fn posture(&self, _account_id: &str) -> AccountPosture {
        AccountPosture::Standard
    }
}

/// A semantic audit / webhook-feed event, emitted at rule executions and
/// submission recalls (plan §3 e10). e11 injects a sink that forwards these to
/// the `mw-admin` append-only audit log + the `mw-server` webhook dispatcher.
///
/// The `detail` payload is deliberately structured metadata only — never a mail
/// body, subject, or address (the §21.1 no-mail-content-in-logs invariant that
/// e9's typed wrappers also enforce).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub account_id: String,
    /// Dotted action verb, e.g. `rule.executed`, `submission.recalled`.
    pub action: String,
    /// The primary object the action touched (a stable id / submission id).
    pub target: Option<String>,
    /// Structured, non-sensitive detail (never mail content).
    pub detail: serde_json::Value,
}

/// Sink for [`AuditEvent`]s. Implemented by e11 (backed by `mw-admin`'s audit
/// log + the `mw-server` webhook feed). Emission is fire-and-forget from the
/// engine's perspective; a no-op when unattached.
pub trait AuditFeed: Send + Sync {
    /// Record one audit / webhook-feed event.
    fn emit(&self, event: AuditEvent);
}

/// The additive V6 hook bundle attached by e11 (MOUNT). Cheaply cloneable — the
/// [`Cache`] is `Arc`-backed and the posture source + feed are trait objects.
#[derive(Clone)]
pub struct V6Hooks {
    cache: Option<Cache>,
    posture: Arc<dyn AccountPostureSource>,
    feed: Option<Arc<dyn AuditFeed>>,
}

impl Default for V6Hooks {
    fn default() -> Self {
        Self {
            cache: None,
            posture: Arc::new(StandardPosture),
            feed: None,
        }
    }
}

impl V6Hooks {
    /// The inert default: no cache, standard posture for every account, no feed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach the layered cache used for cache-aside on the read paths.
    #[must_use]
    pub fn with_cache(mut self, cache: Cache) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Attach the zero-access-aware posture source (e11 → `zeroaccess_accounts`).
    #[must_use]
    pub fn with_posture_source(mut self, src: Arc<dyn AccountPostureSource>) -> Self {
        self.posture = src;
        self
    }

    /// Attach the audit / webhook feed sink.
    #[must_use]
    pub fn with_feed(mut self, feed: Arc<dyn AuditFeed>) -> Self {
        self.feed = Some(feed);
        self
    }
}

impl Engine {
    /// Attach the V6 hooks (cache / posture source / audit feed). Called once by
    /// e11 (MOUNT) after the engine is built and the store backend is selected.
    /// Idempotent-by-replacement — a later call swaps the whole bundle.
    pub fn attach_v6(&self, hooks: V6Hooks) {
        *self.v6.write().expect("v6 hooks lock") = hooks;
    }

    /// A cheap clone of the current hook bundle (cache handle + posture + feed).
    pub(crate) fn v6_hooks(&self) -> V6Hooks {
        self.v6.read().expect("v6 hooks lock").clone()
    }

    /// The cache posture for an account (standard unless e11's source says
    /// zero-access). Public so `mw-server`/tests can assert the boundary.
    pub fn account_posture(&self, account_id: &str) -> AccountPosture {
        self.v6
            .read()
            .expect("v6 hooks lock")
            .posture
            .posture(account_id)
    }

    /// Whether a cache is currently attached (drives the `mailwoman doctor`
    /// engine-side posture line + tests).
    pub fn cache_attached(&self) -> bool {
        self.v6.read().expect("v6 hooks lock").cache.is_some()
    }

    // ── Cache-aside read helpers (plan §3 e10) ───────────────────────────────

    /// Read a message's stored envelope (the "header window" content, SPEC
    /// §15.6 `HeaderWindows`) through the cache when one is attached, else
    /// straight from the store (identical to the V5 path).
    ///
    /// The envelope is plaintext-derived, so it is routed through
    /// [`Cache::get_derived`] with the account posture: a zero-access account's
    /// envelope is loaded per-request and never cached (the store already holds
    /// ciphertext the engine cannot read — this only guarantees no engine cache
    /// tier ever holds a plaintext-derived copy).
    pub(crate) async fn cached_envelope(
        &self,
        account_id: &str,
        stable_id: &str,
    ) -> Result<Option<Vec<u8>>> {
        let hooks = self.v6_hooks();
        let Some(cache) = hooks.cache else {
            return Ok(self.store().get_envelope(stable_id).await?);
        };
        let posture = hooks.posture.posture(account_id);
        let store = self.store().clone();
        let sid = stable_id.to_string();
        let derived = cache
            .get_derived::<Option<Vec<u8>>, _, _>(
                CacheClass::HeaderWindows,
                stable_id,
                posture,
                move || async move {
                    store
                        .get_envelope(&sid)
                        .await
                        .map_err(|e| CacheError::Store(e.to_string()))
                },
            )
            .await
            .map_err(cache_err)?;
        Ok(derived.into_inner())
    }

    /// Read a message's sealed body bytes (SPEC §15.6 `MessageBodies`) through
    /// the cache when one is attached, else straight from the store. The body is
    /// content-addressed by `blob_ref` and immutable, so a hit is always valid;
    /// zero-access bodies bypass every shared tier via [`Cache::get_derived`].
    pub(crate) async fn cached_body(
        &self,
        account_id: &str,
        stable_id: &str,
        blob_ref: &str,
    ) -> Result<Option<Vec<u8>>> {
        let hooks = self.v6_hooks();
        let Some(cache) = hooks.cache else {
            return Ok(self.store().get_body(blob_ref).await?);
        };
        let posture = hooks.posture.posture(account_id);
        let store = self.store().clone();
        let blob = blob_ref.to_string();
        let derived = cache
            .get_derived::<Option<Vec<u8>>, _, _>(
                CacheClass::MessageBodies,
                stable_id,
                posture,
                move || async move {
                    store
                        .get_body(&blob)
                        .await
                        .map_err(|e| CacheError::Store(e.to_string()))
                },
            )
            .await
            .map_err(cache_err)?;
        Ok(derived.into_inner())
    }

    /// Invalidate a message's cached envelope + body across every tier (called
    /// when a message is re-ingested/updated or expunged so the cache-aside
    /// helpers never serve a stale copy). A no-op when no cache is attached.
    pub(crate) async fn invalidate_message_cache(&self, stable_id: &str) {
        let hooks = self.v6_hooks();
        let Some(cache) = hooks.cache else {
            return;
        };
        let _ = cache.invalidate(CacheClass::HeaderWindows, stable_id).await;
        let _ = cache.invalidate(CacheClass::MessageBodies, stable_id).await;
    }

    // ── Audit / webhook feed (plan §3 e10) ───────────────────────────────────

    /// Emit one [`AuditEvent`] to the attached feed (audit log + webhooks). A
    /// no-op when no feed is attached, so the existing path emits nothing.
    pub(crate) fn emit_audit(&self, event: AuditEvent) {
        if let Some(feed) = &self.v6.read().expect("v6 hooks lock").feed {
            feed.emit(event);
        }
    }
}

/// Map a [`CacheError`] into the engine error type. A cache failure is never
/// authoritative (mw-cache degrades to the loader), so this only surfaces a
/// serialization/store fault the loader itself hit.
fn cache_err(e: CacheError) -> EngineError {
    EngineError::Protocol(format!("cache: {e}"))
}
