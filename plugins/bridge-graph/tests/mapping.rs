//! Host-target coverage of the whole Graph mapping, driven through a fixture
//! `Transport` (no wasm toolchain, no live tenant). Exercises mail (folders, delta,
//! Focused-Inbox, fetch, flags, move, submit), contacts + GAL, calendar (shared +
//! rooms + free/busy), To-Do, and the Outlook-parity caps incl. the recall honesty
//! matrix. The in-jail component round-trip lives in `tests/jail.rs`.

use bridge_graph::calendar;
use bridge_graph::caps;
use bridge_graph::contacts;
use bridge_graph::fixtures::FixtureTransport;
use bridge_graph::graph::{BridgeError, GraphClient};
use bridge_graph::mail;
use bridge_graph::todo;
use bridge_graph::types::{
    Flag, MailboxRef, MessageRef, SyncCursor, KEYWORD_FOCUSED, KEYWORD_OTHER,
};

const ACCOUNT: &str = "admin@vogue-homes.com";

fn transport() -> FixtureTransport {
    FixtureTransport::load_default()
}

fn inbox_ref() -> MailboxRef {
    MailboxRef {
        name: "inbox".into(),
        uidvalidity: 1,
    }
}

// ── mail ──────────────────────────────────────────────────────────────────────

#[test]
fn list_mailboxes_maps_roles_and_addressing_keys() {
    let tr = transport();
    let c = GraphClient::new(&tr, ACCOUNT);
    let boxes = mail::list_mailboxes(&c).unwrap();
    assert_eq!(boxes.len(), 3);

    let inbox = &boxes[0];
    assert_eq!(inbox.role, "inbox");
    assert_eq!(inbox.mailbox_ref.name, "inbox"); // well-known name is the key
    assert_eq!(inbox.total, 12);
    assert_eq!(inbox.unread, 3);

    // A folder with no wellKnownName falls back to its opaque id as the key.
    let projects = &boxes[2];
    assert_eq!(projects.role, "none");
    assert_eq!(projects.mailbox_ref.name, "AAMk-projects");
}

#[test]
fn delta_sync_maps_focused_flags_and_cursor() {
    let tr = transport();
    let c = GraphClient::new(&tr, ACCOUNT);

    // Initial sync (empty cursor) — both messages are "added".
    let delta = mail::sync_mailbox(&c, &inbox_ref(), &SyncCursor::default()).unwrap();
    assert_eq!(delta.added.len(), 2);
    assert!(delta.removed.is_empty());

    // msg-1: unread + focused ⇒ [$Focused]; msg-2: read + flagged + other.
    let (_r1, f1) = &delta.flag_changes[0];
    assert!(f1.contains(&Flag::Keyword(KEYWORD_FOCUSED.into())));
    assert!(!f1.contains(&Flag::Seen));

    let (_r2, f2) = &delta.flag_changes[1];
    assert!(f2.contains(&Flag::Seen));
    assert!(f2.contains(&Flag::Flagged));
    assert!(f2.contains(&Flag::Keyword(KEYWORD_OTHER.into())));

    // The cursor is the Graph deltaLink verbatim (lossless SyncCursor::Plugin).
    let cursor_str = String::from_utf8(delta.next_cursor.opaque.clone()).unwrap();
    assert!(cursor_str.contains("$deltatoken=NEXT1"));

    // Follow-up sync with that cursor: msg-2 removed, msg-1 flag change, new cursor.
    let delta2 = mail::sync_mailbox(&c, &inbox_ref(), &delta.next_cursor).unwrap();
    assert_eq!(delta2.removed.len(), 1);
    assert_eq!(delta2.removed[0].raw, "AAMk-msg-2");
    let next2 = String::from_utf8(delta2.next_cursor.opaque).unwrap();
    assert!(next2.contains("$deltatoken=NEXT2"));
}

#[test]
fn fetch_raw_returns_mime() {
    let tr = transport();
    let c = GraphClient::new(&tr, ACCOUNT);
    let mref = MessageRef {
        raw: "AAMk-msg-1".into(),
        mailbox: inbox_ref(),
    };
    let msgs = mail::fetch_raw(&c, std::slice::from_ref(&mref)).unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].message_ref, mref);
    let raw = String::from_utf8(msgs[0].raw.clone()).unwrap();
    assert!(raw.contains("Subject: Q3 roadmap"));
}

#[test]
fn store_flags_move_and_submit_reach_graph() {
    let tr = transport();
    let c = GraphClient::new(&tr, ACCOUNT);
    let mref = MessageRef {
        raw: "AAMk-msg-1".into(),
        mailbox: inbox_ref(),
    };

    // Mark read (isRead) + focused (inferenceClassification) — Focused-Inbox write.
    mail::store_flags(
        &c,
        std::slice::from_ref(&mref),
        &[Flag::Seen, Flag::Keyword(KEYWORD_FOCUSED.into())],
        &[],
    )
    .unwrap();

    // A purely IMAP-only add with nothing Graph-expressible is a no-op success.
    mail::store_flags(&c, std::slice::from_ref(&mref), &[Flag::Recent], &[]).unwrap();

    let archive = MailboxRef {
        name: "archive".into(),
        uidvalidity: 1,
    };
    mail::move_messages(&c, std::slice::from_ref(&mref), &archive).unwrap();

    let sent = mail::submit(&c, &inbox_ref(), b"Subject: hi\r\n\r\nbody\r\n").unwrap();
    assert!(sent.raw.starts_with("sent:"));
}

