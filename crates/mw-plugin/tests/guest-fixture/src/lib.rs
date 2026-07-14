//! A minimal, real `wasm32-wasip2` guest **component** used by the `mw-plugin`
//! host integration tests (t7-e1). It is deliberately small but exercises every
//! security-relevant path the host must enforce:
//!
//! * `account-backend` — canned `capabilities` + `list-mailboxes` (+ a `fetch-raw`
//!   echo) so the host↔trait adapter round-trip is proven THROUGH the WIT boundary.
//! * `addrbook-source::search` — calls the host `http-fetch` import with the query
//!   as the URL, so the host's `net_allowlist` gate is exercised from real guest code
//!   (in-allowlist ⇒ status string; out-of-allowlist ⇒ the host traps it → `denied`).
//! * `dlp-detect::detect` — `"loop"` busy-loops forever (epoch deadline / fuel trip)
//!   and `"alloc"` grows memory without bound (`StoreLimits` memory-ceiling trip).
//!
//! Every other export in the `plugin` world is a trivial stub — a plugin only has to
//! provide the hooks it uses; the host only CALLS the capability-granted ones.

#![allow(clippy::empty_loop)]

wit_bindgen::generate!({
    world: "plugin",
    path: "../../wit",
});

use exports::mailwoman::plugin::account_backend as ab;
use exports::mailwoman::plugin::addrbook_source as addr;
use exports::mailwoman::plugin::autoconfig_source as autoc;
use exports::mailwoman::plugin::dlp_detect as dlp;
use exports::mailwoman::plugin::message_pipeline as pipe;
use exports::mailwoman::plugin::spam_action as spam;

use mailwoman::plugin::host;
use mailwoman::plugin::types::{
    BackendCaps, ChangeEvent, Flag, Mailbox, MailboxDelta, MailboxRef, MessageRef, PluginError,
    RawMessage, SyncCursor,
};

struct Component;

impl ab::Guest for Component {
    fn capabilities() -> Result<BackendCaps, PluginError> {
        Ok(BackendCaps {
            idle: true,
            move_cap: true,
            reactions: false,
            voting: false,
            recall: false,
            focused_sync: false,
        })
    }

    fn list_mailboxes() -> Result<Vec<Mailbox>, PluginError> {
        // Two mailboxes with real strings/roles ⇒ proves record+list+string
        // marshalling across the boundary, not just primitives.
        Ok(vec![
            Mailbox {
                mailbox_ref: MailboxRef {
                    name: "INBOX".into(),
                    uidvalidity: 1,
                },
                role: "inbox".into(),
                parent: None,
                total: 3,
                unread: 1,
            },
            Mailbox {
                mailbox_ref: MailboxRef {
                    name: "Archive".into(),
                    uidvalidity: 1,
                },
                role: "archive".into(),
                parent: None,
                total: 42,
                unread: 0,
            },
        ])
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
        // Echo each ref back with a canned body ⇒ proves the round-trip of nested
        // records + byte lists.
        Ok(refs
            .into_iter()
            .map(|r| RawMessage {
                message_ref: r,
                raw: b"Subject: hello\r\n\r\nfrom the jail\r\n".to_vec(),
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

    fn submit(_mbox: MailboxRef, _raw: Vec<u8>, _flags: Vec<Flag>) -> Result<MessageRef, PluginError> {
        Err(PluginError::Unsupported("submit".into()))
    }

    fn poll_changes() -> Result<Vec<ChangeEvent>, PluginError> {
        Ok(vec![])
    }
}

impl addr::Guest for Component {
    fn search(query: String) -> Result<Vec<String>, PluginError> {
        // Treat the query as a URL and ask the HOST to fetch it. The host enforces
        // the plugin's `net_allowlist`: a URL outside it comes back as
        // `capability-denied`, which we propagate. A URL inside it returns a status.
        let req = host::HttpRequest {
            method: "GET".into(),
            url: query,
            headers: vec![],
            body: None,
        };
        let resp = host::http_fetch(&req)?;
        Ok(vec![format!("status={}", resp.status)])
    }
}

impl dlp::Guest for Component {
    fn detect(body: Vec<u8>) -> Result<Vec<String>, PluginError> {
        match body.as_slice() {
            b"loop" => {
                // Never returns — the host's epoch deadline (or fuel) must preempt.
                loop {}
            }
            b"alloc" => {
                // Grow linear memory without bound — the host's StoreLimits memory
                // ceiling must deny the growth, tripping the guest allocator → trap.
                let mut sink: Vec<Vec<u8>> = Vec::new();
                loop {
                    sink.push(vec![0u8; 8 * 1024 * 1024]);
                }
            }
            _ => Ok(vec![]),
        }
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

impl autoc::Guest for Component {
    fn lookup(_email: String) -> Result<Option<String>, PluginError> {
        Ok(None)
    }
}

impl spam::Guest for Component {
    fn classify(_raw: Vec<u8>) -> Result<String, PluginError> {
        Ok("ham".into())
    }
}

export!(Component);
