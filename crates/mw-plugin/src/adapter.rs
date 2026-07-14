//! The guest `account-backend` export ↔ frozen [`mw_engine::AccountBackend`]
//! adapter (plan §2.1 / §6.5, R1). The engine drives the async loop host-side; the
//! guest is a side-effect-free transform reachable only through the gated host
//! imports. Once adapted, a plugin backend is indistinguishable from `mw-imap`.
//!
//! **Impedance notes (R1, resolved):**
//! * `mw_engine::MessageRef` is a structured enum; the WIT `message-ref.raw` is an
//!   opaque per-backend string. IMAP/POP3 refs are JSON-encoded into `raw` and decoded
//!   back — lossless. A `MessageRef::Plugin { raw }` (a bridge's provider-native id)
//!   rides `raw` VERBATIM (no JSON), so Graph/Gmail ids and EWS ItemIds round-trip
//!   without the earlier `Pop3 { uidl }`/synthetic-uid smuggles (t7-fix-msgref).
//! * `mw_engine::SyncCursor` maps the same way: standards cursors are JSON-encoded into
//!   the WIT `sync-cursor.opaque`, while `SyncCursor::Plugin { opaque }` (e8) passes its
//!   native token bytes through verbatim (Graph deltaLink / Gmail historyId / EWS
//!   SyncState). Decode tries the engine-JSON form first and falls back to `Plugin`.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

// The t10 PIM/parity adapters (plan §4/§5). Declared here — a child of the crate
// root — rather than in `lib.rs`, so this executor's owned file set stays exactly
// `{engine,bindings,adapter,adapter_pim}.rs` (§ commit protocol). Being a descendant
// of the crate root, `adapter_pim` can reach the private `PluginCtx`/`PluginHandle`
// internals it needs; its `pub` items surface on the already-public `PluginHandle`.
#[path = "adapter_pim.rs"]
pub(crate) mod adapter_pim;

use mw_engine::{
    AccountBackend, BackendCaps, ChangeEvent, ChangeSink, EngineError, Flag, MailboxDelta,
    MailboxRole, MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, SyncCursor,
    WatchHandle,
};

use crate::bindings::Plugin;
use crate::bindings::mailwoman::plugin::types as wit;
use crate::host_state::HostState;
use crate::{PluginCtx, PluginError};

/// A live wasmtime store + instantiated component for one plugin session.
pub(crate) struct GuestSession {
    pub(crate) store: wasmtime::Store<HostState>,
    pub(crate) plugin: Plugin,
}

/// Adapts a plugin's `account-backend` export onto the engine's frozen trait.
/// Holds one lazily-created, resource-limited session (instance-per-account).
pub(crate) struct AccountBackendAdapter {
    ctx: Arc<PluginCtx>,
    session: Mutex<Option<Arc<Mutex<GuestSession>>>>,
}

impl AccountBackendAdapter {
    pub(crate) fn new(ctx: Arc<PluginCtx>) -> Self {
        Self {
            ctx,
            session: Mutex::new(None),
        }
    }

    /// Get (or lazily instantiate) the persistent session for this account.
    async fn session(&self) -> Result<Arc<Mutex<GuestSession>>, EngineError> {
        let mut slot = self.session.lock().await;
        if let Some(s) = slot.as_ref() {
            return Ok(s.clone());
        }
        let s = self.ctx.instantiate().await.map_err(plugin_to_engine)?;
        let arc = Arc::new(Mutex::new(s));
        *slot = Some(arc.clone());
        Ok(arc)
    }
}

#[async_trait]
impl AccountBackend for AccountBackendAdapter {
    async fn capabilities(&self) -> mw_engine::Result<BackendCaps> {
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let GuestSession { store, plugin } = &mut *g;
        let ab = plugin.mailwoman_plugin_account_backend();
        let caps = ab
            .call_capabilities(&mut *store)
            .await
            .map_err(|e| trap_to_engine(store, e))?
            .map_err(wit_to_engine)?;
        Ok(BackendCaps {
            idle: caps.idle,
            r#move: caps.move_cap,
            ..Default::default()
        })
    }

