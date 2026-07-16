//! PIM / Outlook-parity adapters — the guest `calendar` / `tasks` / `bridge-parity`
//! exports (the t10 second-world `plugin-pim`, plan §4/§5) ↔ the frozen `mw-engine`
//! `Bridge*` trait objects. **SECURITY-SENSITIVE**: every call here runs THROUGH the
//! same wasmtime jail as `account-backend` — the same resource limits (the epoch
//! deadline and the `StoreLimits` memory ceiling), the same deny-by-default capability
//! gate, and the same trap→typed-error discipline (a guest trap is a [`PluginError`],
//! **never** a host panic; the host survives).
//!
//! ## Per-interface export probing (plan §5)
//! A component exporting only `world plugin`'s interfaces (LanguageTool / Nextcloud,
//! the shipped 0.1.0 plugins) exports NONE of `mailwoman:plugin/{calendar,tasks,
//! bridge-parity}` — [`probe_pim`] returns all-`false`, so every `as_bridge_*`
//! accessor is `None`, no PIM adapter is bound, `Engine::bridge_*` returns `None`, and
//! the engine takes its byte-unchanged standards fallback. Such a component loads
//! exactly as before (the account-backend path is untouched). A PIM-capable bridge
//! (built for `world plugin-pim`) exports all three, so the probe binds them.
//!
//! ## Two-level gate (plan §2, task #2)
//! An adapter is handed to the engine ([`PluginHandle::as_bridge_calendar`] & friends)
//! ONLY when BOTH the interface is present (probe) AND the manifest/admin grant
//! includes [`Capability::AccountBackend`] (the bridge role). Missing either ⇒ `None`
//! ⇒ standards fallback. The low-level host calls ([`PluginHandle::bridge_recall`] &
//! friends, mirroring [`PluginHandle::call_dlp_detect`]) additionally return a typed
//! [`PluginError::CapabilityDenied`] when the capability is absent, so a denied PIM
//! call is an honest error, never a panic.
//!
//! ## Honest `supports-*()` (plan §2.1, e0 contract)
//! Interface *presence* is coarse; a guest that exports `bridge-parity` but does not
//! implement recall answers `supports-recall() -> false`. e13 (MOUNT) calls
//! [`PluginHandle::bridge_parity_caps`] / [`PluginHandle::bridge_supports_calendar`] /
//! [`PluginHandle::bridge_supports_tasks`] to build the honest per-account
//! [`mw_engine::BridgeCaps`] and to decide, per capability, whether to wire the
//! matching `as_bridge_*` trait object into its [`mw_engine::BridgeCapabilitySource`]
//! (present + granted + `supports-* == true`), else leave it `None` for the fallback.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use wasmtime::Store;

use mw_engine::{
    BridgeCalendar, BridgeCalendarInfo, BridgeCaps, BridgeEventDelta, BridgeEventInfo,
    BridgeFocusedSync, BridgeReaction, BridgeReactions, BridgeRecall, BridgeRoomInfo,
    BridgeTaskDelta, BridgeTaskInfo, BridgeTasks, BridgeVoteTally, BridgeVoting, EngineError,
    FocusedState, MessageRef, RecallOutcome,
};

use crate::bindings::pim::PluginPim;
use crate::bindings::pim::exports::mailwoman::plugin::{
    bridge_parity as wparity, calendar as wcal, tasks as wtasks,
};
use crate::bindings::pim::mailwoman::plugin::types as wpim;
use crate::host_state::{HostState, map_call_err, new_store};
use crate::{Capability, PluginCtx, PluginError, PluginHandle, Result};

/// The three PIM interface ids (version-stripped) probed on the raw component.
const IFACE_CALENDAR: &str = "mailwoman:plugin/calendar";
const IFACE_TASKS: &str = "mailwoman:plugin/tasks";
const IFACE_PARITY: &str = "mailwoman:plugin/bridge-parity";

