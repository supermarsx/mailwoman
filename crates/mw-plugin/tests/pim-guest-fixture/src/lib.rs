//! A minimal, real `wasm32-wasip2` guest **component** targeting `world plugin-pim`
//! (the t10 second world), used by the `mw-plugin` host PIM integration tests
//! (t10-e1). Because `plugin-pim` `include`s `world plugin`, this component ALSO
//! exports every `world plugin` interface (so the existing host loads it unchanged)
//! and ADDITIONALLY exports `calendar`/`tasks`/`bridge-parity`, which the host's
//! per-interface probe binds.
//!
//! It exercises the PIM-through-jail paths the host must enforce:
//! * `calendar` — `supports-calendar()->true`, a canned `list-calendars`, a
//!   `sync-events` that returns a one-event delta (proving the calendar delta
//!   round-trip) OR busy-loops forever when the calendar id is `"loop"` (so the host's
//!   epoch-deadline trip → typed `LimitExceeded` is proven on a PIM call).
//! * `bridge-parity` — reactions + recall SUPPORTED (round-trip proof); voting +
//!   focused UNSUPPORTED (`supports-*()->false` + `unsupported` — the honest negative).
//! * `tasks` — a minimal supported implementation.
//!
//! Every `world plugin` export is a trivial stub — a plugin only provides the hooks it
//! uses; the host only CALLS the capability-granted ones.

#![allow(clippy::empty_loop)]

