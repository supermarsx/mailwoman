//! The `wasm32-wasip2` component boundary (gated to `target_family = "wasm"`).
//!
//! Binds the frozen `mailwoman:plugin` WIT world, implements a [`Transport`] over
//! the host imports (`http-fetch` / `oauth-token` / `log`), and adapts the pure
//! [`GmailBackend`] to the `account-backend` guest export. The other exports in the
//! world are trivial stubs (a plugin only provides the hooks it uses; the host
//! calls only the capability-granted ones — here, `account-backend` + `net`).

wit_bindgen::generate!({
    world: "plugin",
    path: "../../crates/mw-plugin/wit",
});

use exports::mailwoman::plugin::account_backend as ab;
use exports::mailwoman::plugin::addrbook_source as addr;
use exports::mailwoman::plugin::autoconfig_source as autoc;
use exports::mailwoman::plugin::dlp_detect as dlp;
use exports::mailwoman::plugin::message_pipeline as pipe;
use exports::mailwoman::plugin::spam_action as spam;

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