/// Which of the optional PIM/parity interfaces a component exports (plan §5).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PimProbe {
    pub(crate) calendar: bool,
    pub(crate) tasks: bool,
    pub(crate) parity: bool,
}

impl PimProbe {
    /// Whether the component exports ANY PIM interface (⇒ it is a `plugin-pim` guest).
    pub(crate) fn any(self) -> bool {
        self.calendar || self.tasks || self.parity
    }
}

/// Enumerate a component's exported interfaces and record which of the three optional
/// PIM interfaces are present (plan §5, task #1). This walks the component TYPE — it
/// neither instantiates nor runs any guest code, so a `world plugin`-only component is
/// classified as "no PIM" without ever entering the jail.
pub(crate) fn probe_pim(
    engine: &wasmtime::Engine,
    component: &wasmtime::component::Component,
) -> PimProbe {
    let mut probe = PimProbe::default();
    for (name, _item) in component.component_type().exports(engine) {
        // Export names are `mailwoman:plugin/<iface>@<version>`; match on the
        // version-stripped id so a future package rev keeps probing correctly.
        let id = name.split_once('@').map_or(name, |(id, _ver)| id);
        match id {
            IFACE_CALENDAR => probe.calendar = true,
            IFACE_TASKS => probe.tasks = true,
            IFACE_PARITY => probe.parity = true,
            _ => {}
        }
    }
    probe
}

/// A live wasmtime store + instantiated `plugin-pim` component for one PIM session.
struct PimSession {
    store: Store<HostState>,
    pim: PluginPim,
}

/// The honest per-account PIM support advertised by a guest's `supports-*()` funcs
/// (plan §2.1). Distinct from mere interface presence (the coarse probe). Internal —
/// exposed to e13 as `mw-engine`'s public [`BridgeCaps`] + calendar/tasks bools, so no
/// mw-plugin-private type crosses the public API.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PimSupport {
    pub(crate) calendar: bool,
    pub(crate) tasks: bool,
    pub(crate) reactions: bool,
    pub(crate) voting: bool,
    pub(crate) recall: bool,
    pub(crate) focused: bool,
}

/// Backs the `mw-engine` `Bridge{Calendar,Tasks,Reactions,Voting,Recall,FocusedSync}`
/// trait objects with through-the-jail WIT calls against one plugin's `plugin-pim`
/// exports. Holds one lazily-created, resource-limited session (instance-per-account),
/// exactly like [`crate::adapter::AccountBackendAdapter`].
pub(crate) struct PluginPimBackend {
    ctx: Arc<PluginCtx>,
    session: Mutex<Option<Arc<Mutex<PimSession>>>>,
}

impl PluginPimBackend {
    pub(crate) fn new(ctx: Arc<PluginCtx>) -> Self {
        Self {
            ctx,
            session: Mutex::new(None),
        }
    }

    fn probe(&self) -> PimProbe {
        probe_pim(&self.ctx.engine, &self.ctx.component)
    }

    /// Deny-by-default gate for a PIM call: the `account-backend` capability (the
    /// bridge role) must be granted, and the required interface must be exported.
    fn ensure(&self, present: bool) -> Result<()> {
        if !self.ctx.granted.contains(&Capability::AccountBackend) {
            return Err(PluginError::CapabilityDenied(
                "PIM/parity requires the account-backend capability".into(),
            ));
        }
        if !present {
            return Err(PluginError::Runtime(
                "plugin does not export this PIM interface".into(),
            ));
        }
        Ok(())
    }

    /// Get (or lazily instantiate) the persistent `plugin-pim` session, under the
    /// SAME resource limits (memory ceiling + epoch deadline + fuel) as the
    /// account-backend path.
    async fn session(&self) -> Result<Arc<Mutex<PimSession>>> {
        let mut slot = self.session.lock().await;
        if let Some(s) = slot.as_ref() {
            return Ok(s.clone());
        }
        let mut store = new_store(
            &self.ctx.engine,
            self.ctx.gate.clone(),
            self.ctx.services.clone(),
            &self.ctx.limits,
            self.ctx.plugin_id.clone(),
            self.ctx.bound_account.clone(),
        )?;
        let pim = PluginPim::instantiate_async(&mut store, &self.ctx.component, &self.ctx.linker)
            .await
            .map_err(|e| map_call_err(&store, e))?;
        let arc = Arc::new(Mutex::new(PimSession { store, pim }));
        *slot = Some(arc.clone());
        Ok(arc)
    }

