//! NOTIFY (RFC 5465) and the extended `STATUS` reply.
//!
//! With NOTIFY one connection learns about changes in every subscribed
//! mailbox: the server sends `* STATUS <mailbox> (...)` for mailboxes other
//! than the selected one. Without NOTIFY the client keeps IDLE on INBOX and
//! sweeps the other mailboxes with `STATUS` now and then; [`StatusTracker`]
//! tells which mailboxes changed between two sweeps, so both paths emit the
//! same per-mailbox change events.
//!
//! `imap-proto` 0.16.7 rejects a `STATUS` reply carrying items it does not
//! know (SIZE, DELETED, APPENDLIMIT, MAILBOXID), so [`parse_status`] parses
//! the reply by hand. Mailbox names are decoded from modified UTF-7 unless
//! UTF-8 names are enabled.

use std::collections::HashMap;

use crate::lex::{Cursor, LexResult, Parsed, bad};
use crate::utf7;

/// A NOTIFY event (RFC 5465 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    MessageNew,
    MessageExpunge,
    FlagChange,
    AnnotationChange,
    MailboxName,
    SubscriptionChange,
    MailboxMetadataChange,
    ServerMetadataChange,
}

impl Event {
    pub fn as_str(self) -> &'static str {
        match self {
            Event::MessageNew => "MessageNew",
            Event::MessageExpunge => "MessageExpunge",
            Event::FlagChange => "FlagChange",
            Event::AnnotationChange => "AnnotationChange",
            Event::MailboxName => "MailboxName",
            Event::SubscriptionChange => "SubscriptionChange",
            Event::MailboxMetadataChange => "MailboxMetadataChange",
            Event::ServerMetadataChange => "ServerMetadataChange",
        }
    }

    fn is_message_event(self) -> bool {
        matches!(
            self,
            Event::MessageNew | Event::MessageExpunge | Event::FlagChange | Event::AnnotationChange
        )
    }
}

/// Which mailboxes an event group covers (RFC 5465 §6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    Selected,
    SelectedDelayed,
    Inboxes,
    Personal,
    Subscribed,
    Subtree(Vec<String>),
    Mailboxes(Vec<String>),
}

impl Filter {
    fn kind(&self) -> &'static str {
        match self {
            Filter::Selected => "selected",
            Filter::SelectedDelayed => "selected-delayed",
            Filter::Inboxes => "inboxes",
            Filter::Personal => "personal",
            Filter::Subscribed => "subscribed",
            Filter::Subtree(_) => "subtree",
            Filter::Mailboxes(_) => "mailboxes",
        }
    }

    fn is_selected(&self) -> bool {
        matches!(self, Filter::Selected | Filter::SelectedDelayed)
    }
}

/// One `(filter events)` group. Empty `events` sends `NONE`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventGroup {
    pub filter: Filter,
    pub events: Vec<Event>,
}

/// A `NOTIFY SET` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifySet {
    /// Send `STATUS` so the server reports the current state of every
    /// watched mailbox at once.
    pub status: bool,
    pub groups: Vec<EventGroup>,
}

/// Why a [`NotifySet`] cannot be sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotifyError {
    NoGroups,
    /// MessageNew and MessageExpunge must be requested together, and
    /// FlagChange / AnnotationChange need both (RFC 5465 §5).
    IncompleteMessageEvents(&'static str),
    /// Mailbox events (MailboxName, ...) apply to mailboxes, not to the
    /// selected mailbox filter.
    MailboxEventOnSelected,
    /// A filter kind appears twice, or both selected filters appear.
    DuplicateFilter(&'static str),
    EmptyMailboxList(&'static str),
    BadMailboxName(String),
}

impl std::fmt::Display for NotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotifyError::NoGroups => f.write_str("NOTIFY SET needs at least one event group"),
            NotifyError::IncompleteMessageEvents(k) => write!(
                f,
                "{k}: MessageNew and MessageExpunge must both be requested"
            ),
            NotifyError::MailboxEventOnSelected => {
                f.write_str("mailbox events cannot be requested for the selected mailbox")
            }
            NotifyError::DuplicateFilter(k) => write!(f, "filter {k} appears more than once"),
            NotifyError::EmptyMailboxList(k) => write!(f, "filter {k} names no mailbox"),
            NotifyError::BadMailboxName(n) => write!(f, "mailbox name {n:?} cannot be sent"),
        }
    }
}

impl std::error::Error for NotifyError {}

/// `NOTIFY NONE`: stop all notifications.
pub const NOTIFY_NONE: &str = "NOTIFY NONE";