wit_bindgen::generate!({
    world: "plugin-pim",
    path: "../../wit",
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

// ── world plugin (included) — trivial account-backend + hook stubs ────────────────

impl ab::Guest for Component {
    fn capabilities() -> Result<BackendCaps, PluginError> {
        Ok(BackendCaps {
            idle: true,
            move_cap: true,
            // Coarse account-level advertisement; the per-interface `supports-*()`
            // funcs are the authoritative signal the host probes.
            reactions: true,
            voting: false,
            recall: true,
            focused_sync: false,
        })
    }
    fn list_mailboxes() -> Result<Vec<Mailbox>, PluginError> {
        Ok(vec![Mailbox {
            mailbox_ref: MailboxRef {
                name: "INBOX".into(),
                uidvalidity: 1,
            },
            role: "inbox".into(),
            parent: None,
            total: 1,
            unread: 0,
        }])
    }
    fn sync_mailbox(_mbox: MailboxRef, cursor: SyncCursor) -> Result<MailboxDelta, PluginError> {
        Ok(MailboxDelta {
            added: vec![],
            removed: vec![],
            flag_changes: vec![],
            next_cursor: cursor,
        })
    }
    fn fetch_raw(refs: Vec<MessageRef>) -> Result<Vec<RawMessage>, PluginError> {
        Ok(refs
            .into_iter()
            .map(|r| RawMessage {
                message_ref: r,
                raw: b"Subject: hi\r\n\r\nbody\r\n".to_vec(),
                msg_flags: vec![Flag::Seen],
                internaldate: None,
            })
            .collect())
    }
    fn store_flags(
        _refs: Vec<MessageRef>,
        _add: Vec<Flag>,
        _remove: Vec<Flag>,
    ) -> Result<(), PluginError> {
        Ok(())
    }
    fn move_messages(_refs: Vec<MessageRef>, _to: MailboxRef) -> Result<(), PluginError> {
        Ok(())
    }
    fn submit(
        _mbox: MailboxRef,
        _raw: Vec<u8>,
        _flags: Vec<Flag>,
    ) -> Result<MessageRef, PluginError> {
        Err(PluginError::Unsupported("submit".into()))
    }
    fn poll_changes() -> Result<Vec<ChangeEvent>, PluginError> {
        Ok(vec![])
    }
}

impl addr::Guest for Component {
    fn search(_query: String) -> Result<Vec<String>, PluginError> {
        Ok(vec![])
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
impl pipe::Guest for Component {
    fn message_in(raw: Vec<u8>) -> Result<Vec<u8>, PluginError> {
        Ok(raw)
    }
    fn message_out(raw: Vec<u8>) -> Result<Vec<u8>, PluginError> {
        Ok(raw)
    }
}
impl spam::Guest for Component {
    fn classify(_raw: Vec<u8>) -> Result<String, PluginError> {
        Ok("ham".into())
    }
}

// ── calendar (SUPPORTED) ──────────────────────────────────────────────────────────

impl cal::Guest for Component {
    fn supports_calendar() -> bool {
        true
    }
    fn list_calendars() -> Result<Vec<cal::CalInfo>, PluginError> {
        Ok(vec![cal::CalInfo {
            id: "cal-1".into(),
            name: "Calendar".into(),
            role: "calendar".into(),
            read_only: false,
        }])
    }
    fn sync_events(calendar_id: String, cursor: SyncCursor) -> Result<cal::EventDelta, PluginError> {
        // The host's epoch deadline must preempt a runaway PIM call — proven by the
        // `"loop"` calendar id (never returns).
        if calendar_id == "loop" {
            loop {}
        }
        let _ = cursor;
        Ok(cal::EventDelta {
            changed: vec![cal::EventInfo {
                id: "evt-1".into(),
                calendar_id,
                ical: "BEGIN:VEVENT\r\nUID:evt-1\r\nSUMMARY:Standup\r\nEND:VEVENT\r\n".into(),
                start: Some("2026-07-14T09:00:00Z".into()),
                end: Some("2026-07-14T09:15:00Z".into()),
            }],
            removed: vec!["evt-old".into()],
            next_cursor: SyncCursor {
                opaque: b"cursor-2".to_vec(),
            },
        })
    }
    fn find_rooms() -> Result<Vec<cal::RoomInfo>, PluginError> {
        Ok(vec![cal::RoomInfo {
            address: "room-a@example.com".into(),
            name: "Room A".into(),
            capacity: Some(8),
        }])
    }
    fn get_schedule(_who: String, _start: String, _end: String) -> Result<String, PluginError> {
        Ok("BEGIN:VFREEBUSY\r\nEND:VFREEBUSY\r\n".into())
    }
}

// ── tasks (SUPPORTED) ─────────────────────────────────────────────────────────────

impl tasks_ex::Guest for Component {
    fn supports_tasks() -> bool {
        true
    }
    fn list_tasks() -> Result<Vec<tasks_ex::TaskInfo>, PluginError> {
        Ok(vec![tasks_ex::TaskInfo {
            id: "task-1".into(),
            list_id: "list-1".into(),
            ical: "BEGIN:VTODO\r\nUID:task-1\r\nSUMMARY:Do it\r\nEND:VTODO\r\n".into(),
            completed: false,
        }])
    }
    fn sync_tasks(_list_id: String, cursor: SyncCursor) -> Result<tasks_ex::TaskDelta, PluginError> {
        let _ = cursor;
        Ok(tasks_ex::TaskDelta {
            changed: vec![],
            removed: vec![],
            next_cursor: SyncCursor {
                opaque: b"task-cursor".to_vec(),
            },
        })
    }
    fn complete(_id: String) -> Result<(), PluginError> {
        Ok(())
    }
}

// ── bridge-parity (reactions + recall SUPPORTED; voting + focused UNSUPPORTED) ────

impl parity::Guest for Component {
    fn supports_reactions() -> bool {
        true
    }
    fn supports_voting() -> bool {
        false
    }
    fn supports_recall() -> bool {
        true
    }
    fn supports_focused() -> bool {
        false
    }

    fn set_reaction(_msg: MessageRef, _emoji: String, _add: bool) -> Result<(), PluginError> {
        Ok(())
    }
    fn get_reactions(_msg: MessageRef) -> Result<Vec<parity::Reaction>, PluginError> {
        Ok(vec![parity::Reaction {
            actor: "alice@example.com".into(),
            emoji: "👍".into(),
        }])
    }

    fn cast_vote(_msg: MessageRef, _choice: String) -> Result<(), PluginError> {
        Err(PluginError::Unsupported("voting".into()))
    }
    fn tally(_msg: MessageRef) -> Result<Vec<parity::VoteTally>, PluginError> {
        Ok(vec![])
    }

    fn recall(_msg: MessageRef) -> Result<parity::RecallOutcome, PluginError> {
        Ok(parity::RecallOutcome::Requested)
    }

    fn get_focused(_msg: MessageRef) -> Result<parity::FocusedState, PluginError> {
        Ok(parity::FocusedState::Other)
    }
    fn set_focused(_msg: MessageRef, _focused: bool) -> Result<(), PluginError> {
        Err(PluginError::Unsupported("focused".into()))
    }
}

export!(Component);