    // ── supports-*() honesty probe (calls into the guest) ─────────────────────────

    pub(crate) async fn supports(&self) -> Result<PimSupport> {
        let probe = self.probe();
        self.ensure(probe.any())?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let mut out = PimSupport::default();
        if probe.calendar {
            let cal = pim.mailwoman_plugin_calendar();
            out.calendar = cal
                .call_supports_calendar(&mut *store)
                .await
                .map_err(|e| map_call_err(store, e))?;
        }
        if probe.tasks {
            let t = pim.mailwoman_plugin_tasks();
            out.tasks = t
                .call_supports_tasks(&mut *store)
                .await
                .map_err(|e| map_call_err(store, e))?;
        }
        if probe.parity {
            let p = pim.mailwoman_plugin_bridge_parity();
            out.reactions = p
                .call_supports_reactions(&mut *store)
                .await
                .map_err(|e| map_call_err(store, e))?;
            out.voting = p
                .call_supports_voting(&mut *store)
                .await
                .map_err(|e| map_call_err(store, e))?;
            out.recall = p
                .call_supports_recall(&mut *store)
                .await
                .map_err(|e| map_call_err(store, e))?;
            out.focused = p
                .call_supports_focused(&mut *store)
                .await
                .map_err(|e| map_call_err(store, e))?;
        }
        Ok(out)
    }

    // ── calendar core calls (PluginError-typed) ───────────────────────────────────

