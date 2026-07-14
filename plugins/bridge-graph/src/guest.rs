//! The `wasm32-wasip2` component glue: wires the pure Graph mapping to the frozen
//! `mailwoman:plugin` WIT exports over the gated host imports. Compiled ONLY for the
//! guest target (`cfg(target_arch = "wasm32")`), so the host-target lib view stays
//! pure Rust. A guest opens no socket and holds no long-lived credential — it asks
//! the host for a transient token per call and hands every request to `http-fetch`.

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

use crate::graph::{BridgeError, GraphClient, HttpRequestSpec, HttpResponseData, Transport};
use crate::types as t;
use crate::{caps, contacts, mail};

/// The account the bridge is bound to. The host maps this to the concrete account
/// and holds/refreshes its OAuth secret; the guest passes it opaquely.
const ACCOUNT: &str = "";

struct Component;

// ── Transport over the gated host imports ─────────────────────────────────────

struct HostTransport;

impl Transport for HostTransport {
    fn token(&self, account: &str) -> crate::graph::Result<String> {
        host::oauth_token(account).map_err(wit_err_to_bridge)
    }

    fn fetch(&self, req: HttpRequestSpec) -> crate::graph::Result<HttpResponseData> {
        let hr = host::HttpRequest {
            method: req.method,
            url: req.url,
            headers: req.headers,
            body: req.body,
        };
        let resp = host::http_fetch(&hr).map_err(wit_err_to_bridge)?;
        Ok(HttpResponseData {
            status: resp.status,
            headers: resp.headers,
            body: resp.body,
        })
    }
}

fn client() -> (HostTransport, &'static str) {
    (HostTransport, ACCOUNT)
}

// ── account-backend export ────────────────────────────────────────────────────

impl ab::Guest for Component {
    fn capabilities() -> Result<wit::BackendCaps, wit::PluginError> {
        Ok(caps_to_wit(caps::backend_caps()))
    }

    fn list_mailboxes() -> Result<Vec<wit::Mailbox>, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        mail::list_mailboxes(&c)
            .map(|v| v.into_iter().map(mailbox_to_wit).collect())
            .map_err(bridge_to_wit)
    }

    fn sync_mailbox(
        mbox: wit::MailboxRef,
        cursor: wit::SyncCursor,
    ) -> Result<wit::MailboxDelta, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        mail::sync_mailbox(&c, &mailbox_ref_from_wit(&mbox), &cursor_from_wit(cursor))
            .map(delta_to_wit)
            .map_err(bridge_to_wit)
    }

    fn fetch_raw(refs: Vec<wit::MessageRef>) -> Result<Vec<wit::RawMessage>, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        let plain: Vec<t::MessageRef> = refs.iter().map(msgref_from_wit).collect();
        mail::fetch_raw(&c, &plain)
            .map(|v| v.into_iter().map(rawmsg_to_wit).collect())
            .map_err(bridge_to_wit)
    }

    fn store_flags(
        refs: Vec<wit::MessageRef>,
        add: Vec<wit::Flag>,
        remove: Vec<wit::Flag>,
    ) -> Result<(), wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        let plain: Vec<t::MessageRef> = refs.iter().map(msgref_from_wit).collect();
        let add: Vec<t::Flag> = add.into_iter().map(flag_from_wit).collect();
        let remove: Vec<t::Flag> = remove.into_iter().map(flag_from_wit).collect();
        mail::store_flags(&c, &plain, &add, &remove).map_err(bridge_to_wit)
    }

    fn move_messages(
        refs: Vec<wit::MessageRef>,
        to: wit::MailboxRef,
    ) -> Result<(), wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        let plain: Vec<t::MessageRef> = refs.iter().map(msgref_from_wit).collect();
        mail::move_messages(&c, &plain, &mailbox_ref_from_wit(&to)).map_err(bridge_to_wit)
    }

    fn submit(
        mbox: wit::MailboxRef,
        raw: Vec<u8>,
        _msg_flags: Vec<wit::Flag>,
    ) -> Result<wit::MessageRef, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        mail::submit(&c, &mailbox_ref_from_wit(&mbox), &raw)
            .map(msgref_to_wit)
            .map_err(bridge_to_wit)
    }

    fn poll_changes() -> Result<Vec<wit::ChangeEvent>, wit::PluginError> {
        Ok(mail::poll_changes()
            .into_iter()
            .map(change_to_wit)
            .collect())
    }
}

// ── addrbook-source export (contacts + GAL) ───────────────────────────────────

impl addr::Guest for Component {
    fn search(query: String) -> Result<Vec<String>, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        contacts::addrbook_search(&c, &query).map_err(bridge_to_wit)
    }
}

// ── unused exports (the host only calls capability-granted ones) ──────────────

impl pipe::Guest for Component {
    fn message_in(raw: Vec<u8>) -> Result<Vec<u8>, wit::PluginError> {
        Ok(raw)
    }
    fn message_out(raw: Vec<u8>) -> Result<Vec<u8>, wit::PluginError> {
        Ok(raw)
    }
}

impl autoc::Guest for Component {
    fn lookup(_email: String) -> Result<Option<String>, wit::PluginError> {
        Ok(None)
    }
}

impl dlp::Guest for Component {
    fn detect(_body: Vec<u8>) -> Result<Vec<String>, wit::PluginError> {
        Ok(vec![])
    }
}

