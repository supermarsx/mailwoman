//! The `wasm32-wasip2` guest component (t10-e6). Compiled ONLY for wasm (gated by
//! `lib.rs`) so the host build never sees the wasm-import extern blocks.
//!
//! `spam-action::classify` is the real hook: it frames the message as a SPAMC/1.5
//! `SYMBOLS` request ([`crate::build_spamc_request`]), ships it to `spamd` over the host
//! `http-fetch` byte transport (net-allowlisted), and maps the SPAMD reply to the verdict
//! contract via [`crate::classify_spamc`]. It is **fail-soft** — an unreachable daemon, a
//! denied host, or any transport error resolves to [`crate::VERDICT_UNKNOWN`] (`Ok`,
//! never `Err`, never a panic, never a hard block).
//!
//! Every other export in the frozen world — including the optional PIM/parity interfaces
//! — is a trivial stub advertising no support, so the host binds nothing for them.

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
use mailwoman::plugin::types::{
    BackendCaps, ChangeEvent, Flag, Mailbox, MailboxDelta, MailboxRef, MessageRef, PluginError,
    RawMessage, SyncCursor,
};

struct Component;

fn unsupported(what: &str) -> PluginError {
    PluginError::Unsupported(format!("{what} not implemented by spam-spamassassin"))
}

/// The `spamd` endpoint (`host:port`): the `endpoint` KV config value if the admin set
/// one (and `store:kv-scoped` is granted), else the compiled default. The host still
/// enforces the `net_allowlist` on the resulting host regardless of this value.
fn endpoint() -> String {
    match host::kv_get("endpoint") {
        Some(bytes) => match String::from_utf8(bytes) {
            Ok(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => crate::DEFAULT_ENDPOINT.to_string(),
        },
        None => crate::DEFAULT_ENDPOINT.to_string(),
    }
}

fn err_note(e: &PluginError) -> String {
    match e {
        PluginError::CapabilityDenied(m) => format!("denied: {m}"),
        PluginError::Transport(m) => format!("transport: {m}"),
        PluginError::Auth(m) => format!("auth: {m}"),
        PluginError::Protocol(m) => format!("protocol: {m}"),
        PluginError::Unsupported(m) => format!("unsupported: {m}"),
        PluginError::MailboxNotFound(m) => format!("not-found: {m}"),
        PluginError::LimitExceeded(m) => format!("limit: {m}"),
        PluginError::Other(m) => format!("error: {m}"),
    }
}

// ── The real hook: SPAMC/1.5 over the host-mediated byte transport (fail-soft) ─────

impl spam::Guest for Component {
    fn classify(raw: Vec<u8>) -> Result<String, PluginError> {
        // The SPAMC request frame IS the transport payload; the host fetcher carries it
        // to `spamd` (raw TCP :783 via a relay, per the module transport note).
        let frame = crate::build_spamc_request("SYMBOLS", &raw);
        let req = host::HttpRequest {
            method: "POST".into(),
            url: format!("http://{}/", endpoint()),
            headers: vec![("content-type".into(), "application/octet-stream".into())],
            body: Some(frame),
        };
        // Fail-soft: `capability-denied` (net not granted / host outside the allowlist)
        // and any transport error map to an explicit UNKNOWN verdict — never a hard
        // block, never a panic.
        match host::http_fetch(&req) {
            Ok(resp) => Ok(crate::classify_spamc(&resp.body)),
            Err(e) => Ok(crate::unknown_verdict(&format!(
                "spamd unreachable: {}",
                err_note(&e)
            ))),
        }
    }
}

// ── Trivial stubs for the rest of the world ─────────────────────────────────────

impl ab::Guest for Component {
    fn capabilities() -> Result<BackendCaps, PluginError> {
        Err(unsupported("account-backend"))
    }
    fn list_mailboxes() -> Result<Vec<Mailbox>, PluginError> {
        Err(unsupported("account-backend"))
    }
    fn sync_mailbox(_mbox: MailboxRef, _cursor: SyncCursor) -> Result<MailboxDelta, PluginError> {
        Err(unsupported("account-backend"))
    }
    fn fetch_raw(_refs: Vec<MessageRef>) -> Result<Vec<RawMessage>, PluginError> {
        Err(unsupported("account-backend"))
    }
    fn store_flags(
        _refs: Vec<MessageRef>,
        _add: Vec<Flag>,
        _remove: Vec<Flag>,
    ) -> Result<(), PluginError> {
        Err(unsupported("account-backend"))
    }
    fn move_messages(_refs: Vec<MessageRef>, _to: MailboxRef) -> Result<(), PluginError> {
        Err(unsupported("account-backend"))
    }
    fn submit(
        _mbox: MailboxRef,
        _raw: Vec<u8>,
        _flags: Vec<Flag>,
    ) -> Result<MessageRef, PluginError> {
        Err(unsupported("account-backend"))
    }
    fn poll_changes() -> Result<Vec<ChangeEvent>, PluginError> {
        Err(unsupported("account-backend"))
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
        Err(unsupported("addrbook-source"))
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
        Err(unsupported("calendar"))
    }
    fn sync_events(
        _calendar_id: String,
        _cursor: SyncCursor,
    ) -> Result<cal::EventDelta, PluginError> {
        Err(unsupported("calendar"))
    }
    fn find_rooms() -> Result<Vec<cal::RoomInfo>, PluginError> {
        Err(unsupported("calendar"))
    }
    fn get_schedule(_who: String, _start: String, _end: String) -> Result<String, PluginError> {
        Err(unsupported("calendar"))
    }
}

impl tasks_ex::Guest for Component {
    fn supports_tasks() -> bool {
        false
    }
    fn list_tasks() -> Result<Vec<tasks_ex::TaskInfo>, PluginError> {
        Err(unsupported("tasks"))
    }
    fn sync_tasks(
        _list_id: String,
        _cursor: SyncCursor,
    ) -> Result<tasks_ex::TaskDelta, PluginError> {
        Err(unsupported("tasks"))
    }
    fn complete(_id: String) -> Result<(), PluginError> {
        Err(unsupported("tasks"))
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
        Err(unsupported("bridge-parity"))
    }
    fn get_reactions(_msg: MessageRef) -> Result<Vec<parity::Reaction>, PluginError> {
        Err(unsupported("bridge-parity"))
    }
    fn cast_vote(_msg: MessageRef, _choice: String) -> Result<(), PluginError> {
        Err(unsupported("bridge-parity"))
    }
    fn tally(_msg: MessageRef) -> Result<Vec<parity::VoteTally>, PluginError> {
        Err(unsupported("bridge-parity"))
    }
    fn recall(_msg: MessageRef) -> Result<parity::RecallOutcome, PluginError> {
        Err(unsupported("bridge-parity"))
    }
    fn get_focused(_msg: MessageRef) -> Result<parity::FocusedState, PluginError> {
        Err(unsupported("bridge-parity"))
    }
    fn set_focused(_msg: MessageRef, _focused: bool) -> Result<(), PluginError> {
        Err(unsupported("bridge-parity"))
    }
}

export!(Component);