    pub(crate) async fn calendars_list(&self) -> Result<Vec<BridgeCalendarInfo>> {
        self.ensure(self.probe().calendar)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let cal = pim.mailwoman_plugin_calendar();
        let list = cal
            .call_list_calendars(&mut *store)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)?;
        Ok(list.into_iter().map(cal_info_to_engine).collect())
    }

    pub(crate) async fn events_sync(
        &self,
        calendar_id: &str,
        cursor: &[u8],
    ) -> Result<BridgeEventDelta> {
        self.ensure(self.probe().calendar)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let cal = pim.mailwoman_plugin_calendar();
        let wcur = wpim::SyncCursor {
            opaque: cursor.to_vec(),
        };
        let delta = cal
            .call_sync_events(&mut *store, calendar_id, &wcur)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)?;
        Ok(event_delta_to_engine(delta))
    }

    pub(crate) async fn rooms_find(&self) -> Result<Vec<BridgeRoomInfo>> {
        self.ensure(self.probe().calendar)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let cal = pim.mailwoman_plugin_calendar();
        let rooms = cal
            .call_find_rooms(&mut *store)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)?;
        Ok(rooms.into_iter().map(room_info_to_engine).collect())
    }

    pub(crate) async fn schedule_get(&self, who: &str, start: &str, end: &str) -> Result<String> {
        self.ensure(self.probe().calendar)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let cal = pim.mailwoman_plugin_calendar();
        cal.call_get_schedule(&mut *store, who, start, end)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)
    }

    // ── tasks core calls ──────────────────────────────────────────────────────────

    pub(crate) async fn tasks_list(&self) -> Result<Vec<BridgeTaskInfo>> {
        self.ensure(self.probe().tasks)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let t = pim.mailwoman_plugin_tasks();
        let list = t
            .call_list_tasks(&mut *store)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)?;
        Ok(list.into_iter().map(task_info_to_engine).collect())
    }

    pub(crate) async fn tasks_sync(&self, list_id: &str, cursor: &[u8]) -> Result<BridgeTaskDelta> {
        self.ensure(self.probe().tasks)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let t = pim.mailwoman_plugin_tasks();
        let wcur = wpim::SyncCursor {
            opaque: cursor.to_vec(),
        };
        let delta = t
            .call_sync_tasks(&mut *store, list_id, &wcur)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)?;
        Ok(task_delta_to_engine(delta))
    }

    pub(crate) async fn task_complete(&self, id: &str) -> Result<()> {
        self.ensure(self.probe().tasks)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let t = pim.mailwoman_plugin_tasks();
        t.call_complete(&mut *store, id)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)
    }

    // ── bridge-parity core calls ──────────────────────────────────────────────────

    pub(crate) async fn reaction_set(
        &self,
        msg: &MessageRef,
        emoji: &str,
        add: bool,
    ) -> Result<()> {
        self.ensure(self.probe().parity)?;
        let wmsg = msgref_to_pim(msg)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let p = pim.mailwoman_plugin_bridge_parity();
        p.call_set_reaction(&mut *store, &wmsg, emoji, add)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)
    }

    pub(crate) async fn reactions_get(&self, msg: &MessageRef) -> Result<Vec<BridgeReaction>> {
        self.ensure(self.probe().parity)?;
        let wmsg = msgref_to_pim(msg)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let p = pim.mailwoman_plugin_bridge_parity();
        let rs = p
            .call_get_reactions(&mut *store, &wmsg)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)?;
        Ok(rs.into_iter().map(reaction_to_engine).collect())
    }

    pub(crate) async fn vote_cast(&self, msg: &MessageRef, choice: &str) -> Result<()> {
        self.ensure(self.probe().parity)?;
        let wmsg = msgref_to_pim(msg)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let p = pim.mailwoman_plugin_bridge_parity();
        p.call_cast_vote(&mut *store, &wmsg, choice)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)
    }

    pub(crate) async fn vote_tally(&self, msg: &MessageRef) -> Result<Vec<BridgeVoteTally>> {
        self.ensure(self.probe().parity)?;
        let wmsg = msgref_to_pim(msg)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let p = pim.mailwoman_plugin_bridge_parity();
        let t = p
            .call_tally(&mut *store, &wmsg)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)?;
        Ok(t.into_iter().map(vote_tally_to_engine).collect())
    }

    pub(crate) async fn msg_recall(&self, msg: &MessageRef) -> Result<RecallOutcome> {
        self.ensure(self.probe().parity)?;
        let wmsg = msgref_to_pim(msg)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let p = pim.mailwoman_plugin_bridge_parity();
        let out = p
            .call_recall(&mut *store, &wmsg)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)?;
        Ok(recall_outcome_to_engine(out))
    }

    pub(crate) async fn focused_get(&self, msg: &MessageRef) -> Result<FocusedState> {
        self.ensure(self.probe().parity)?;
        let wmsg = msgref_to_pim(msg)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let p = pim.mailwoman_plugin_bridge_parity();
        let f = p
            .call_get_focused(&mut *store, &wmsg)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)?;
        Ok(focused_state_to_engine(f))
    }

    pub(crate) async fn focused_set(&self, msg: &MessageRef, focused: bool) -> Result<()> {
        self.ensure(self.probe().parity)?;
        let wmsg = msgref_to_pim(msg)?;
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let PimSession { store, pim } = &mut *g;
        let p = pim.mailwoman_plugin_bridge_parity();
        p.call_set_focused(&mut *store, &wmsg, focused)
            .await
            .map_err(|e| map_call_err(store, e))?
            .map_err(wit_pim_err)
    }
}

// ── engine trait impls (through-jail; PluginError → EngineError) ──────────────────