/// The message events every watch asks for.
pub const MESSAGE_EVENTS: &[Event] = &[Event::MessageNew, Event::MessageExpunge, Event::FlagChange];

impl NotifySet {
    /// The watch Mailwoman uses: message changes in the selected mailbox and in
    /// every subscribed mailbox, plus mailbox renames and subscription changes
    /// in personal mailboxes so the folder list stays current.
    pub fn watch_subscribed() -> Self {
        NotifySet {
            status: false,
            groups: vec![
                EventGroup {
                    filter: Filter::Selected,
                    events: MESSAGE_EVENTS.to_vec(),
                },
                EventGroup {
                    filter: Filter::Subscribed,
                    events: MESSAGE_EVENTS.to_vec(),
                },
                EventGroup {
                    filter: Filter::Personal,
                    events: vec![Event::MailboxName, Event::SubscriptionChange],
                },
            ],
        }
    }

    /// The command text (without the tag). Mailbox names are encoded for the
    /// session: UTF-8 when `utf8` is enabled, else modified UTF-7.
    pub fn command(&self, utf8: bool) -> Result<String, NotifyError> {
        if self.groups.is_empty() {
            return Err(NotifyError::NoGroups);
        }
        let mut seen: Vec<&'static str> = Vec::new();
        let mut out = String::from("NOTIFY SET");
        if self.status {
            out.push_str(" STATUS");
        }
        for group in &self.groups {
            let kind = group.filter.kind();
            let selected_clash = group.filter.is_selected()
                && seen
                    .iter()
                    .any(|k| *k == "selected" || *k == "selected-delayed");
            if seen.contains(&kind) || selected_clash {
                return Err(NotifyError::DuplicateFilter(kind));
            }
            seen.push(kind);
            validate_events(group)?;

            out.push_str(" (");
            out.push_str(kind);
            if let Filter::Subtree(names) | Filter::Mailboxes(names) = &group.filter {
                if names.is_empty() {
                    return Err(NotifyError::EmptyMailboxList(kind));
                }
                out.push_str(" (");
                for (i, name) in names.iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    let arg = utf7::mailbox_arg(name, utf8)
                        .map_err(|_| NotifyError::BadMailboxName(name.clone()))?;
                    out.push_str(&arg);
                }
                out.push(')');
            }
            if group.events.is_empty() {
                out.push_str(" NONE");
            } else {
                out.push_str(" (");
                for (i, ev) in group.events.iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    out.push_str(ev.as_str());
                }
                out.push(')');
            }
            out.push(')');
        }
        Ok(out)
    }
}

fn validate_events(group: &EventGroup) -> Result<(), NotifyError> {
    let kind = group.filter.kind();
    let has = |e| group.events.contains(&e);
    let new = has(Event::MessageNew);
    let expunge = has(Event::MessageExpunge);
    if new != expunge {
        return Err(NotifyError::IncompleteMessageEvents(kind));
    }
    if (has(Event::FlagChange) || has(Event::AnnotationChange)) && !new {
        return Err(NotifyError::IncompleteMessageEvents(kind));
    }
    if group.filter.is_selected() && group.events.iter().any(|e| !e.is_message_event()) {
        return Err(NotifyError::MailboxEventOnSelected);
    }
    Ok(())
}

/// The items of a `STATUS` reply. Absent items stay `None`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MailboxStatus {
    pub messages: Option<u32>,
    pub recent: Option<u32>,
    pub uidnext: Option<u32>,
    pub uidvalidity: Option<u32>,
    pub unseen: Option<u32>,
    pub deleted: Option<u32>,
    pub highestmodseq: Option<u64>,
    /// STATUS=SIZE (RFC 8438): total size in octets.
    pub size: Option<u64>,
    /// RFC 9208 DELETED-STORAGE.
    pub deleted_storage: Option<u64>,
    /// APPENDLIMIT (RFC 7889): `Some(None)` for `NIL`, no limit.
    pub appendlimit: Option<Option<u64>>,
    /// OBJECTID (RFC 8474).
    pub mailboxid: Option<String>,
}

impl MailboxStatus {
    /// True when `other` shows a change in the messages of the mailbox: a
    /// different count, next UID, validity, unseen count or mod-sequence.
    /// Items missing on either side are not compared.
    pub fn differs_from(&self, other: &MailboxStatus) -> bool {
        fn diff<T: PartialEq>(a: &Option<T>, b: &Option<T>) -> bool {
            matches!((a, b), (Some(a), Some(b)) if a != b)
        }
        diff(&self.messages, &other.messages)
            || diff(&self.uidnext, &other.uidnext)
            || diff(&self.uidvalidity, &other.uidvalidity)
            || diff(&self.unseen, &other.unseen)
            || diff(&self.highestmodseq, &other.highestmodseq)
            || diff(&self.deleted, &other.deleted)
    }
}

