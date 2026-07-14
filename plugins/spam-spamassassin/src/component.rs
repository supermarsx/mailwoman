//! The `wasm32-wasip2` guest component (t10-e0 scaffold; e6 fills). Compiled ONLY
//! for wasm (gated by `lib.rs`).
//!
//! `spam-action::classify` is the real hook (spamd over host `http-fetch`); every
//! other export in the frozen `plugin` world — including the optional PIM/parity
//! `calendar`/`tasks`/`bridge-parity` interfaces — is a trivial stub advertising no
//! support, so the host's per-interface probe binds nothing.

wit_bindgen::generate!({
    world: "plugin-pim",
    path: "../../crates/mw-plugin/wit",
});

use exports::mailwoman::plugin::account_backend as ab;
use exports::mailwoman::plugin::addrbook_source as addr;
use exports::mailwoman::plugin::autoconfig_source as autoc;
use exports::mailwoman::plugin::bridge_parity as parity;
use exports::mailwoman::plugin::calendar as cal;
use exports::mailwoman::plugin::dlp_detect as dlp;
use exports::mailwoman::plugin::message_pipeline as pipe;
use exports::mailwoman::plugin::spam_action as spam;
use exports::mailwoman::plugin::tasks as tasks_ex;

use mailwoman::plugin::types::{
    BackendCaps, ChangeEvent, Flag, Mailbox, MailboxDelta, MailboxRef, MessageRef, PluginError,
    RawMessage, SyncCursor,
};

struct Component;

/// A uniform "this hook is a scaffold stub" error.
fn stub(what: &str) -> PluginError {
    PluginError::Unsupported(format!(
        "{what} not implemented in the spam-spamassassin scaffold (t10-e0)"
    ))
}

// ── The real hook: spamd classify/train over host-mediated HTTP (e6 fills) ──────

impl spam::Guest for Component {
    fn classify(_raw: Vec<u8>) -> Result<String, PluginError> {
        Err(stub("spam-action classify"))
    }
}

// ── Trivial stubs for the rest of the world ─────────────────────────────────────

impl ab::Guest for Component {
    fn capabilities() -> Result<BackendCaps, PluginError> {
        Err(stub("account-backend"))
    }
    fn list_mailboxes() -> Result<Vec<Mailbox>, PluginError> {
        Err(stub("account-backend"))
    }
    fn sync_mailbox(_mbox: MailboxRef, _cursor: SyncCursor) -> Result<MailboxDelta, PluginError> {
        Err(stub("account-backend"))
    }
    fn fetch_raw(_refs: Vec<MessageRef>) -> Result<Vec<RawMessage>, PluginError> {
        Err(stub("account-backend"))
    }
    fn store_flags(
        _refs: Vec<MessageRef>,
        _add: Vec<Flag>,
        _remove: Vec<Flag>,
    ) -> Result<(), PluginError> {
        Err(stub("account-backend"))
    }
    fn move_messages(_refs: Vec<MessageRef>, _to: MailboxRef) -> Result<(), PluginError> {
        Err(stub("account-backend"))
    }
    fn submit(
        _mbox: MailboxRef,
        _raw: Vec<u8>,
        _flags: Vec<Flag>,
    ) -> Result<MessageRef, PluginError> {
        Err(stub("account-backend"))
    }
    fn poll_changes() -> Result<Vec<ChangeEvent>, PluginError> {
        Err(stub("account-backend"))
    }
}

impl pipe::Guest for Component {
    fn message_in(raw: Vec<u8>) -> Result<Vec<u8>, PluginError> {
        Ok(raw)
    }
    fn message_out(raw: Vec<u8>) -> Result<Vec<u8>, PluginError> {
        Ok(raw)
    }
}

impl addr::Guest for Component {
    fn search(_query: String) -> Result<Vec<String>, PluginError> {
        Err(stub("addrbook-source"))
    }
}

impl autoc::Guest for Component {
    fn lookup(_email: String) -> Result<Option<String>, PluginError> {
        Ok(None)
    }
}

impl dlp::Guest for Component {
    fn detect(_body: Vec<u8>) -> Result<Vec<String>, PluginError> {
        Ok(vec![])
    }
}

// ── optional PIM/parity stubs (advertise no support → host binds nothing) ─

impl cal::Guest for Component {
    fn supports_calendar() -> bool {
        false
    }
    fn list_calendars() -> Result<Vec<cal::CalInfo>, PluginError> {
        Err(stub("calendar"))
    }
    fn sync_events(
        _calendar_id: String,
        _cursor: SyncCursor,
    ) -> Result<cal::EventDelta, PluginError> {
        Err(stub("calendar"))
    }
    fn find_rooms() -> Result<Vec<cal::RoomInfo>, PluginError> {
        Err(stub("calendar"))
    }
    fn get_schedule(_who: String, _start: String, _end: String) -> Result<String, PluginError> {
        Err(stub("calendar"))
    }
}

impl tasks_ex::Guest for Component {
    fn supports_tasks() -> bool {
        false
    }
    fn list_tasks() -> Result<Vec<tasks_ex::TaskInfo>, PluginError> {
        Err(stub("tasks"))
    }
    fn sync_tasks(
        _list_id: String,
        _cursor: SyncCursor,
    ) -> Result<tasks_ex::TaskDelta, PluginError> {
        Err(stub("tasks"))
    }
    fn complete(_id: String) -> Result<(), PluginError> {
        Err(stub("tasks"))
    }
}

impl parity::Guest for Component {
    fn supports_reactions() -> bool {
        false
    }
    fn supports_voting() -> bool {
        false
    }
    fn supports_recall() -> bool {
        false
    }
    fn supports_focused() -> bool {
        false
    }
    fn set_reaction(_msg: MessageRef, _emoji: String, _add: bool) -> Result<(), PluginError> {
        Err(stub("bridge-parity"))
    }
    fn get_reactions(_msg: MessageRef) -> Result<Vec<parity::Reaction>, PluginError> {
        Err(stub("bridge-parity"))
    }
    fn cast_vote(_msg: MessageRef, _choice: String) -> Result<(), PluginError> {
        Err(stub("bridge-parity"))
    }
    fn tally(_msg: MessageRef) -> Result<Vec<parity::VoteTally>, PluginError> {
        Err(stub("bridge-parity"))
    }
    fn recall(_msg: MessageRef) -> Result<parity::RecallOutcome, PluginError> {
        Err(stub("bridge-parity"))
    }
    fn get_focused(_msg: MessageRef) -> Result<parity::FocusedState, PluginError> {
        Err(stub("bridge-parity"))
    }
    fn set_focused(_msg: MessageRef, _focused: bool) -> Result<(), PluginError> {
        Err(stub("bridge-parity"))
    }
}

export!(Component);
