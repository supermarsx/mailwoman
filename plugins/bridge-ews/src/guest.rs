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

use crate::{ews, ntlm, pim, wire};

// t10-e3: target the `plugin-pim` second world (plan §5 / t10-e0) so this bridge ALSO
// exports the optional `calendar`/`tasks`/`bridge-parity` interfaces the host probes.
// Because `plugin-pim` `include`s `world plugin`, the account-backend/mail exports are
// byte-unchanged and the existing host `bindgen!({world: "plugin"})` still loads this
// component; the PIM exports are additive.
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

// The account this component instance is bound to. The host maps it to the concrete
// account and unseals THAT account's EWS endpoint + credentials via the gated
// `basic-credentials` import; the guest passes it opaquely and holds no long-lived
// secret (mirrors the Graph/Gmail `oauth-token` account handle). One instance backs
// one account, so the empty handle resolves to the bound account host-side.
const ACCOUNT: &str = "";

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

fn post(url: &str, auth: Option<&str>, body: &[u8]) -> Result<host::HttpResponse, PluginError> {
    let mut headers = vec![(
        "Content-Type".to_string(),
        "text/xml; charset=utf-8".to_string(),
    )];
    if let Some(a) = auth {
        headers.push(("Authorization".to_string(), a.to_string()));
    }
    host::http_fetch(&host::HttpRequest {
        method: "POST".into(),
        url: url.into(),
        headers,
        body: Some(body.to_vec()),
    })
}

/// Run one EWS SOAP call over the host transport. The bound account's endpoint +
/// credentials are pulled per call through the gated `basic-credentials` host import
/// (the secret is host-held; the guest keeps it only for the duration of this call).
/// The auth scheme is selected PER ACCOUNT: an account configured without an NT
/// domain uses HTTP Basic; one with a domain uses pure-Rust NTLMv2 (which needs the
/// cleartext password + domain + workstation to derive NTOWFv2 — an OAuth bearer
/// cannot serve it, which is why this seam exists alongside `oauth-token`).
fn ews_call(soap_body: &str) -> Result<String, PluginError> {
    let creds = host::basic_credentials(ACCOUNT)?;
    let body = soap_body.as_bytes();
    if creds.domain.is_empty() {
        basic_call(&creds, body)
    } else {
        ntlm_call(&creds, body)
    }
}

/// HTTP Basic path: attach `Authorization: Basic base64(user:password)` up front.
fn basic_call(creds: &host::HostCredentials, body: &[u8]) -> Result<String, PluginError> {
    let auth = format!(
        "Basic {}",
        b64(format!("{}:{}", creds.user, creds.password).as_bytes())
    );
    let r = post(&creds.endpoint, Some(&auth), body)?;
    if (200..300).contains(&r.status) {
        return Ok(String::from_utf8_lossy(&r.body).into_owned());
    }
    if r.status == 401 {
        return Err(PluginError::Auth(
            "EWS Basic authentication failed (HTTP 401)".into(),
        ));
    }
    Err(PluginError::Transport(format!("EWS HTTP {}", r.status)))
}