/// One parsed `* STATUS` reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusReply {
    /// The decoded mailbox name.
    pub mailbox: String,
    /// The name exactly as the server sent it, for later commands.
    pub wire_name: String,
    pub status: MailboxStatus,
}

/// Parse one `* STATUS <mailbox> (<items>)` CRLF reply from the start of
/// `buf`. `utf8` says whether UTF-8 mailbox names are enabled.
pub fn parse_status(buf: &[u8], utf8: bool) -> Parsed<StatusReply> {
    parse_status_inner(buf, utf8).into()
}

fn parse_status_inner(buf: &[u8], utf8: bool) -> LexResult<(usize, StatusReply)> {
    let mut c = Cursor::new(buf);
    c.keyword("* STATUS ")?;
    let raw = c.astring()?;
    let wire_name = String::from_utf8(raw)
        .map_err(|_| crate::lex::LexError::Bad("mailbox name is not UTF-8".into()))?;
    let mailbox = if wire_name.eq_ignore_ascii_case("INBOX") {
        "INBOX".to_string()
    } else {
        utf7::decode_lossless(&wire_name, utf8)
    };
    c.sp()?;
    c.byte(b'(')?;
    let mut status = MailboxStatus::default();
    let mut first = true;
    while !c.eat(b')')? {
        if !first {
            c.sp()?;
        }
        first = false;
        let item = c.atom()?.to_ascii_uppercase();
        c.sp()?;
        match item.as_str() {
            "MESSAGES" => status.messages = Some(c.number()?),
            "RECENT" => status.recent = Some(c.number()?),
            "UIDNEXT" => status.uidnext = Some(c.nz_number()?),
            "UIDVALIDITY" => status.uidvalidity = Some(c.nz_number()?),
            "UNSEEN" => status.unseen = Some(c.number()?),
            "DELETED" => status.deleted = Some(c.number()?),
            "HIGHESTMODSEQ" => status.highestmodseq = Some(c.number64()?),
            "SIZE" => status.size = Some(c.number64()?),
            "DELETED-STORAGE" => status.deleted_storage = Some(c.number64()?),
            "APPENDLIMIT" => {
                status.appendlimit = Some(if c.keyword("NIL").is_ok() {
                    None
                } else {
                    Some(c.number64()?)
                })
            }
            "MAILBOXID" => {
                c.byte(b'(')?;
                let id = c.atom()?;
                if id.len() > 255 {
                    return bad("MAILBOXID longer than 255 characters");
                }
                status.mailboxid = Some(id.to_string());
                c.byte(b')')?;
            }
            _ => c.skip_ext_value()?,
        }
    }
    // Some servers put a space before the CRLF.
    while c.eat(b' ')? {}
    c.crlf()?;
    Ok((
        c.pos(),
        StatusReply {
            mailbox,
            wire_name,
            status,
        },
    ))
}

/// A notification-related signal found in a response's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signal {
    /// `[NOTIFICATIONOVERFLOW]`: the server stopped notifying; the client
    /// must resynchronise every watched mailbox and send NOTIFY again.
    Overflow,
    /// `[BADEVENT (...)]`: the server refused these events; retry without them.
    BadEvent(Vec<String>),
}

/// Look for a NOTIFY response code at the start of a response text (what
/// `imap-proto` leaves in `information` for codes it does not know).
pub fn signal(text: &str) -> Option<Signal> {
    let rest = text.trim_start().strip_prefix('[')?;
    let end = rest.find(']')?;
    let code = &rest[..end];
    let (name, args) = code.split_once(' ').unwrap_or((code, ""));
    if name.eq_ignore_ascii_case("NOTIFICATIONOVERFLOW") {
        return Some(Signal::Overflow);
    }
    if name.eq_ignore_ascii_case("BADEVENT") {
        let list = args
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .split_whitespace()
            .map(str::to_string)
            .collect();
        return Some(Signal::BadEvent(list));
    }
    None
}

/// Last known `STATUS` per mailbox, for the sweep that replaces NOTIFY when
/// the server lacks it.
#[derive(Debug, Default)]
pub struct StatusTracker {
    last: HashMap<String, MailboxStatus>,
}