#[async_trait]
impl BridgeCalendar for PluginPimBackend {
    async fn list_calendars(&self) -> mw_engine::Result<Vec<BridgeCalendarInfo>> {
        self.calendars_list().await.map_err(plugin_to_engine)
    }
    async fn sync_events(
        &self,
        calendar_id: &str,
        cursor: &[u8],
    ) -> mw_engine::Result<BridgeEventDelta> {
        self.events_sync(calendar_id, cursor)
            .await
            .map_err(plugin_to_engine)
    }
    async fn find_rooms(&self) -> mw_engine::Result<Vec<BridgeRoomInfo>> {
        self.rooms_find().await.map_err(plugin_to_engine)
    }
    async fn get_schedule(&self, who: &str, start: &str, end: &str) -> mw_engine::Result<String> {
        self.schedule_get(who, start, end)
            .await
            .map_err(plugin_to_engine)
    }
}

#[async_trait]
impl BridgeTasks for PluginPimBackend {
    async fn list_tasks(&self) -> mw_engine::Result<Vec<BridgeTaskInfo>> {
        self.tasks_list().await.map_err(plugin_to_engine)
    }
    async fn sync_tasks(&self, list_id: &str, cursor: &[u8]) -> mw_engine::Result<BridgeTaskDelta> {
        self.tasks_sync(list_id, cursor)
            .await
            .map_err(plugin_to_engine)
    }
    async fn complete(&self, id: &str) -> mw_engine::Result<()> {
        self.task_complete(id).await.map_err(plugin_to_engine)
    }
}

#[async_trait]
impl BridgeReactions for PluginPimBackend {
    async fn set_reaction(
        &self,
        msg: &MessageRef,
        emoji: &str,
        add: bool,
    ) -> mw_engine::Result<()> {
        self.reaction_set(msg, emoji, add)
            .await
            .map_err(plugin_to_engine)
    }
    async fn get_reactions(&self, msg: &MessageRef) -> mw_engine::Result<Vec<BridgeReaction>> {
        self.reactions_get(msg).await.map_err(plugin_to_engine)
    }
}

#[async_trait]
impl BridgeVoting for PluginPimBackend {
    async fn cast_vote(&self, msg: &MessageRef, option: &str) -> mw_engine::Result<()> {
        self.vote_cast(msg, option).await.map_err(plugin_to_engine)
    }
    async fn tally(&self, msg: &MessageRef) -> mw_engine::Result<Vec<BridgeVoteTally>> {
        self.vote_tally(msg).await.map_err(plugin_to_engine)
    }
}

#[async_trait]
impl BridgeRecall for PluginPimBackend {
    async fn recall(&self, msg: &MessageRef) -> mw_engine::Result<RecallOutcome> {
        self.msg_recall(msg).await.map_err(plugin_to_engine)
    }
}

#[async_trait]
impl BridgeFocusedSync for PluginPimBackend {
    async fn focused_state(&self, msg: &MessageRef) -> mw_engine::Result<FocusedState> {
        self.focused_get(msg).await.map_err(plugin_to_engine)
    }
    async fn set_focused(&self, msg: &MessageRef, focused: bool) -> mw_engine::Result<()> {
        self.focused_set(msg, focused)
            .await
            .map_err(plugin_to_engine)
    }
}

// ── PluginHandle PIM surface (public; consumed by e13 MOUNT + the tests) ──────────

impl PluginHandle {
    /// The PIM/parity export probe for this component (plan §5). All-`false` for a
    /// shipped `world plugin`-only plugin (account-backend/LanguageTool/Nextcloud).
    fn pim_probe(&self) -> PimProbe {
        probe_pim(&self.ctx.engine, &self.ctx.component)
    }

    /// Whether this component exports the PIM/parity interfaces AND the bridge
    /// (`account-backend`) capability is granted — i.e. whether ANY PIM adapter can be
    /// bound. `false` ⇒ the engine keeps its byte-unchanged standards fallback.
    #[must_use]
    pub fn advertises_pim(&self) -> bool {
        self.pim_probe().any() && self.ctx.granted.contains(&Capability::AccountBackend)
    }

    fn pim_backend(&self) -> Arc<PluginPimBackend> {
        Arc::new(PluginPimBackend::new(self.ctx.clone()))
    }

