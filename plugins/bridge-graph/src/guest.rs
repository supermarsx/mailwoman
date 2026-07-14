//! The `wasm32-wasip2` component glue: wires the pure Graph mapping to the frozen
//! `mailwoman:plugin` WIT exports over the gated host imports. Compiled ONLY for the
//! guest target (`cfg(target_arch = "wasm32")`), so the host-target lib view stays
//! pure Rust. A guest opens no socket and holds no long-lived credential — it asks
//! the host for a transient token per call and hands every request to `http-fetch`.
//!
//! Targets the t10 second world `plugin-pim` (plan §5, t10-e0/e1): a superset of
//! `world plugin` that ADDITIONALLY exports the optional `calendar` / `tasks` /
//! `bridge-parity` interfaces. Because `plugin-pim` `include`s `world plugin`, the
//! existing host `bindgen!({world: "plugin"})` still loads this component unchanged;
//! the host's per-interface probe (t10-e1) additionally binds the three PIM exports.
//! The PIM/parity exports wire the existing fixture-tested pure functions
//! ([`crate::calendar`], [`crate::todo`], [`crate::caps`], [`crate::mail`]) — no
//! provider mapping is rewritten here; `supports-*()` reflect Graph's REAL support.

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

use crate::graph::{BridgeError, GraphClient, HttpRequestSpec, HttpResponseData, Transport};
use crate::types as t;
use crate::{calendar, caps, contacts, mail, todo};

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

// ── calendar export (t10 PIM seam) ────────────────────────────────────────────
//
// Graph genuinely supports calendars (incl. shared), event-delta sync, room
// resources, and free/busy ⇒ `supports-calendar() -> true`. Wires the existing
// fixture-tested `crate::calendar` pure functions; the structured `EventInfo` is
// serialized to the RFC 5545 VEVENT / VFREEBUSY text the WIT seam carries.

impl cal::Guest for Component {
    fn supports_calendar() -> bool {
        true
    }

    fn list_calendars() -> Result<Vec<cal::CalInfo>, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        calendar::list_calendars(&c, acct)
            .map(|v| v.into_iter().map(cal_info_to_wit).collect())
            .map_err(bridge_to_wit)
    }

    fn sync_events(
        calendar_id: String,
        cursor: wit::SyncCursor,
    ) -> Result<cal::EventDelta, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        // Graph's event delta is account-wide (`/me/events/delta`); the calendar-id is
        // echoed onto each event so the engine can attribute it.
        calendar::sync_events(&c, &cursor_from_wit(cursor))
            .map(|d| event_delta_to_wit(d, &calendar_id))
            .map_err(bridge_to_wit)
    }

    fn find_rooms() -> Result<Vec<cal::RoomInfo>, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        calendar::find_rooms(&c)
            .map(|v| v.into_iter().map(room_to_wit).collect())
            .map_err(bridge_to_wit)
    }

    fn get_schedule(who: String, start: String, end: String) -> Result<String, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        let sched = calendar::get_schedule(&c, std::slice::from_ref(&who), &start, &end)
            .map_err(bridge_to_wit)?;
        let view = sched.into_iter().next().map(|(_, v)| v).unwrap_or_default();
        Ok(freebusy_to_ical(&who, &start, &end, &view))
    }
}

// ── tasks export (t10 PIM seam) ───────────────────────────────────────────────
//
// Graph genuinely supports Microsoft To-Do ⇒ `supports-tasks() -> true`. Wires the
// existing `crate::todo` list/task readers; each task is serialized to VTODO text.

impl tasks_ex::Guest for Component {
    fn supports_tasks() -> bool {
        true
    }

