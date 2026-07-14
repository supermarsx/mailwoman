//! The `wasm32-wasip2` component: implements the `mailwoman:plugin` guest exports
//! over EWS, running all transport through the host `http-fetch` import (the guest
//! opens no socket). Compiled only for `wasm32` — the host build excludes it so
//! `cargo build --workspace` stays green (`crate::guest` is `#[cfg(target_arch =
//! "wasm32")]` in `lib.rs`).
//!
//! The `wit_bindgen::generate!` expansion + `export!` glue below require `unsafe`
//! for the WASI-p2 ABI, so this module re-enables it locally (the crate root is
//! `#![deny(unsafe_code)]`); no hand-written `unsafe` appears in this file.
#![allow(unsafe_code)]

use base64::Engine as _;

use crate::{ENDPOINT_URL, ews, ntlm, pim, state, wire};

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
use mailwoman::plugin::types::{
    BackendCaps, ChangeEvent, Flag, Mailbox, MailboxDelta, MailboxRef, MessageRef, PluginError,
    RawMessage, SyncCursor,
};

// Placeholder on-prem credentials — a real deployment binds these per-account at
// mount (sealed store / scoped KV). The recorded-fixture server does not validate
// them; they exist so the NTLMv2 handshake produces a well-formed Type 3 message.
const EWS_USER: &str = "svc-mailwoman";
const EWS_DOMAIN: &str = "CORP";
const EWS_PASSWORD: &str = "placeholder";
const EWS_WORKSTATION: &str = "MAILWOMAN";

struct Component;

fn protocol(msg: impl Into<String>) -> PluginError {
    PluginError::Protocol(msg.into())
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn post(auth: Option<&str>, body: &[u8]) -> Result<host::HttpResponse, PluginError> {
    let mut headers = vec![(
        "Content-Type".to_string(),
        "text/xml; charset=utf-8".to_string(),
    )];
    if let Some(a) = auth {
        headers.push(("Authorization".to_string(), a.to_string()));
    }
    host::http_fetch(&host::HttpRequest {
        method: "POST".into(),
        url: ENDPOINT_URL.into(),
        headers,
        body: Some(body.to_vec()),
    })
}

/// Run one EWS SOAP call over the host transport, performing the NTLMv2 401
/// challenge/response dance when the server demands it.
fn ews_call(soap_body: &str) -> Result<String, PluginError> {
    let body = soap_body.as_bytes();

    // Step 1: send the request with an NTLM NEGOTIATE (Type 1) token.
    let type1 = ntlm::type1_message();
    let neg = format!("NTLM {}", b64(&type1));
    let r1 = post(Some(&neg), body)?;

    if r1.status == 401 {
        // Step 2: parse the CHALLENGE (Type 2) and answer with AUTHENTICATE (Type 3).
        let www = header(&r1.headers, "WWW-Authenticate")
            .ok_or_else(|| PluginError::Auth("401 without WWW-Authenticate".into()))?;
        let token = www
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| PluginError::Auth("malformed NTLM challenge header".into()))?;
        let type2 = base64::engine::general_purpose::STANDARD
            .decode(token)
            .map_err(|e| PluginError::Auth(format!("NTLM challenge base64: {e}")))?;
        let challenge = ntlm::parse_challenge(&type2).map_err(PluginError::Auth)?;

        let rnd = host::random(8);
        let mut client_challenge = [0u8; 8];
        for (i, b) in rnd.iter().take(8).enumerate() {
            client_challenge[i] = *b;
        }
        let timestamp = ntlm::filetime_from_unix_millis(host::now());
        let ntowf2 = ntlm::ntowf_v2(EWS_USER, EWS_DOMAIN, EWS_PASSWORD);
        let resp = ntlm::ntlmv2_response(
            &ntowf2,
            &challenge.server_challenge,
            &client_challenge,
            timestamp,
            &challenge.target_info,
        );
        let type3 = ntlm::type3_message(EWS_USER, EWS_DOMAIN, EWS_WORKSTATION, &resp);
        let auth = format!("NTLM {}", b64(&type3));
        let r2 = post(Some(&auth), body)?;
        if (200..300).contains(&r2.status) {
            return Ok(String::from_utf8_lossy(&r2.body).into_owned());
        }
        return Err(PluginError::Auth(format!(
            "EWS NTLM authentication failed (HTTP {})",
            r2.status
        )));
    }

    if (200..300).contains(&r1.status) {
        return Ok(String::from_utf8_lossy(&r1.body).into_owned());
    }
    Err(PluginError::Transport(format!("EWS HTTP {}", r1.status)))
}

fn msgref(folder_id: &str, uid: u32, mbox: &MailboxRef) -> MessageRef {
    MessageRef {
        raw: wire::encode_msgref(&wire::MsgRef::imap(folder_id, 1, uid)),
        mailbox: mbox.clone(),
    }
}

fn item_id_of(r: &MessageRef) -> Result<(String, String), PluginError> {
    let uid = wire::decode_msgref(&r.raw)
        .and_then(|m| m.uid())
        .ok_or_else(|| protocol("message-ref carries no uid"))?;
    state::lookup_item(uid)
        .ok_or_else(|| PluginError::MailboxNotFound(format!("no EWS item for uid {uid}")))
}

impl ab::Guest for Component {
    fn capabilities() -> Result<BackendCaps, PluginError> {
        Ok(BackendCaps {
            idle: false, // no server push over EWS; the engine delta-polls sync
            move_cap: true,
            reactions: false,    // EWS has no reactions
            voting: true,        // Outlook voting buttons (crate::pim)
            recall: true,        // best-effort, honest (crate::pim::RecallOutcome)
            focused_sync: false, // Focused Inbox is a Graph/Outlook feature
        })
    }