    /// Whether a PIM adapter may be bound for `present` (interface probed) AND the
    /// `account-backend` capability is granted (deny-by-default, task #2).
    fn pim_bindable(&self, present: bool) -> bool {
        present && self.ctx.granted.contains(&Capability::AccountBackend)
    }

    /// The bridge-native calendar adapter for the engine, or `None` (⇒ CalDAV/standards
    /// fallback). `Some` iff the `calendar` interface is exported AND `account-backend`
    /// is granted. e13 should additionally check [`PluginHandle::bridge_supports_calendar`]
    /// (the honest `supports-calendar()`), wiring this only when it is `true`.
    #[must_use]
    pub fn as_bridge_calendar(&self) -> Option<Arc<dyn BridgeCalendar>> {
        self.pim_bindable(self.pim_probe().calendar)
            .then(|| self.pim_backend() as Arc<dyn BridgeCalendar>)
    }

    /// The bridge-native tasks adapter, or `None` (⇒ standards fallback). See
    /// [`PluginHandle::as_bridge_calendar`]; honest gate is [`PluginHandle::bridge_supports_tasks`].
    #[must_use]
    pub fn as_bridge_tasks(&self) -> Option<Arc<dyn BridgeTasks>> {
        self.pim_bindable(self.pim_probe().tasks)
            .then(|| self.pim_backend() as Arc<dyn BridgeTasks>)
    }

    /// The bridge-native reactions adapter, or `None` (⇒ header-convention fallback).
    /// Gated on the `bridge-parity` interface + `account-backend`; e13 wires it only
    /// when `bridge_parity_caps().reactions` is `true`.
    #[must_use]
    pub fn as_bridge_reactions(&self) -> Option<Arc<dyn BridgeReactions>> {
        self.pim_bindable(self.pim_probe().parity)
            .then(|| self.pim_backend() as Arc<dyn BridgeReactions>)
    }

    /// The bridge-native voting adapter, or `None`. See [`PluginHandle::as_bridge_reactions`].
    #[must_use]
    pub fn as_bridge_voting(&self) -> Option<Arc<dyn BridgeVoting>> {
        self.pim_bindable(self.pim_probe().parity)
            .then(|| self.pim_backend() as Arc<dyn BridgeVoting>)
    }

    /// The bridge-native recall adapter, or `None`. See [`PluginHandle::as_bridge_reactions`].
    #[must_use]
    pub fn as_bridge_recall(&self) -> Option<Arc<dyn BridgeRecall>> {
        self.pim_bindable(self.pim_probe().parity)
            .then(|| self.pim_backend() as Arc<dyn BridgeRecall>)
    }

    /// The bridge-native Focused-Inbox-sync adapter, or `None`. See
    /// [`PluginHandle::as_bridge_reactions`].
    #[must_use]
    pub fn as_bridge_focused_sync(&self) -> Option<Arc<dyn BridgeFocusedSync>> {
        self.pim_bindable(self.pim_probe().parity)
            .then(|| self.pim_backend() as Arc<dyn BridgeFocusedSync>)
    }

    // ── honest `supports-*()` probes (e13 builds BridgeCaps + routing from these) ──

    /// The full per-account PIM support the guest advertises via its `supports-*()`
    /// funcs (through the jail). Requires `account-backend`; deny ⇒ `CapabilityDenied`.
    /// Internal; surfaced publicly as [`PluginHandle::bridge_parity_caps`] +
    /// [`PluginHandle::bridge_supports_calendar`] / [`PluginHandle::bridge_supports_tasks`].
    pub(crate) async fn bridge_pim_support(&self) -> Result<PimSupport> {
        self.pim_backend().supports().await
    }