    async fn list_mailboxes(&self) -> mw_engine::Result<Vec<RawMailbox>> {
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let GuestSession { store, plugin } = &mut *g;
        let ab = plugin.mailwoman_plugin_account_backend();
        let list = ab
            .call_list_mailboxes(&mut *store)
            .await
            .map_err(|e| trap_to_engine(store, e))?
            .map_err(wit_to_engine)?;
        Ok(list.into_iter().map(mailbox_to_engine).collect())
    }

    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> mw_engine::Result<MailboxDelta> {
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let GuestSession { store, plugin } = &mut *g;
        let ab = plugin.mailwoman_plugin_account_backend();
        let wmbox = mailbox_ref_to_wit(mbox);
        let wcur = cursor_to_wit(cursor)?;
        let delta = ab
            .call_sync_mailbox(&mut *store, &wmbox, &wcur)
            .await
            .map_err(|e| trap_to_engine(store, e))?
            .map_err(wit_to_engine)?;
        delta_to_engine(delta)
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> mw_engine::Result<Vec<RawMessage>> {
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let GuestSession { store, plugin } = &mut *g;
        let ab = plugin.mailwoman_plugin_account_backend();
        let wrefs: Vec<wit::MessageRef> =
            refs.iter().map(msgref_to_wit).collect::<Result<_, _>>()?;
        let out = ab
            .call_fetch_raw(&mut *store, &wrefs)
            .await
            .map_err(|e| trap_to_engine(store, e))?
            .map_err(wit_to_engine)?;
        out.into_iter().map(rawmsg_to_engine).collect()
    }

    async fn store_flags(
        &self,
        refs: &[MessageRef],
        add: &[Flag],
        remove: &[Flag],
    ) -> mw_engine::Result<()> {
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let GuestSession { store, plugin } = &mut *g;
        let ab = plugin.mailwoman_plugin_account_backend();
        let wrefs: Vec<wit::MessageRef> =
            refs.iter().map(msgref_to_wit).collect::<Result<_, _>>()?;
        let wadd: Vec<wit::Flag> = add.iter().map(flag_to_wit).collect();
        let wrem: Vec<wit::Flag> = remove.iter().map(flag_to_wit).collect();
        ab.call_store_flags(&mut *store, &wrefs, &wadd, &wrem)
            .await
            .map_err(|e| trap_to_engine(store, e))?
            .map_err(wit_to_engine)
    }

    async fn move_messages(
        &self,
        refs: &[MessageRef],
        to: &RawMailboxRef,
    ) -> mw_engine::Result<MoveOutcome> {
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let GuestSession { store, plugin } = &mut *g;
        let ab = plugin.mailwoman_plugin_account_backend();
        let wrefs: Vec<wit::MessageRef> =
            refs.iter().map(msgref_to_wit).collect::<Result<_, _>>()?;
        let wto = mailbox_ref_to_wit(to);
        ab.call_move_messages(&mut *store, &wrefs, &wto)
            .await
            .map_err(|e| trap_to_engine(store, e))?
            .map_err(wit_to_engine)?;
        // The WIT move returns unit; the engine re-derives destination refs by
        // Message-ID (bridges rarely expose UIDPLUS-style COPYUID).
        Ok(MoveOutcome::RederiveByMessageId)
    }

    async fn append(
        &self,
        mbox: &RawMailboxRef,
        raw: &[u8],
        flags: &[Flag],
    ) -> mw_engine::Result<MessageRef> {
        let sess = self.session().await?;
        let mut g = sess.lock().await;
        let GuestSession { store, plugin } = &mut *g;
        let ab = plugin.mailwoman_plugin_account_backend();
        let wmbox = mailbox_ref_to_wit(mbox);
        let wflags: Vec<wit::Flag> = flags.iter().map(flag_to_wit).collect();
        let r = ab
            .call_submit(&mut *store, &wmbox, raw, &wflags)
            .await
            .map_err(|e| trap_to_engine(store, e))?
            .map_err(wit_to_engine)?;
        msgref_to_engine(&r)
    }

    async fn watch(&self, sink: ChangeSink) -> mw_engine::Result<WatchHandle> {
        // Host-driven polling loop (the WIT `poll-changes` replaces the WASI async
        // stream, R1). A fresh, resource-limited session is used so the poll loop
        // never contends with foreground syncs; the WatchHandle stops it.
        let (stop_tx, mut stop_rx) = tokio::sync::watch::channel(false);
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let mut sess = match ctx.instantiate().await {
                Ok(s) => s,
                Err(_) => return,
            };
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                tokio::select! {
                    _ = stop_rx.changed() => {
                        if *stop_rx.borrow() { break; }
                    }
                    _ = ticker.tick() => {
                        let GuestSession { store, plugin } = &mut sess;
                        let ab = plugin.mailwoman_plugin_account_backend();
                        if let Ok(Ok(events)) = ab.call_poll_changes(&mut *store).await {
                            for ev in events {
                                let _ = sink.emit(change_to_engine(ev));
                            }
                        }
                    }
                }
            }
        });
        Ok(WatchHandle::new(stop_tx))
    }
}