    fn list_mailboxes() -> Result<Vec<Mailbox>, PluginError> {
        let xml = ews_call(&ews::sync_folder_hierarchy_request(""))?;
        let (folders, _state) = ews::parse_folder_hierarchy(&xml).map_err(protocol)?;
        Ok(folders
            .into_iter()
            .map(|f| Mailbox {
                mailbox_ref: MailboxRef {
                    name: f.id,
                    uidvalidity: 1,
                },
                role: f.role.to_string(),
                parent: None,
                total: f.total,
                unread: f.unread,
            })
            .collect())
    }

    fn sync_mailbox(mbox: MailboxRef, cursor: SyncCursor) -> Result<MailboxDelta, PluginError> {
        let folder_id = mbox.name.clone();
        let sync_state = wire::decode_cursor(&cursor.opaque);
        let xml = ews_call(&ews::sync_folder_items_request(
            &folder_id,
            &sync_state,
            512,
        ))?;
        let delta = ews::parse_folder_items(&xml).map_err(protocol)?;

        let added = delta
            .added
            .iter()
            .map(|it| {
                let uid = state::assign_uid(&it.id, &it.change_key);
                msgref(&folder_id, uid, &mbox)
            })
            .collect();
        let removed = delta
            .removed
            .iter()
            .map(|it| {
                // Reuse the uid if we minted one for this item earlier this session,
                // else mint one so the engine can reconcile the removal by ref.
                let uid = state::assign_uid(&it.id, &it.change_key);
                msgref(&folder_id, uid, &mbox)
            })
            .collect();

        Ok(MailboxDelta {
            added,
            removed,
            flag_changes: vec![],
            next_cursor: SyncCursor {
                opaque: wire::encode_cursor(&delta.sync_state),
            },
        })
    }

    fn fetch_raw(refs: Vec<MessageRef>) -> Result<Vec<RawMessage>, PluginError> {
        let mut out = Vec::with_capacity(refs.len());
        for r in refs {
            let (item_id, _ck) = item_id_of(&r)?;
            let xml = ews_call(&ews::get_item_mime_request(&item_id))?;
            let raw = ews::parse_item_mime(&xml).map_err(protocol)?;
            out.push(RawMessage {
                message_ref: r,
                raw,
                msg_flags: vec![],
                internaldate: None,
            });
        }
        Ok(out)
    }

    fn store_flags(
        refs: Vec<MessageRef>,
        add: Vec<Flag>,
        remove: Vec<Flag>,
    ) -> Result<(), PluginError> {
        let seen_add = add.iter().any(|f| matches!(f, Flag::Seen));
        let seen_remove = remove.iter().any(|f| matches!(f, Flag::Seen));
        if !seen_add && !seen_remove {
            return Ok(()); // only the Seen/read flag maps to an EWS property
        }
        let is_read = seen_add && !seen_remove;
        for r in refs {
            let (item_id, change_key) = item_id_of(&r)?;
            let xml = ews_call(&ews::update_read_flag_request(
                &item_id,
                &change_key,
                is_read,
            ))?;
            if !crate::soap::is_success(&xml) {
                return Err(protocol(
                    crate::soap::message_text(&xml).unwrap_or_else(|| "UpdateItem failed".into()),
                ));
            }
        }
        Ok(())
    }

    fn move_messages(refs: Vec<MessageRef>, to: MailboxRef) -> Result<(), PluginError> {
        for r in refs {
            let (item_id, _ck) = item_id_of(&r)?;
            let xml = ews_call(&ews::move_item_request(&item_id, &to.name))?;
            if !crate::soap::is_success(&xml) {
                return Err(protocol(
                    crate::soap::message_text(&xml).unwrap_or_else(|| "MoveItem failed".into()),
                ));
            }
        }
        Ok(())
    }

    fn submit(
        mbox: MailboxRef,
        raw: Vec<u8>,
        _msg_flags: Vec<Flag>,
    ) -> Result<MessageRef, PluginError> {
        let xml = ews_call(&ews::create_item_send_request(&raw))?;
        let item = ews::parse_created_item_id(&xml)
            .map_err(protocol)?
            .ok_or_else(|| protocol("CreateItem returned no ItemId"))?;
        let uid = state::assign_uid(&item.id, &item.change_key);
        Ok(msgref(&mbox.name, uid, &mbox))
    }

    fn poll_changes() -> Result<Vec<ChangeEvent>, PluginError> {
        // EWS bridges resync via SyncFolderItems on the engine's cadence; there is
        // no server-push socket, so nothing to drain here.
        Ok(vec![])
    }
}

impl addr::Guest for Component {
    fn search(query: String) -> Result<Vec<String>, PluginError> {
        let xml = ews_call(&ews::resolve_names_request(&query))?;
        Ok(ews::parse_resolve_names(&xml)
            .into_iter()
            .map(|e| e.formatted())
            .collect())
    }
}

// The remaining exports are not this bridge's role; stub them (the host only calls
// capability-granted exports, and the EWS manifest grants none of these).
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

impl dlp::Guest for Component {
    fn detect(_body: Vec<u8>) -> Result<Vec<String>, PluginError> {
        Ok(vec![])
    }
}

impl spam::Guest for Component {
    fn classify(_raw: Vec<u8>) -> Result<String, PluginError> {
        Ok("ham".into())
    }
}

// Reference `pim` so the wasm build keeps the PIM surface (calendar/free-busy/
// rooms/OOF/recall/voting) linked and available to the host mount layer even though
// those operations are not part of the account-backend WIT export.
#[allow(dead_code)]
fn _pim_surface_is_linked() {
    let _ = pim::get_room_lists_request();
}

export!(Component);