impl spam::Guest for Component {
    fn classify(_raw: Vec<u8>) -> Result<String, wit::PluginError> {
        Ok("ham".into())
    }
}

// ── conversions: plain ↔ WIT ──────────────────────────────────────────────────

fn caps_to_wit(c: t::BackendCaps) -> wit::BackendCaps {
    wit::BackendCaps {
        idle: c.idle,
        move_cap: c.move_cap,
        reactions: c.reactions,
        voting: c.voting,
        recall: c.recall,
        focused_sync: c.focused_sync,
    }
}

fn mailbox_ref_to_wit(m: t::MailboxRef) -> wit::MailboxRef {
    wit::MailboxRef {
        name: m.name,
        uidvalidity: m.uidvalidity,
    }
}

fn mailbox_ref_from_wit(m: &wit::MailboxRef) -> t::MailboxRef {
    t::MailboxRef {
        name: m.name.clone(),
        uidvalidity: m.uidvalidity,
    }
}

fn mailbox_to_wit(m: t::Mailbox) -> wit::Mailbox {
    wit::Mailbox {
        mailbox_ref: mailbox_ref_to_wit(m.mailbox_ref),
        role: m.role,
        parent: m.parent,
        total: m.total,
        unread: m.unread,
    }
}

fn flag_to_wit(f: t::Flag) -> wit::Flag {
    match f {
        t::Flag::Seen => wit::Flag::Seen,
        t::Flag::Answered => wit::Flag::Answered,
        t::Flag::Flagged => wit::Flag::Flagged,
        t::Flag::Deleted => wit::Flag::Deleted,
        t::Flag::Draft => wit::Flag::Draft,
        t::Flag::Recent => wit::Flag::Recent,
        t::Flag::Keyword(k) => wit::Flag::Keyword(k),
    }
}

fn flag_from_wit(f: wit::Flag) -> t::Flag {
    match f {
        wit::Flag::Seen => t::Flag::Seen,
        wit::Flag::Answered => t::Flag::Answered,
        wit::Flag::Flagged => t::Flag::Flagged,
        wit::Flag::Deleted => t::Flag::Deleted,
        wit::Flag::Draft => t::Flag::Draft,
        wit::Flag::Recent => t::Flag::Recent,
        wit::Flag::Keyword(k) => t::Flag::Keyword(k),
    }
}

fn msgref_to_wit(m: t::MessageRef) -> wit::MessageRef {
    wit::MessageRef {
        raw: m.raw,
        mailbox: mailbox_ref_to_wit(m.mailbox),
    }
}

fn msgref_from_wit(m: &wit::MessageRef) -> t::MessageRef {
    t::MessageRef {
        raw: m.raw.clone(),
        mailbox: mailbox_ref_from_wit(&m.mailbox),
    }
}

fn rawmsg_to_wit(m: t::RawMessage) -> wit::RawMessage {
    wit::RawMessage {
        message_ref: msgref_to_wit(m.message_ref),
        raw: m.raw,
        msg_flags: m.msg_flags.into_iter().map(flag_to_wit).collect(),
        internaldate: m.internaldate,
    }
}

fn cursor_from_wit(c: wit::SyncCursor) -> t::SyncCursor {
    t::SyncCursor { opaque: c.opaque }
}

fn cursor_to_wit(c: t::SyncCursor) -> wit::SyncCursor {
    wit::SyncCursor { opaque: c.opaque }
}

fn delta_to_wit(d: t::MailboxDelta) -> wit::MailboxDelta {
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

fn change_to_wit(e: t::ChangeEvent) -> wit::ChangeEvent {
    match e {
        t::ChangeEvent::MailboxChanged(m) => {
            wit::ChangeEvent::MailboxChanged(mailbox_ref_to_wit(m))
        }
        t::ChangeEvent::Disconnected => wit::ChangeEvent::Disconnected,
    }
}

// ── error mapping ─────────────────────────────────────────────────────────────

fn bridge_to_wit(e: BridgeError) -> wit::PluginError {
    match e {
        BridgeError::Protocol(m) => wit::PluginError::Protocol(m),
        BridgeError::Auth(m) => wit::PluginError::Auth(m),
        BridgeError::Transport(m) => wit::PluginError::Transport(m),
        BridgeError::Unsupported(m) => wit::PluginError::Unsupported(m),
        BridgeError::MailboxNotFound(m) => wit::PluginError::MailboxNotFound(m),
    }
}

fn wit_err_to_bridge(e: wit::PluginError) -> BridgeError {
    match e {
        wit::PluginError::Protocol(m) => BridgeError::Protocol(m),
        wit::PluginError::Auth(m) => BridgeError::Auth(m),
        wit::PluginError::Transport(m) => BridgeError::Transport(m),
        wit::PluginError::Unsupported(m) => BridgeError::Unsupported(m),
        wit::PluginError::MailboxNotFound(m) => BridgeError::MailboxNotFound(m),
        wit::PluginError::LimitExceeded(m) => BridgeError::Transport(format!("limit: {m}")),
        wit::PluginError::CapabilityDenied(m) => BridgeError::Auth(format!("denied: {m}")),
        wit::PluginError::Other(m) => BridgeError::Protocol(m),
    }
}

export!(Component);