    fn list_tasks() -> Result<Vec<tasks_ex::TaskInfo>, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        // The WIT `list-tasks` has no list arg, so aggregate over every To-Do list.
        // A list we cannot read is skipped (best-effort), never fatal.
        let lists = todo::list_lists(&c).map_err(bridge_to_wit)?;
        let mut out = Vec::new();
        for l in lists {
            if let Ok(tasks) = todo::list_tasks(&c, &l.id) {
                out.extend(tasks.into_iter().map(|t| task_to_wit(t, &l.id)));
            }
        }
        Ok(out)
    }

    fn sync_tasks(
        list_id: String,
        _cursor: wit::SyncCursor,
    ) -> Result<tasks_ex::TaskDelta, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        // Graph To-Do exposes no delta token in this mapping — return a full snapshot
        // of the list as `changed` with an empty cursor (honest best-effort: a full
        // re-list each sync, no phantom removals).
        let changed = todo::list_tasks(&c, &list_id)
            .map_err(bridge_to_wit)?
            .into_iter()
            .map(|t| task_to_wit(t, &list_id))
            .collect();
        Ok(tasks_ex::TaskDelta {
            changed,
            removed: Vec::new(),
            next_cursor: wit::SyncCursor { opaque: Vec::new() },
        })
    }

    fn complete(id: String) -> Result<(), wit::PluginError> {
        // Graph requires the OWNING list id to mutate a task, so the engine passes the
        // composite `list-id/task-id` (both ride `TaskInfo`). A bare id can't be
        // resolved to a list ⇒ honest `unsupported` rather than a false success.
        let (list_id, task_id) = match id.rsplit_once('/') {
            Some((l, t)) if !l.is_empty() && !t.is_empty() => (l, t),
            _ => {
                return Err(wit::PluginError::Unsupported(
                    "task completion needs the owning list id (list-id/task-id)".into(),
                ));
            }
        };
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        let body = serde_json::json!({ "status": "completed" });
        c.patch_ignore(&format!("/me/todo/lists/{list_id}/tasks/{task_id}"), &body)
            .map_err(bridge_to_wit)
    }
}

// ── bridge-parity export (t10 Outlook-parity seam) ────────────────────────────
//
// HONEST per-provider support (plan §2.6 / SPEC §10.3): Graph offers reactions,
// voting, message-recall (all best-effort) and Focused-Inbox (genuine) ⇒ all four
// `supports-*() -> true`. Wires the existing `crate::caps` / `crate::mail` impls; the
// recall honesty matrix is preserved verbatim (never a false claim of success).

impl parity::Guest for Component {
    fn supports_reactions() -> bool {
        true
    }
    fn supports_voting() -> bool {
        true
    }
    fn supports_recall() -> bool {
        true
    }
    fn supports_focused() -> bool {
        true
    }

    fn set_reaction(
        msg: wit::MessageRef,
        emoji: String,
        add: bool,
    ) -> Result<(), wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        if add {
            // Wire the existing best-effort reaction post.
            caps::react(&c, &msg.raw, &emoji)
                .map(|_| ())
                .map_err(bridge_to_wit)
        } else {
            // Removal: best-effort `unreact`, mirroring `caps::react`'s degrade-not-fail
            // posture (a non-2xx is a no-op success, never a surfaced failure).
            let body = serde_json::json!({ "reactionType": emoji });
            match c.post_ignore(&format!("/me/messages/{}/unreact", msg.raw), &body) {
                Ok(()) => Ok(()),
                Err(BridgeError::Auth(_)) => Err(wit::PluginError::Auth("unreact rejected".into())),
                Err(_) => Ok(()),
            }
        }
    }

    fn get_reactions(_msg: wit::MessageRef) -> Result<Vec<parity::Reaction>, wit::PluginError> {
        // Graph exposes no read-back for mailbox-message reactions — an honest empty
        // list (reactions are a best-effort write surface, never a fabricated read).
        Ok(Vec::new())
    }

    fn cast_vote(msg: wit::MessageRef, choice: String) -> Result<(), wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        caps::vote(&c, &msg.raw, &choice).map_err(bridge_to_wit)
    }

    fn tally(_msg: wit::MessageRef) -> Result<Vec<parity::VoteTally>, wit::PluginError> {
        // Outlook voting responses are not aggregated by Graph — an honest empty tally.
        Ok(Vec::new())
    }

    fn recall(msg: wit::MessageRef) -> Result<parity::RecallOutcome, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        let outcome = caps::recall(&c, &msg.raw).map_err(bridge_to_wit)?;
        Ok(recall_to_wit(outcome))
    }

    fn get_focused(msg: wit::MessageRef) -> Result<parity::FocusedState, wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        // Focused-Inbox is a genuine Graph field (`inferenceClassification`).
        let m: crate::model::GraphMessage = c
            .get_json(&format!(
                "/me/messages/{}?$select=inferenceClassification",
                msg.raw
            ))
            .map_err(bridge_to_wit)?;
        Ok(match m.inference_classification.as_deref() {
            Some("focused") => parity::FocusedState::Focused,
            _ => parity::FocusedState::Other,
        })
    }

    fn set_focused(msg: wit::MessageRef, focused: bool) -> Result<(), wit::PluginError> {
        let (tr, acct) = client();
        let c = GraphClient::new(&tr, acct);
        let mref = msgref_from_wit(&msg);
        let kw = if focused {
            t::KEYWORD_FOCUSED
        } else {
            t::KEYWORD_OTHER
        };
        // Reuse the mail flag-write mapping ($Focused/$Other → inferenceClassification).
        mail::store_flags(
            &c,
            std::slice::from_ref(&mref),
            &[t::Flag::Keyword(kw.to_string())],
            &[],
        )
        .map_err(bridge_to_wit)
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

// ── PIM / parity conversions (plain ↔ WIT) ────────────────────────────────────

/// Escape an iCalendar TEXT value (RFC 5545 §3.3.11): `\`, `;`, `,`, and newlines.
fn ical_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\r', "")
        .replace('\n', "\\n")
}