// ── contacts + GAL ────────────────────────────────────────────────────────────

#[test]
fn addrbook_search_merges_and_dedupes() {
    let tr = transport();
    let c = GraphClient::new(&tr, ACCOUNT);
    let entries = contacts::addrbook_search(&c, "car").unwrap();

    // Carol appears in contacts, people, AND the GAL — must be de-duplicated.
    let carol = entries
        .iter()
        .filter(|e| e.to_ascii_lowercase().contains("carol@vogue-homes.com"))
        .count();
    assert_eq!(carol, 1, "carol de-duplicated across sources: {entries:?}");

    // GAL fallback to userPrincipalName when `mail` is null (Frank).
    assert!(entries.iter().any(|e| e.contains("frank@vogue-homes.com")));
    // People-only relevance hit (Erik).
    assert!(entries.iter().any(|e| e.contains("erik@vogue-homes.com")));
}

// ── calendar ──────────────────────────────────────────────────────────────────

#[test]
fn calendars_flags_shared_and_events_delta() {
    let tr = transport();
    let c = GraphClient::new(&tr, ACCOUNT);

    let cals = calendar::list_calendars(&c, ACCOUNT).unwrap();
    assert_eq!(cals.len(), 2);
    assert!(!cals[0].shared, "own calendar not shared");
    assert!(cals[1].shared, "grace's calendar is shared");
    assert!(!cals[1].can_edit);

    let delta = calendar::sync_events(&c, &SyncCursor::default()).unwrap();
    assert_eq!(delta.events.len(), 1);
    assert_eq!(delta.events[0].subject, "Sprint review");
    assert_eq!(delta.events[0].location.as_deref(), Some("Room Everest"));
    assert_eq!(delta.removed, vec!["evt-2".to_string()]);
    assert!(String::from_utf8(delta.next_cursor.opaque)
        .unwrap()
        .contains("EVTNEXT"));
}

#[test]
fn rooms_and_free_busy() {
    let tr = transport();
    let c = GraphClient::new(&tr, ACCOUNT);

    let rooms = calendar::find_rooms(&c).unwrap();
    assert_eq!(rooms.len(), 2);
    assert_eq!(rooms[0].name, "Everest");
    assert_eq!(rooms[0].capacity, Some(12));

    let sched = calendar::get_schedule(
        &c,
        &[
            "everest@vogue-homes.com".into(),
            "grace@vogue-homes.com".into(),
        ],
        "2026-07-15T00:00:00",
        "2026-07-15T23:59:59",
    )
    .unwrap();
    assert_eq!(sched.len(), 2);
    assert_eq!(sched[0].1, "002200"); // busy view for the room
}

// ── To-Do ─────────────────────────────────────────────────────────────────────

#[test]
fn todo_lists_and_tasks() {
    let tr = transport();
    let c = GraphClient::new(&tr, ACCOUNT);

    let lists = todo::list_lists(&c).unwrap();
    assert_eq!(lists.len(), 2);
    assert!(lists[0].owner);
    assert!(!lists[1].owner);

    let tasks = todo::list_tasks(&c, "list-tasks").unwrap();
    assert_eq!(tasks.len(), 2);
    assert!(!tasks[0].completed);
    assert_eq!(tasks[0].title, "Draft release notes");
    assert!(tasks[1].completed);
}

// ── Outlook-parity caps + recall honesty matrix ───────────────────────────────

#[test]
fn advertised_caps_include_outlook_parity() {
    let caps = caps::backend_caps();
    assert!(caps.reactions && caps.voting && caps.recall && caps.focused_sync);
}

#[test]
fn recall_is_never_guaranteed_and_declines_when_read() {
    let tr = transport();
    let c = GraphClient::new(&tr, ACCOUNT);

    // Unread ⇒ request is issued, but the outcome is NEVER guaranteed.
    let unread = caps::recall(&c, "recall-unread").unwrap();
    assert!(unread.requested);
    assert!(!unread.guaranteed, "Graph recall is never guaranteed");
    assert!(unread.note.contains("best-effort"));

    // Already read ⇒ the bridge declines rather than pretend it can recall.
    let read = caps::recall(&c, "recall-read").unwrap();
    assert!(!read.requested);
    assert!(!read.guaranteed);
    assert!(read.note.contains("already read"));
}

#[test]
fn reactions_voting_categories_reach_graph() {
    let tr = transport();
    let c = GraphClient::new(&tr, ACCOUNT);

    assert!(caps::react(&c, "react-msg", "👍").unwrap());
    caps::vote(&c, "vote-msg", "Approve").unwrap();
    // Categories is a genuine, fully-supported Graph field (PATCH /me/messages).
    caps::set_categories(&c, "AAMk-msg-1", &["Blue category".into()]).unwrap();
}

#[test]
fn missing_fixture_is_a_transport_error_not_a_panic() {
    let tr = transport();
    let c = GraphClient::new(&tr, ACCOUNT);
    let err = mail::fetch_raw(
        &c,
        &[MessageRef {
            raw: "nope".into(),
            mailbox: MailboxRef {
                name: "nowhere-folder-xyz".into(),
                uidvalidity: 9,
            },
        }],
    );
    // The $value fixture matches any message id, so this actually succeeds — assert
    // instead that an unmatched *path* surfaces a typed error, never a panic.
    let _ = err; // fetch_raw hits /$value which is fixtured; use a truly-unmatched call:
    let bogus = c.get_bytes("/me/unmatched/endpoint");
    assert!(matches!(bogus, Err(BridgeError::Transport(_))));
}
