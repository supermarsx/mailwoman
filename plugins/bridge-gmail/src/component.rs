//! The `wasm32-wasip2` component boundary (gated to `target_family = "wasm"`).
//!
//! Binds the `mailwoman:plugin` WIT `plugin-pim` world (the t10 second world; a
//! superset of `world plugin`), implements a [`Transport`] over the host imports
//! (`http-fetch` / `oauth-token` / `log`), and adapts the pure [`GmailBackend`] to
//! the `account-backend` guest export. The other `world plugin` exports are trivial
//! stubs (a plugin only provides the hooks it uses; the host calls only the
//! capability-granted ones — here, `account-backend` + `net`).
//!
//! ## PIM / Outlook-parity (t10, the HONEST negative path)
//! Targeting `plugin-pim` means the guest ALSO exports the three optional
//! `calendar` / `tasks` / `bridge-parity` interfaces so the host can PROBE them.
//! But the Gmail bridge is **mail-scoped** — its OAuth grant + REST surface cover
//! Gmail messages/labels only; Google Calendar and Google Tasks are SEPARATE Google
//! APIs this bridge neither authorizes nor talks to, and Gmail has none of the
//! Outlook-native reaction/voting/recall/Focused-Inbox features. So every
//! `supports-*()` here returns `false` (matching [`GmailBackend::capabilities`]) and
//! every data func returns `unsupported` (recall ⇒ `recall-outcome::unsupported`,
//! the §10.3 honesty variant). The host binds the interfaces but the honest
//! `supports-*() -> false` keeps `Engine::bridge_*` `None` ⇒ the engine stays on its
//! byte-unchanged standards fallback (native CalDAV/CardDAV). Honesty over
//! feature-count: no Google API integration is fabricated here.

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

use mailwoman::plugin::host;
use mailwoman::plugin::types as wit;

use crate::backend::{GmailBackend, HttpResponse, Transport};
use crate::model;

struct Component;

/// The host-mediated transport: every call routes through a gated host import, so
/// the guest opens no socket and never sees the OAuth secret.
struct HostTransport;

impl Transport for HostTransport {
    fn oauth_token(&self, account: &str) -> model::Result<String> {
        host::oauth_token(account).map_err(wit_err_to_bridge)
    }

    fn http(
        &self,
        method: &str,
        url: &str,
        headers: Vec<(String, String)>,
        body: Option<Vec<u8>>,
    ) -> model::Result<HttpResponse> {
        let req = host::HttpRequest {
            method: method.to_string(),
            url: url.to_string(),
            headers,
            body,
        };
        let resp = host::http_fetch(&req).map_err(wit_err_to_bridge)?;
        Ok(HttpResponse {
            status: resp.status,
            body: resp.body,
        })
    }

    fn log(&self, msg: &str) {
        // No-content floor: only an opaque diagnostic string, never mail content.
        host::log(host::LogLevel::Debug, msg);
    }
}

fn backend() -> GmailBackend<HostTransport> {
    GmailBackend::new(HostTransport)
}

impl ab::Guest for Component {
    fn capabilities() -> Result<wit::BackendCaps, wit::PluginError> {
        backend()
            .capabilities()
            .map(caps_to_wit)
            .map_err(bridge_to_wit)
    }

    fn list_mailboxes() -> Result<Vec<wit::Mailbox>, wit::PluginError> {
        backend()
            .list_mailboxes()
            .map(|v| v.into_iter().map(mailbox_to_wit).collect())
            .map_err(bridge_to_wit)
    }

    fn sync_mailbox(
        mbox: wit::MailboxRef,
        cursor: wit::SyncCursor,
    ) -> Result<wit::MailboxDelta, wit::PluginError> {
        let m = mailboxref_from_wit(mbox);
        let c = model::SyncCursor {
            opaque: cursor.opaque,
        };
        backend()
            .sync_mailbox(&m, &c)
            .map(delta_to_wit)
            .map_err(bridge_to_wit)
    }

    fn fetch_raw(refs: Vec<wit::MessageRef>) -> Result<Vec<wit::RawMessage>, wit::PluginError> {
        let refs: Vec<model::MessageRef> = refs.into_iter().map(msgref_from_wit).collect();
        backend()
            .fetch_raw(&refs)
            .map(|v| v.into_iter().map(rawmsg_to_wit).collect())
            .map_err(bridge_to_wit)
    }

    fn store_flags(
        refs: Vec<wit::MessageRef>,
        add: Vec<wit::Flag>,
        remove: Vec<wit::Flag>,
    ) -> Result<(), wit::PluginError> {
        let refs: Vec<model::MessageRef> = refs.into_iter().map(msgref_from_wit).collect();
        let add: Vec<model::Flag> = add.into_iter().map(flag_from_wit).collect();
        let remove: Vec<model::Flag> = remove.into_iter().map(flag_from_wit).collect();
        backend()
            .store_flags(&refs, &add, &remove)
            .map_err(bridge_to_wit)
    }