// ── conversions ───────────────────────────────────────────────────────────────

fn mailbox_ref_to_wit(m: &RawMailboxRef) -> wit::MailboxRef {
    wit::MailboxRef {
        name: m.name.clone(),
        uidvalidity: m.uidvalidity,
    }
}

fn mailbox_ref_to_engine(m: wit::MailboxRef) -> RawMailboxRef {
    RawMailboxRef {
        name: m.name,
        uidvalidity: m.uidvalidity,
    }
}

fn role_from_str(s: &str) -> MailboxRole {
    match s {
        "inbox" => MailboxRole::Inbox,
        "archive" => MailboxRole::Archive,
        "drafts" => MailboxRole::Drafts,
        "sent" => MailboxRole::Sent,
        "trash" => MailboxRole::Trash,
        "junk" => MailboxRole::Junk,
        "flagged" => MailboxRole::Flagged,
        "all" => MailboxRole::All,
        _ => MailboxRole::None,
    }
}

fn mailbox_to_engine(m: wit::Mailbox) -> RawMailbox {
    RawMailbox {
        role: role_from_str(&m.role),
        parent: m.parent,
        // Bridges resync via the opaque cursor; UID coordinates are not meaningful.
        uidnext: 0,
        highestmodseq: 0,
        total: m.total,
        unread: m.unread,
        mailbox_ref: mailbox_ref_to_engine(m.mailbox_ref),
    }
}

fn flag_to_wit(f: &Flag) -> wit::Flag {
    match f {
        Flag::Seen => wit::Flag::Seen,
        Flag::Answered => wit::Flag::Answered,
        Flag::Flagged => wit::Flag::Flagged,
        Flag::Deleted => wit::Flag::Deleted,
        Flag::Draft => wit::Flag::Draft,
        Flag::Recent => wit::Flag::Recent,
        Flag::Keyword(k) => wit::Flag::Keyword(k.clone()),
    }
}

fn flag_to_engine(f: wit::Flag) -> Flag {
    match f {
        wit::Flag::Seen => Flag::Seen,
        wit::Flag::Answered => Flag::Answered,
        wit::Flag::Flagged => Flag::Flagged,
        wit::Flag::Deleted => Flag::Deleted,
        wit::Flag::Draft => Flag::Draft,
        wit::Flag::Recent => Flag::Recent,
        wit::Flag::Keyword(k) => Flag::Keyword(k),
    }
}

fn msgref_to_wit(r: &MessageRef) -> Result<wit::MessageRef, EngineError> {
    match r {
        // Plugin passthrough (R1-residual): a bridge's provider-native id rides the
        // WIT `message-ref.raw` VERBATIM — no JSON wrapping — so a Graph/Gmail id or
        // EWS ItemId round-trips losslessly. A `Plugin` ref carries no engine mailbox
        // coordinate (bridges pack any owning-folder context into `raw` themselves),
        // so the WIT `mailbox` is a neutral placeholder.
        MessageRef::Plugin { raw } => Ok(wit::MessageRef {
            raw: raw.clone(),
            mailbox: wit::MailboxRef {
                name: String::new(),
                uidvalidity: 0,
            },
        }),
        // IMAP/POP3 refs keep the JSON-in-`raw` encoding (byte-unchanged).
        MessageRef::Imap { mailbox, .. } => Ok(wit::MessageRef {
            raw: serde_json::to_string(r)
                .map_err(|e| EngineError::Protocol(format!("encode message-ref: {e}")))?,
            mailbox: mailbox_ref_to_wit(mailbox),
        }),
        MessageRef::Pop3 { .. } => Ok(wit::MessageRef {
            raw: serde_json::to_string(r)
                .map_err(|e| EngineError::Protocol(format!("encode message-ref: {e}")))?,
            mailbox: wit::MailboxRef {
                name: "INBOX".into(),
                uidvalidity: 0,
            },
        }),
    }
}

fn msgref_to_engine(r: &wit::MessageRef) -> Result<MessageRef, EngineError> {
    // A bridge emits a provider-native id that is NOT engine-`MessageRef` JSON — carry
    // it losslessly in `MessageRef::Plugin`. An engine-JSON ref (the IMAP/POP3-shaped
    // ref a non-bridge guest may echo back) still decodes to its exact variant, keeping
    // that path byte-unchanged.
    match serde_json::from_str::<MessageRef>(&r.raw) {
        Ok(m) => Ok(m),
        Err(_) => Ok(MessageRef::Plugin { raw: r.raw.clone() }),
    }
}