    /// The Outlook-parity subset of the support probe as an `mw-engine` [`BridgeCaps`]
    /// (reactions / voting / recall / focused-sync), for e13's `BridgeCapabilitySource::caps`.
    pub async fn bridge_parity_caps(&self) -> Result<BridgeCaps> {
        let s = self.bridge_pim_support().await?;
        Ok(BridgeCaps {
            reactions: s.reactions,
            voting: s.voting,
            recall: s.recall,
            focused_sync: s.focused,
        })
    }

    /// Whether the guest honestly implements calendar for the bound account.
    pub async fn bridge_supports_calendar(&self) -> Result<bool> {
        Ok(self.bridge_pim_support().await?.calendar)
    }

    /// Whether the guest honestly implements tasks for the bound account.
    pub async fn bridge_supports_tasks(&self) -> Result<bool> {
        Ok(self.bridge_pim_support().await?.tasks)
    }

    // ── low-level host PIM calls (PluginError-typed; mirror `call_dlp_detect`) ─────
    // These are the primitives the trait adapters wrap; they return the typed
    // `PluginError` directly (a denied call ⇒ `CapabilityDenied`, an out-of-deadline
    // call ⇒ `LimitExceeded`) and are the host-level entry points the jail tests drive.

    /// Add/remove a reaction on a message (parity interface + `account-backend`).
    pub async fn bridge_set_reaction(
        &self,
        msg: &MessageRef,
        emoji: &str,
        add: bool,
    ) -> Result<()> {
        self.pim_backend().reaction_set(msg, emoji, add).await
    }

    /// List the reactions on a message.
    pub async fn bridge_get_reactions(&self, msg: &MessageRef) -> Result<Vec<BridgeReaction>> {
        self.pim_backend().reactions_get(msg).await
    }

    /// Attempt to recall a sent message; returns the honest [`RecallOutcome`].
    pub async fn bridge_recall(&self, msg: &MessageRef) -> Result<RecallOutcome> {
        self.pim_backend().msg_recall(msg).await
    }

    /// Sync calendar events since an opaque cursor (calendar interface + `account-backend`).
    pub async fn bridge_sync_events(
        &self,
        calendar_id: &str,
        cursor: &[u8],
    ) -> Result<BridgeEventDelta> {
        self.pim_backend().events_sync(calendar_id, cursor).await
    }

    /// The calendars the bound account exposes.
    pub async fn bridge_list_calendars(&self) -> Result<Vec<BridgeCalendarInfo>> {
        self.pim_backend().calendars_list().await
    }
}

// ── error mapping ─────────────────────────────────────────────────────────────────

/// Map a `plugin-pim` WIT `plugin-error` (returned in-band by the guest) → the host's
/// typed [`PluginError`], preserving the `LimitExceeded`/`CapabilityDenied` attribution.
fn wit_pim_err(e: wpim::PluginError) -> PluginError {
    match e {
        wpim::PluginError::LimitExceeded(m) => PluginError::LimitExceeded(m),
        wpim::PluginError::CapabilityDenied(m) => PluginError::CapabilityDenied(m),
        wpim::PluginError::Protocol(m)
        | wpim::PluginError::Auth(m)
        | wpim::PluginError::Transport(m)
        | wpim::PluginError::Unsupported(m)
        | wpim::PluginError::MailboxNotFound(m)
        | wpim::PluginError::Other(m) => PluginError::Runtime(m),
    }
}

/// Map a host-boundary [`PluginError`] onto the engine's error type (so a PIM call
/// degrades under the engine's uniform retry/error policy, exactly like the
/// account-backend adapter's `plugin_to_engine`).
fn plugin_to_engine(e: PluginError) -> EngineError {
    match e {
        PluginError::LimitExceeded(m) => {
            EngineError::Transport(format!("plugin limit exceeded: {m}"))
        }
        PluginError::CapabilityDenied(m) => {
            EngineError::Unsupported(format!("plugin capability denied: {m}"))
        }
        other => EngineError::Protocol(other.to_string()),
    }
}

// ── WIT (`plugin-pim`) → mw-engine record conversions ─────────────────────────────