impl StatusTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a reply; true when the mailbox changed since the last one. The
    /// first sighting of a mailbox only records it.
    pub fn observe(&mut self, mailbox: &str, status: MailboxStatus) -> bool {
        match self.last.insert(mailbox.to_string(), status) {
            Some(prev) => prev.differs_from(&self.last[mailbox]),
            None => false,
        }
    }

    /// Forget mailboxes that were not part of the latest sweep (deleted or
    /// unsubscribed), so a later reappearance counts as a first sighting.
    pub fn retain(&mut self, present: &[&str]) {
        self.last.retain(|name, _| present.contains(&name.as_str()));
    }

    pub fn len(&self) -> usize {
        self.last.len()
    }

    pub fn is_empty(&self) -> bool {
        self.last.is_empty()
    }
}

/// Fuzz entry point: parse arbitrary bytes as a STATUS reply (both name
/// modes) and as a response text signal. Must never panic.
pub fn fuzz(data: &[u8]) {
    for utf8 in [false, true] {
        if let Parsed::Done { consumed, .. } = parse_status(data, utf8) {
            assert!(consumed <= data.len());
        }
    }
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = signal(text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn done(buf: &[u8], utf8: bool) -> (usize, StatusReply) {
        match parse_status(buf, utf8) {
            Parsed::Done { consumed, value } => (consumed, value),
            other => panic!(
                "expected Done for {:?}, got {other:?}",
                String::from_utf8_lossy(buf)
            ),
        }
    }

    #[test]
    fn watch_subscribed_command() {
        assert_eq!(
            NotifySet::watch_subscribed().command(false).unwrap(),
            "NOTIFY SET (selected (MessageNew MessageExpunge FlagChange)) \
             (subscribed (MessageNew MessageExpunge FlagChange)) \
             (personal (MailboxName SubscriptionChange))"
        );
    }

    #[test]
    fn mailbox_filters_encode_names_per_session() {
        let set = NotifySet {
            status: true,
            groups: vec![
                EventGroup {
                    filter: Filter::Mailboxes(vec!["Entwürfe".into(), "Sent".into()]),
                    events: MESSAGE_EVENTS.to_vec(),
                },
                EventGroup {
                    filter: Filter::Subtree(vec!["Archive".into()]),
                    events: vec![],
                },
            ],
        };
        assert_eq!(
            set.command(false).unwrap(),
            "NOTIFY SET STATUS (mailboxes (\"Entw&APw-rfe\" \"Sent\") \
             (MessageNew MessageExpunge FlagChange)) (subtree (\"Archive\") NONE)"
        );
        assert!(set.command(true).unwrap().contains("\"Entwürfe\""));
    }

    #[test]
    fn invalid_sets_are_refused() {
        let group = |filter, events: &[Event]| EventGroup {
            filter,
            events: events.to_vec(),
        };
        let cmd = |groups| {
            NotifySet {
                status: false,
                groups,
            }
            .command(false)
        };
        assert_eq!(cmd(vec![]), Err(NotifyError::NoGroups));
        assert_eq!(
            cmd(vec![group(Filter::Inboxes, &[Event::MessageNew])]),
            Err(NotifyError::IncompleteMessageEvents("inboxes"))
        );
        assert_eq!(
            cmd(vec![group(Filter::Inboxes, &[Event::FlagChange])]),
            Err(NotifyError::IncompleteMessageEvents("inboxes"))
        );
        assert_eq!(
            cmd(vec![group(Filter::Selected, &[Event::MailboxName])]),
            Err(NotifyError::MailboxEventOnSelected)
        );
        assert_eq!(
            cmd(vec![
                group(Filter::Selected, MESSAGE_EVENTS),
                group(Filter::SelectedDelayed, MESSAGE_EVENTS)
            ]),
            Err(NotifyError::DuplicateFilter("selected-delayed"))
        );
        assert_eq!(
            cmd(vec![group(Filter::Mailboxes(vec![]), MESSAGE_EVENTS)]),
            Err(NotifyError::EmptyMailboxList("mailboxes"))
        );
        assert_eq!(
            cmd(vec![group(
                Filter::Mailboxes(vec!["a\rb".into()]),
                MESSAGE_EVENTS
            )]),
            Err(NotifyError::BadMailboxName("a\rb".into()))
        );
    }

    #[test]
    fn status_with_extension_items() {
        let (n, r) = done(
            b"* STATUS \"Entw&APw-rfe\" (MESSAGES 4 UIDNEXT 9 UIDVALIDITY 1 UNSEEN 2 \
              HIGHESTMODSEQ 77 SIZE 123456 DELETED 1 APPENDLIMIT NIL \
              MAILBOXID (F2212ea87-6097-4256-9d51-71338625) X-NEW (a b))\r\nA1 OK\r\n",
            false,
        );
        assert!(n > 0);
        assert_eq!(r.mailbox, "Entwürfe");
        assert_eq!(r.wire_name, "Entw&APw-rfe");
        let s = r.status;
        assert_eq!(s.messages, Some(4));
        assert_eq!(s.uidnext, Some(9));
        assert_eq!(s.unseen, Some(2));
        assert_eq!(s.highestmodseq, Some(77));
        assert_eq!(s.size, Some(123_456));
        assert_eq!(s.deleted, Some(1));
        assert_eq!(s.appendlimit, Some(None));
        assert_eq!(
            s.mailboxid.as_deref(),
            Some("F2212ea87-6097-4256-9d51-71338625")
        );
    }

    #[test]
    fn status_names_inbox_atoms_literals_and_utf8() {
        assert_eq!(
            done(b"* STATUS inbox (MESSAGES 1)\r\n", false).1.mailbox,
            "INBOX"
        );
        assert_eq!(
            done(b"* STATUS Work (MESSAGES 1)\r\n", false).1.mailbox,
            "Work"
        );
        assert_eq!(
            done(b"* STATUS {4}\r\nA B) (MESSAGES 1)\r\n", false)
                .1
                .mailbox,
            "A B)"
        );
        let utf8 = "* STATUS \"Entwürfe\" (APPENDLIMIT 1024) \r\n";
        let r = done(utf8.as_bytes(), true).1;
        assert_eq!(r.mailbox, "Entwürfe");
        assert_eq!(r.status.appendlimit, Some(Some(1024)));
        assert_eq!(
            done(b"* STATUS x ()\r\n", false).1.status,
            MailboxStatus::default()
        );
    }

    #[test]
    fn status_truncation_and_garbage() {
        let full = b"* STATUS \"Sent\" (MESSAGES 3 UIDNEXT 12)\r\n";
        for cut in 1..full.len() {
            assert_eq!(
                parse_status(&full[..cut], false),
                Parsed::Incomplete,
                "cut {cut}"
            );
        }
        for buf in [
            &b"* LIST () \"/\" x\r\n"[..],
            b"* STATUS x (MESSAGES x)\r\n",
            b"* STATUS x (UIDNEXT 0)\r\n",
            b"* STATUS x MESSAGES 1\r\n",
            b"* STATUS \"\xff\" (MESSAGES 1)\r\n",
            b"* STATUS x (MESSAGES 1\r\n",
        ] {
            assert!(
                matches!(parse_status(buf, false), Parsed::Bad(_)),
                "{:?}",
                String::from_utf8_lossy(buf)
            );
        }
    }

    #[test]
    fn signals_in_response_text() {
        assert_eq!(
            signal("[NOTIFICATIONOVERFLOW] too many"),
            Some(Signal::Overflow)
        );
        assert_eq!(
            signal("[BADEVENT (MessageNew MessageExpunge FlagChange)] nope"),
            Some(Signal::BadEvent(vec![
                "MessageNew".into(),
                "MessageExpunge".into(),
                "FlagChange".into()
            ]))
        );
        assert_eq!(signal("[ALERT] hello"), None);
        assert_eq!(signal("plain text"), None);
        assert_eq!(signal("[unterminated"), None);
    }

    #[test]
    fn tracker_reports_changes_after_first_sighting() {
        let st = |messages, uidnext| MailboxStatus {
            messages: Some(messages),
            uidnext: Some(uidnext),
            ..MailboxStatus::default()
        };
        let mut t = StatusTracker::new();
        assert!(!t.observe("Sent", st(3, 10)));
        assert!(!t.observe("Sent", st(3, 10)));
        assert!(t.observe("Sent", st(4, 11)));
        // An expunge then an append keeps MESSAGES but moves UIDNEXT.
        assert!(t.observe("Sent", st(4, 12)));
        // A reply without the compared items is not a change.
        assert!(!t.observe("Sent", MailboxStatus::default()));
        assert!(!t.observe("Work", st(1, 2)));
        t.retain(&["Work"]);
        assert_eq!(t.len(), 1);
        assert!(!t.observe("Sent", st(9, 99)));
    }

    #[test]
    fn fuzz_corpus_never_panics() {
        for buf in [
            &b""[..],
            b"* STATUS ",
            b"* STATUS {99999999}\r\n",
            b"* STATUS x (MAILBOXID ())\r\n",
            b"* STATUS x (HIGHESTMODSEQ 99999999999999999999)\r\n",
            b"[BADEVENT",
            b"[]",
            "[BADEVENT (\u{1f4e7})]".as_bytes(),
        ] {
            fuzz(buf);
        }
    }
}