fn cursor_to_wit(c: &SyncCursor) -> Result<wit::SyncCursor, EngineError> {
    match c {
        // Plugin passthrough: a bridge's native token (Graph deltaLink / Gmail
        // historyId / EWS SyncState) rides `sync-cursor.opaque` VERBATIM.
        SyncCursor::Plugin { opaque } => Ok(wit::SyncCursor {
            opaque: opaque.clone(),
        }),
        // Standards cursors keep the JSON-in-`opaque` encoding (byte-unchanged).
        _ => {
            let opaque = serde_json::to_vec(c)
                .map_err(|e| EngineError::Protocol(format!("encode sync-cursor: {e}")))?;
            Ok(wit::SyncCursor { opaque })
        }
    }
}

fn cursor_to_engine(c: wit::SyncCursor) -> Result<SyncCursor, EngineError> {
    // A standards `SyncCursor` JSON (a non-bridge guest echoing one back) decodes to its
    // exact variant; a bridge's raw native token is not that JSON → `Plugin`.
    match serde_json::from_slice::<SyncCursor>(&c.opaque) {
        Ok(cur) => Ok(cur),
        Err(_) => Ok(SyncCursor::Plugin { opaque: c.opaque }),
    }
}

fn rawmsg_to_engine(m: wit::RawMessage) -> Result<RawMessage, EngineError> {
    Ok(RawMessage {
        message_ref: msgref_to_engine(&m.message_ref)?,
        raw: m.raw,
        flags: m.msg_flags.into_iter().map(flag_to_engine).collect(),
        internaldate: m.internaldate,
    })
}

fn delta_to_engine(d: wit::MailboxDelta) -> Result<MailboxDelta, EngineError> {
    let added = d
        .added
        .iter()
        .map(msgref_to_engine)
        .collect::<Result<_, _>>()?;
    let removed = d
        .removed
        .iter()
        .map(msgref_to_engine)
        .collect::<Result<_, _>>()?;
    let flag_changes = d
        .flag_changes
        .into_iter()
        .map(|(r, fs)| {
            Ok::<_, EngineError>((
                msgref_to_engine(&r)?,
                fs.into_iter().map(flag_to_engine).collect(),
            ))
        })
        .collect::<Result<_, _>>()?;
    Ok(MailboxDelta {
        added,
        removed,
        flag_changes,
        next_cursor: cursor_to_engine(d.next_cursor)?,
    })
}

fn change_to_engine(e: wit::ChangeEvent) -> ChangeEvent {
    match e {
        wit::ChangeEvent::MailboxChanged(m) => ChangeEvent::MailboxChanged {
            mailbox: mailbox_ref_to_engine(m),
        },
        wit::ChangeEvent::Disconnected => ChangeEvent::Disconnected,
    }
}

// ── error mapping ─────────────────────────────────────────────────────────────

/// Map a WIT `plugin-error` variant (returned in-band by the guest) → `PluginError`.
pub(crate) fn wit_to_plugin_err(e: wit::PluginError) -> PluginError {
    match e {
        wit::PluginError::LimitExceeded(m) => PluginError::LimitExceeded(m),
        wit::PluginError::CapabilityDenied(m) => PluginError::CapabilityDenied(m),
        wit::PluginError::Protocol(m)
        | wit::PluginError::Auth(m)
        | wit::PluginError::Transport(m)
        | wit::PluginError::Unsupported(m)
        | wit::PluginError::MailboxNotFound(m)
        | wit::PluginError::Other(m) => PluginError::Runtime(m),
    }
}

fn wit_to_engine(e: wit::PluginError) -> EngineError {
    match e {
        wit::PluginError::Protocol(m) => EngineError::Protocol(m),
        wit::PluginError::Auth(m) => EngineError::Auth(m),
        wit::PluginError::Transport(m) => EngineError::Transport(m),
        wit::PluginError::Unsupported(m) => EngineError::Unsupported(m),
        wit::PluginError::MailboxNotFound(m) => EngineError::MailboxNotFound(m),
        wit::PluginError::LimitExceeded(m) => {
            EngineError::Transport(format!("plugin limit exceeded: {m}"))
        }
        wit::PluginError::CapabilityDenied(m) => {
            EngineError::Unsupported(format!("plugin capability denied: {m}"))
        }
        wit::PluginError::Other(m) => EngineError::Protocol(m),
    }
}

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

/// Map a wasmtime trap during an account-backend call → `EngineError`, attributing
/// resource-limit trips (a plugin must never take the engine down).
fn trap_to_engine(store: &wasmtime::Store<HostState>, e: wasmtime::Error) -> EngineError {
    plugin_to_engine(crate::host_state::map_call_err(store, e))
}