    fn move_messages(
        refs: Vec<wit::MessageRef>,
        to: wit::MailboxRef,
    ) -> Result<(), wit::PluginError> {
        let refs: Vec<model::MessageRef> = refs.into_iter().map(msgref_from_wit).collect();
        let to = mailboxref_from_wit(to);
        backend().move_messages(&refs, &to).map_err(bridge_to_wit)
    }

    fn submit(
        mbox: wit::MailboxRef,
        raw: Vec<u8>,
        msg_flags: Vec<wit::Flag>,
    ) -> Result<wit::MessageRef, wit::PluginError> {
        let m = mailboxref_from_wit(mbox);
        let flags: Vec<model::Flag> = msg_flags.into_iter().map(flag_from_wit).collect();
        backend()
            .submit(&m, &raw, &flags)
            .map(msgref_to_wit)
            .map_err(bridge_to_wit)
    }

    fn poll_changes() -> Result<Vec<wit::ChangeEvent>, wit::PluginError> {
        backend()
            .poll_changes()
            .map(|v| v.into_iter().map(change_to_wit).collect())
            .map_err(bridge_to_wit)
    }
}

// ── unused exports of the world — trivial stubs ─────────────────────────────────

impl addr::Guest for Component {
    fn search(_query: String) -> Result<Vec<String>, wit::PluginError> {
        Err(wit::PluginError::Unsupported("addrbook-source".into()))
    }
}
impl autoc::Guest for Component {
    fn lookup(_email: String) -> Result<Option<String>, wit::PluginError> {
        Ok(None)
    }
}
impl dlp::Guest for Component {
    fn detect(_body: Vec<u8>) -> Result<Vec<String>, wit::PluginError> {
        Ok(Vec::new())
    }
}
impl pipe::Guest for Component {
    fn message_in(raw: Vec<u8>) -> Result<Vec<u8>, wit::PluginError> {
        Ok(raw)
    }
    fn message_out(raw: Vec<u8>) -> Result<Vec<u8>, wit::PluginError> {
        Ok(raw)
    }
}
impl spam::Guest for Component {
    fn classify(_raw: Vec<u8>) -> Result<String, wit::PluginError> {
        Ok("ham".into())
    }
}

// ── PIM / Outlook-parity (t10) — HONEST mail-only: everything unsupported ─────────
//
// The Gmail bridge exports these three interfaces so the host can PROBE them, but it
// genuinely provides NONE of them: Google Calendar / Google Tasks are separate Google
// APIs outside this bridge's OAuth scope + REST surface, and Gmail has no Outlook
// reaction/voting/recall/Focused-Inbox parity. Each `supports-*()` therefore returns
// `false` (matching `GmailBackend::capabilities`) and each data func returns
// `unsupported`, so the host keeps the engine on its standards fallback. Do NOT
// fabricate calendar/tasks support the Gmail bridge does not have.

/// The one honest reason string every unsupported PIM func returns.
const NO_PIM: &str = "bridge-gmail is mail-only: no calendar/tasks/parity support";

impl cal::Guest for Component {
    fn supports_calendar() -> bool {
        false
    }
    fn list_calendars() -> Result<Vec<cal::CalInfo>, wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }
    fn sync_events(
        _calendar_id: String,
        _cursor: wit::SyncCursor,
    ) -> Result<cal::EventDelta, wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }
    fn find_rooms() -> Result<Vec<cal::RoomInfo>, wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }
    fn get_schedule(
        _who: String,
        _start: String,
        _end: String,
    ) -> Result<String, wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }
}

impl tasks_ex::Guest for Component {
    fn supports_tasks() -> bool {
        false
    }
    fn list_tasks() -> Result<Vec<tasks_ex::TaskInfo>, wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }
    fn sync_tasks(
        _list_id: String,
        _cursor: wit::SyncCursor,
    ) -> Result<tasks_ex::TaskDelta, wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }
    fn complete(_id: String) -> Result<(), wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
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

    fn set_reaction(
        _msg: wit::MessageRef,
        _emoji: String,
        _add: bool,
    ) -> Result<(), wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }
    fn get_reactions(_msg: wit::MessageRef) -> Result<Vec<parity::Reaction>, wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }

    fn cast_vote(_msg: wit::MessageRef, _choice: String) -> Result<(), wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }
    fn tally(_msg: wit::MessageRef) -> Result<Vec<parity::VoteTally>, wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }

    // Gmail has no server-side recall — the honest §10.3 outcome is `unsupported`,
    // NOT `failed` (nothing was attempted or lost) and NOT `requested`.
    fn recall(_msg: wit::MessageRef) -> Result<parity::RecallOutcome, wit::PluginError> {
        Ok(parity::RecallOutcome::Unsupported)
    }

    fn get_focused(_msg: wit::MessageRef) -> Result<parity::FocusedState, wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }
    fn set_focused(_msg: wit::MessageRef, _focused: bool) -> Result<(), wit::PluginError> {
        Err(wit::PluginError::Unsupported(NO_PIM.into()))
    }
}

// ── conversions: model ↔ WIT ────────────────────────────────────────────────────