/// NTLMv2 path: the 401 NEGOTIATE (Type 1) → CHALLENGE (Type 2) → AUTHENTICATE
/// (Type 3) dance, keyed by the account's cleartext credential.
fn ntlm_call(creds: &host::HostCredentials, body: &[u8]) -> Result<String, PluginError> {
    // Step 1: send the request with an NTLM NEGOTIATE (Type 1) token.
    let type1 = ntlm::type1_message();
    let neg = format!("NTLM {}", b64(&type1));
    let r1 = post(&creds.endpoint, Some(&neg), body)?;

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
        let ntowf2 = ntlm::ntowf_v2(&creds.user, &creds.domain, &creds.password);
        let resp = ntlm::ntlmv2_response(
            &ntowf2,
            &challenge.server_challenge,
            &client_challenge,
            timestamp,
            &challenge.target_info,
        );
        let type3 = ntlm::type3_message(&creds.user, &creds.domain, &creds.workstation, &resp);
        let auth = format!("NTLM {}", b64(&type3));
        let r2 = post(&creds.endpoint, Some(&auth), body)?;
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

fn msgref(item_id: &str, change_key: &str, mbox: &MailboxRef) -> MessageRef {
    MessageRef {
        // Pack the native EWS ItemId + ChangeKey into the opaque `raw`; the host adapter
        // round-trips it verbatim as MessageRef::Plugin (no synthetic-UID map needed).
        raw: wire::encode_msgref(item_id, change_key),
        mailbox: mbox.clone(),
    }
}

fn item_id_of(r: &MessageRef) -> Result<(String, String), PluginError> {
    wire::decode_msgref(&r.raw).ok_or_else(|| protocol("message-ref carries no EWS item id"))
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
            .map(|it| msgref(&it.id, &it.change_key, &mbox))
            .collect();
        let removed = delta
            .removed
            .iter()
            .map(|it| msgref(&it.id, &it.change_key, &mbox))
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
        Ok(msgref(&item.id, &item.change_key, &mbox))
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

// ── PIM / Outlook-parity exports (t10-e3) ─────────────────────────────────────────
//
// Honest EWS support matrix (plan §1.2 / §2.1, this executor's charter). The
// `supports-*()` funcs are the authoritative per-interface probe the host binds on;
// they NEVER claim a capability EWS lacks:
//   * calendar  — SUPPORTED (FindItem CalendarView + free/busy + rooms).
//   * tasks     — SUPPORTED (FindItem over the Tasks distinguished folder).
//   * reactions — NOT an EWS feature ⇒ false.
//   * voting    — NOT exposed through the parity seam ⇒ false.
//   * recall    — no third-party recall/unsend API (§10.3 honesty) ⇒ false /
//                 `recall-outcome::unsupported`.
//   * focused   — Focused Inbox is a Graph/Outlook feature ⇒ false.
// Where a capability is false, the host binds the interface but the honest
// `supports-*()` keeps the engine on its byte-unchanged standards fallback.

fn task_to_wit(t: pim::EwsTask) -> tasks_ex::TaskInfo {
    // The opaque per-backend task id carries ItemId + ChangeKey (same encoding as the
    // account-backend `message-ref.raw`) so `complete(id)` can issue an EWS UpdateItem.
    let id = wire::encode_msgref(&t.id, &t.change_key);
    let ical = pim::task_to_vtodo(&t);
    tasks_ex::TaskInfo {
        id,
        list_id: "tasks".into(),
        ical,
        completed: t.complete,
    }
}

impl cal::Guest for Component {
    fn supports_calendar() -> bool {
        true
    }

    fn list_calendars() -> Result<Vec<cal::CalInfo>, PluginError> {
        // EWS exposes the bound mailbox's primary Calendar distinguished folder.
        Ok(vec![cal::CalInfo {
            id: "calendar".into(),
            name: "Calendar".into(),
            role: "calendar".into(),
            read_only: false,
        }])
    }

    fn sync_events(
        calendar_id: String,
        _cursor: SyncCursor,
    ) -> Result<cal::EventDelta, PluginError> {
        // EWS FindItem+CalendarView is a window query (not a true incremental delta):
        // re-read the default window and let the engine reconcile by event UID.
        let xml = ews_call(&pim::find_calendar_events_request(
            pim::CAL_WINDOW_START,
            pim::CAL_WINDOW_END,
        ))?;
        let changed = pim::parse_calendar_events(&xml)
            .into_iter()
            .map(|e| cal::EventInfo {
                id: e.id.clone(),
                calendar_id: calendar_id.clone(),
                ical: pim::cal_event_to_vevent(&e),
                start: (!e.start.is_empty()).then(|| e.start.clone()),
                end: (!e.end.is_empty()).then(|| e.end.clone()),
            })
            .collect();
        Ok(cal::EventDelta {
            changed,
            removed: vec![],
            next_cursor: SyncCursor {
                opaque: pim::CAL_WINDOW_END.as_bytes().to_vec(),
            },
        })
    }

    fn find_rooms() -> Result<Vec<cal::RoomInfo>, PluginError> {
        let xml = ews_call(&pim::get_room_lists_request())?;
        Ok(pim::parse_room_addresses(&xml)
            .into_iter()
            .map(|g| cal::RoomInfo {
                address: g.email,
                name: g.display_name,
                capacity: None,
            })
            .collect())
    }

    fn get_schedule(who: String, start: String, end: String) -> Result<String, PluginError> {
        let xml = ews_call(&pim::get_user_availability_request(&[&who], &start, &end))?;
        let blocks = pim::parse_free_busy(&xml);
        Ok(pim::free_busy_to_vfreebusy(&who, &blocks))
    }
}

impl tasks_ex::Guest for Component {
    fn supports_tasks() -> bool {
        true
    }

    fn list_tasks() -> Result<Vec<tasks_ex::TaskInfo>, PluginError> {
        let xml = ews_call(&pim::find_tasks_request())?;
        Ok(pim::parse_tasks(&xml)
            .into_iter()
            .map(task_to_wit)
            .collect())
    }

    fn sync_tasks(
        _list_id: String,
        _cursor: SyncCursor,
    ) -> Result<tasks_ex::TaskDelta, PluginError> {
        let xml = ews_call(&pim::find_tasks_request())?;
        let changed = pim::parse_tasks(&xml)
            .into_iter()
            .map(task_to_wit)
            .collect();
        Ok(tasks_ex::TaskDelta {
            changed,
            removed: vec![],
            next_cursor: SyncCursor {
                opaque: b"ews-tasks".to_vec(),
            },
        })
    }

    fn complete(id: String) -> Result<(), PluginError> {
        let (item_id, change_key) =
            wire::decode_msgref(&id).ok_or_else(|| protocol("task id carries no EWS item id"))?;
        let xml = ews_call(&pim::complete_task_request(&item_id, &change_key))?;
        if crate::soap::is_success(&xml) {
            Ok(())
        } else {
            Err(protocol(crate::soap::message_text(&xml).unwrap_or_else(
                || "UpdateItem (task complete) failed".into(),
            )))
        }
    }
}

impl parity::Guest for Component {
    fn supports_reactions() -> bool {
        false // EWS has no reactions
    }
    fn supports_voting() -> bool {
        false // Outlook voting buttons are not exposed through the parity seam
    }
    fn supports_recall() -> bool {
        false // no third-party recall/unsend API (§10.3 honesty)
    }
    fn supports_focused() -> bool {
        false // Focused Inbox is a Graph/Outlook feature
    }

    fn set_reaction(_msg: MessageRef, _emoji: String, _add: bool) -> Result<(), PluginError> {
        Err(PluginError::Unsupported("EWS has no reactions".into()))
    }
    fn get_reactions(_msg: MessageRef) -> Result<Vec<parity::Reaction>, PluginError> {
        Ok(vec![])
    }

    fn cast_vote(_msg: MessageRef, _choice: String) -> Result<(), PluginError> {
        Err(PluginError::Unsupported(
            "EWS voting is not exposed through the parity seam".into(),
        ))
    }
    fn tally(_msg: MessageRef) -> Result<Vec<parity::VoteTally>, PluginError> {
        Ok(vec![])
    }

    fn recall(_msg: MessageRef) -> Result<parity::RecallOutcome, PluginError> {
        // Honest: EWS has no cross-organization unsend; never claim a recall it cannot
        // perform (mirrors mw_engine::v7::RecallOutcome::Unsupported).
        Ok(parity::RecallOutcome::Unsupported)
    }

    fn get_focused(_msg: MessageRef) -> Result<parity::FocusedState, PluginError> {
        Ok(parity::FocusedState::Other)
    }
    fn set_focused(_msg: MessageRef, _focused: bool) -> Result<(), PluginError> {
        Err(PluginError::Unsupported(
            "Focused Inbox is a Graph/Outlook feature".into(),
        ))
    }
}

export!(Component);