fn cal_info_to_engine(c: wcal::CalInfo) -> BridgeCalendarInfo {
    BridgeCalendarInfo {
        id: c.id,
        name: c.name,
        role: c.role,
        read_only: c.read_only,
    }
}

fn room_info_to_engine(r: wcal::RoomInfo) -> BridgeRoomInfo {
    BridgeRoomInfo {
        address: r.address,
        name: r.name,
        capacity: r.capacity,
    }
}

fn event_info_to_engine(e: wcal::EventInfo) -> BridgeEventInfo {
    BridgeEventInfo {
        id: e.id,
        calendar_id: e.calendar_id,
        ical: e.ical,
        start: e.start,
        end: e.end,
    }
}

fn event_delta_to_engine(d: wcal::EventDelta) -> BridgeEventDelta {
    BridgeEventDelta {
        changed: d.changed.into_iter().map(event_info_to_engine).collect(),
        removed: d.removed,
        next_cursor: d.next_cursor.opaque,
    }
}

fn task_info_to_engine(t: wtasks::TaskInfo) -> BridgeTaskInfo {
    BridgeTaskInfo {
        id: t.id,
        list_id: t.list_id,
        ical: t.ical,
        completed: t.completed,
    }
}

fn task_delta_to_engine(d: wtasks::TaskDelta) -> BridgeTaskDelta {
    BridgeTaskDelta {
        changed: d.changed.into_iter().map(task_info_to_engine).collect(),
        removed: d.removed,
        next_cursor: d.next_cursor.opaque,
    }
}

fn reaction_to_engine(r: wparity::Reaction) -> BridgeReaction {
    BridgeReaction {
        actor: r.actor,
        emoji: r.emoji,
    }
}

fn vote_tally_to_engine(v: wparity::VoteTally) -> BridgeVoteTally {
    // WIT names the field `choice` (`option` is a reserved WIT keyword); the engine
    // record names it `option`.
    BridgeVoteTally {
        option: v.choice,
        count: v.count,
    }
}

fn recall_outcome_to_engine(o: wparity::RecallOutcome) -> RecallOutcome {
    match o {
        wparity::RecallOutcome::Requested => RecallOutcome::Requested,
        wparity::RecallOutcome::Unsupported => RecallOutcome::Unsupported,
        wparity::RecallOutcome::Failed(reason) => RecallOutcome::Failed { reason },
    }
}

fn focused_state_to_engine(f: wparity::FocusedState) -> FocusedState {
    match f {
        wparity::FocusedState::Focused => FocusedState::Focused,
        wparity::FocusedState::Other => FocusedState::Other,
    }
}

/// Encode an engine [`MessageRef`] into the `plugin-pim` WIT `message-ref` — the SAME
/// opaque-`raw` encoding the account-backend adapter uses (a bridge's provider-native
/// id rides `raw` verbatim; an IMAP/POP3 ref is JSON-in-`raw`), so a `message-ref`
/// round-trips identically across both seams.
fn msgref_to_pim(r: &MessageRef) -> Result<wpim::MessageRef> {
    match r {
        MessageRef::Plugin { raw } => Ok(wpim::MessageRef {
            raw: raw.clone(),
            mailbox: wpim::MailboxRef {
                name: String::new(),
                uidvalidity: 0,
            },
        }),
        MessageRef::Imap { mailbox, .. } => Ok(wpim::MessageRef {
            raw: serde_json::to_string(r)
                .map_err(|e| PluginError::Runtime(format!("encode message-ref: {e}")))?,
            mailbox: wpim::MailboxRef {
                name: mailbox.name.clone(),
                uidvalidity: mailbox.uidvalidity,
            },
        }),
        MessageRef::Pop3 { .. } => Ok(wpim::MessageRef {
            raw: serde_json::to_string(r)
                .map_err(|e| PluginError::Runtime(format!("encode message-ref: {e}")))?,
            mailbox: wpim::MailboxRef {
                name: "INBOX".into(),
                uidvalidity: 0,
            },
        }),
    }
}