fn cal_info_to_wit(c: calendar::CalendarInfo) -> cal::CalInfo {
    cal::CalInfo {
        id: c.id,
        name: c.name,
        role: "calendar".to_string(),
        read_only: !c.can_edit,
    }
}

fn room_to_wit(r: calendar::RoomInfo) -> cal::RoomInfo {
    cal::RoomInfo {
        address: r.email,
        name: r.name,
        capacity: r.capacity,
    }
}

/// Serialize a structured [`calendar::EventInfo`] into the WIT event carrying its
/// RFC 5545 VEVENT text (the engine already speaks iCalendar) plus coarse coords.
fn event_to_wit(e: calendar::EventInfo, calendar_id: &str) -> cal::EventInfo {
    let mut ical = String::from("BEGIN:VEVENT\r\n");
    ical.push_str(&format!("UID:{}\r\n", e.id));
    ical.push_str(&format!("SUMMARY:{}\r\n", ical_escape(&e.subject)));
    if let Some(s) = &e.start {
        ical.push_str(&format!("DTSTART:{s}\r\n"));
    }
    if let Some(en) = &e.end {
        ical.push_str(&format!("DTEND:{en}\r\n"));
    }
    if let Some(loc) = &e.location {
        ical.push_str(&format!("LOCATION:{}\r\n", ical_escape(loc)));
    }
    if e.all_day {
        ical.push_str("X-MICROSOFT-CDO-ALLDAYEVENT:TRUE\r\n");
    }
    ical.push_str("END:VEVENT\r\n");
    cal::EventInfo {
        id: e.id,
        calendar_id: calendar_id.to_string(),
        ical,
        start: e.start,
        end: e.end,
    }
}

fn event_delta_to_wit(d: calendar::EventDelta, calendar_id: &str) -> cal::EventDelta {
    cal::EventDelta {
        changed: d
            .events
            .into_iter()
            .map(|e| event_to_wit(e, calendar_id))
            .collect(),
        removed: d.removed,
        next_cursor: cursor_to_wit(d.next_cursor),
    }
}

/// A minimal VFREEBUSY carrying Graph's per-interval `availabilityView` verbatim (no
/// lossy period expansion — the engine reads the coarse busy string).
fn freebusy_to_ical(who: &str, start: &str, end: &str, availability_view: &str) -> String {
    format!(
        "BEGIN:VFREEBUSY\r\nATTENDEE:{who}\r\nDTSTART:{start}\r\nDTEND:{end}\r\n\
         X-MICROSOFT-AVAILABILITYVIEW:{availability_view}\r\nEND:VFREEBUSY\r\n"
    )
}

/// Serialize a [`todo::TodoTaskInfo`] into the WIT task carrying its VTODO text.
fn task_to_wit(t: todo::TodoTaskInfo, list_id: &str) -> tasks_ex::TaskInfo {
    let mut ical = String::from("BEGIN:VTODO\r\n");
    ical.push_str(&format!("UID:{}\r\n", t.id));
    ical.push_str(&format!("SUMMARY:{}\r\n", ical_escape(&t.title)));
    if let Some(due) = &t.due {
        ical.push_str(&format!("DUE:{due}\r\n"));
    }
    ical.push_str(if t.completed {
        "STATUS:COMPLETED\r\n"
    } else {
        "STATUS:NEEDS-ACTION\r\n"
    });
    ical.push_str("END:VTODO\r\n");
    tasks_ex::TaskInfo {
        id: t.id,
        list_id: list_id.to_string(),
        ical,
        completed: t.completed,
    }
}

/// Map the honest [`caps::RecallOutcome`] onto the WIT variant, PRESERVING the recall
/// honesty matrix (§10.3): `guaranteed` is always false, so success is NEVER claimed —
/// an accepted request is `requested`; a declined attempt (e.g. already-read) carries
/// its plain-language limitation as `failed(note)`.
fn recall_to_wit(o: caps::RecallOutcome) -> parity::RecallOutcome {
    if o.requested {
        parity::RecallOutcome::Requested
    } else {
        parity::RecallOutcome::Failed(o.note)
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