fn caps_to_wit(c: model::BackendCaps) -> wit::BackendCaps {
    wit::BackendCaps {
        idle: c.idle,
        move_cap: c.move_cap,
        reactions: c.reactions,
        voting: c.voting,
        recall: c.recall,
        focused_sync: c.focused_sync,
    }
}

fn mailboxref_to_wit(m: model::MailboxRef) -> wit::MailboxRef {
    wit::MailboxRef {
        name: m.name,
        uidvalidity: m.uidvalidity,
    }
}

fn mailboxref_from_wit(m: wit::MailboxRef) -> model::MailboxRef {
    model::MailboxRef {
        name: m.name,
        uidvalidity: m.uidvalidity,
    }
}

fn mailbox_to_wit(m: model::Mailbox) -> wit::Mailbox {
    wit::Mailbox {
        mailbox_ref: mailboxref_to_wit(m.mailbox_ref),
        role: m.role,
        parent: m.parent,
        total: m.total,
        unread: m.unread,
    }
}

fn msgref_to_wit(r: model::MessageRef) -> wit::MessageRef {
    wit::MessageRef {
        raw: r.raw,
        mailbox: mailboxref_to_wit(r.mailbox),
    }
}

fn msgref_from_wit(r: wit::MessageRef) -> model::MessageRef {
    model::MessageRef {
        raw: r.raw,
        mailbox: mailboxref_from_wit(r.mailbox),
    }
}

fn flag_to_wit(f: model::Flag) -> wit::Flag {
    match f {
        model::Flag::Seen => wit::Flag::Seen,
        model::Flag::Answered => wit::Flag::Answered,
        model::Flag::Flagged => wit::Flag::Flagged,
        model::Flag::Deleted => wit::Flag::Deleted,
        model::Flag::Draft => wit::Flag::Draft,
        model::Flag::Recent => wit::Flag::Recent,
        model::Flag::Keyword(k) => wit::Flag::Keyword(k),
    }
}

fn flag_from_wit(f: wit::Flag) -> model::Flag {
    match f {
        wit::Flag::Seen => model::Flag::Seen,
        wit::Flag::Answered => model::Flag::Answered,
        wit::Flag::Flagged => model::Flag::Flagged,
        wit::Flag::Deleted => model::Flag::Deleted,
        wit::Flag::Draft => model::Flag::Draft,
        wit::Flag::Recent => model::Flag::Recent,
        wit::Flag::Keyword(k) => model::Flag::Keyword(k),
    }
}

fn rawmsg_to_wit(m: model::RawMessage) -> wit::RawMessage {
    wit::RawMessage {
        message_ref: msgref_to_wit(m.message_ref),
        raw: m.raw,
        msg_flags: m.msg_flags.into_iter().map(flag_to_wit).collect(),
        internaldate: m.internaldate,
    }
}

fn cursor_to_wit(c: model::SyncCursor) -> wit::SyncCursor {
    wit::SyncCursor { opaque: c.opaque }
}

fn delta_to_wit(d: model::MailboxDelta) -> wit::MailboxDelta {
    wit::MailboxDelta {
        added: d.added.into_iter().map(msgref_to_wit).collect(),
        removed: d.removed.into_iter().map(msgref_to_wit).collect(),
        flag_changes: d
            .flag_changes
            .into_iter()
            .map(|(r, fs)| (msgref_to_wit(r), fs.into_iter().map(flag_to_wit).collect()))
            .collect(),
        next_cursor: cursor_to_wit(d.next_cursor),
    }
}

fn change_to_wit(e: model::ChangeEvent) -> wit::ChangeEvent {
    match e {
        model::ChangeEvent::MailboxChanged(m) => {
            wit::ChangeEvent::MailboxChanged(mailboxref_to_wit(m))
        }
        model::ChangeEvent::Disconnected => wit::ChangeEvent::Disconnected,
    }
}

fn bridge_to_wit(e: model::BridgeError) -> wit::PluginError {
    match e {
        model::BridgeError::Protocol(m) => wit::PluginError::Protocol(m),
        model::BridgeError::Auth(m) => wit::PluginError::Auth(m),
        model::BridgeError::Transport(m) => wit::PluginError::Transport(m),
        model::BridgeError::Unsupported(m) => wit::PluginError::Unsupported(m),
        model::BridgeError::MailboxNotFound(m) => wit::PluginError::MailboxNotFound(m),
        model::BridgeError::Other(m) => wit::PluginError::Other(m),
    }
}

fn wit_err_to_bridge(e: wit::PluginError) -> model::BridgeError {
    match e {
        wit::PluginError::Protocol(m) => model::BridgeError::Protocol(m),
        wit::PluginError::Auth(m) => model::BridgeError::Auth(m),
        wit::PluginError::Transport(m) => model::BridgeError::Transport(m),
        wit::PluginError::Unsupported(m) => model::BridgeError::Unsupported(m),
        wit::PluginError::MailboxNotFound(m) => model::BridgeError::MailboxNotFound(m),
        wit::PluginError::LimitExceeded(m) | wit::PluginError::CapabilityDenied(m) => {
            model::BridgeError::Transport(m)
        }
        wit::PluginError::Other(m) => model::BridgeError::Other(m),
    }
}

export!(Component);
