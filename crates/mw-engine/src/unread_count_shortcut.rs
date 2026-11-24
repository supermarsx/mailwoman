//! The folder unread count from the stored counter (t50hp-e26, plan A3 S3-a).
//!
//! The web client's list batch asks `Email/query {inMailbox: <folder>,
//! notKeyword: "$seen"}` with `limit` 0 or 1 and `calculateTotal`, only to
//! learn how many unread messages the folder holds. `notKeyword` routes to the
//! index (`EmailFilter::needs_search`), which enumerated and loaded every
//! unread document of the folder for that number. `mailboxes.unread` holds it
//! exactly (0122: every row write maintains it in its own transaction).
//!
//! This module only recognises the shape and counts what happened; the
//! answer itself is built in `jmap.rs`'s `query_page`. Anything that is not
//! exactly this shape keeps the generic path.
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

use crate::query::EmailFilter;

/// The JMAP keyword the shortcut answers for. Exact spelling only.
const SEEN: &str = "$seen";

/// Why an unread-count-shaped query kept the generic path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fallback {
    /// A sort other than the default, or tie breakers.
    Sort,
    /// `anchor` or a `position` other than 0.
    Window,
    /// No `limit`, or a `limit` above 1.
    Limit,
    /// `calculateTotal` not asked for.
    NoTotal,
    /// Not a real folder of this account (saved-search folder, foreign id).
    Folder,
    /// Zero-access account, or a posture source this check cannot read.
    Posture,
    /// More unread messages than the index reports a total for.
    OverCap,
}

/// Process-wide counters, read by [`crate::Engine::unread_count_shortcut_status`].
static TAKEN_LIMIT0: AtomicU64 = AtomicU64::new(0);
static TAKEN_LIMIT1: AtomicU64 = AtomicU64::new(0);
static FALLBACK: [AtomicU64; 7] = [const { AtomicU64::new(0) }; 7];

/// Snapshot of the shortcut counters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnreadCountShortcutStatus {
    pub taken_limit0: u64,
    pub taken_limit1: u64,
    pub fallback_sort: u64,
    pub fallback_window: u64,
    pub fallback_limit: u64,
    pub fallback_no_total: u64,
    pub fallback_folder: u64,
    pub fallback_posture: u64,
    pub fallback_over_cap: u64,
}

impl crate::Engine {
    /// How often `Email/query` answered a folder's unread count from the
    /// stored counter, and why the shaped queries that did not fell back.
    /// Process-wide, monotonic, read-only.
    pub fn unread_count_shortcut_status(&self) -> UnreadCountShortcutStatus {
        status()
    }
}

fn status() -> UnreadCountShortcutStatus {
    let f = |reason: Fallback| FALLBACK[reason as usize].load(Ordering::Relaxed);
    UnreadCountShortcutStatus {
        taken_limit0: TAKEN_LIMIT0.load(Ordering::Relaxed),
        taken_limit1: TAKEN_LIMIT1.load(Ordering::Relaxed),
        fallback_sort: f(Fallback::Sort),
        fallback_window: f(Fallback::Window),
        fallback_limit: f(Fallback::Limit),
        fallback_no_total: f(Fallback::NoTotal),
        fallback_folder: f(Fallback::Folder),
        fallback_posture: f(Fallback::Posture),
        fallback_over_cap: f(Fallback::OverCap),
    }
}

pub(crate) fn fallback(reason: Fallback) {
    FALLBACK[reason as usize].fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn taken(limit: u64) {
    if limit == 0 { &TAKEN_LIMIT0 } else { &TAKEN_LIMIT1 }.fetch_add(1, Ordering::Relaxed);
}

// Differential tests run the generic path for the same query on the same
// thread (the current-thread test runtime) to compare whole responses.
#[cfg(test)]
thread_local! {
    pub(crate) static GENERIC_ONLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether `filter` (as parsed, after the Events-view rewrite) is exactly
/// `{inMailbox, notKeyword: "$seen"}`. Every other field, `semantic`
/// included, must be absent.
pub(crate) fn is_unread_count_filter(filter: &EmailFilter) -> bool {
    #[cfg(test)]
    if GENERIC_ONLY.with(std::cell::Cell::get) {
        return false;
    }
    filter.in_mailbox.is_some()
        && filter.not_keyword.as_deref() == Some(SEEN)
        && EmailFilter { in_mailbox: None, not_keyword: None, ..filter.clone() } == EmailFilter::default()
}

/// The window and options of an unread-count-shaped query: `Ok(limit)` (0 or
/// 1) when the shortcut may answer, otherwise the reason it may not.
pub(crate) fn window(args: &Value, custom_sort: bool) -> Result<u64, Fallback> {
    if custom_sort {
        return Err(Fallback::Sort);
    }
    if args.get("anchor").is_some_and(|a| !a.is_null())
        || args.get("position").and_then(Value::as_u64).unwrap_or(0) != 0
        || args.get("position").is_some_and(|p| !p.is_null() && p.as_u64().is_none())
    {
        return Err(Fallback::Window);
    }
    let limit = match args.get("limit").and_then(Value::as_u64) {
        Some(n @ (0 | 1)) => n,
        _ => return Err(Fallback::Limit),
    };
    if args.get("calculateTotal").and_then(Value::as_bool) != Some(true) {
        return Err(Fallback::NoTotal);
    }
    Ok(limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parsed(v: Value) -> EmailFilter {
        EmailFilter::parse_saved_json(&v.to_string()).unwrap()
    }

    #[test]
    fn only_the_exact_filter_shape_is_recognised() {
        assert!(is_unread_count_filter(&parsed(json!({"inMailbox":"m","notKeyword":"$seen"}))));
        for other in [
            json!({"inMailbox":"m"}),
            json!({"notKeyword":"$seen"}),
            json!({"inMailbox":"m","notKeyword":"$flagged"}),
            json!({"inMailbox":"m","notKeyword":"$Seen"}),
            json!({"inMailbox":"m","notKeyword":"$seen","text":"x"}),
            json!({"inMailbox":"m","notKeyword":"$seen","hasKeyword":"$flagged"}),
            json!({"inMailbox":"m","notKeyword":"$seen","inMailboxOtherThan":["j"]}),
            json!({"inMailbox":"m","notKeyword":"$seen","inMailboxes":["m"]}),
            json!({"inMailbox":"m","notKeyword":"$seen","semantic":true}),
            json!({"inMailbox":"m","notKeyword":"$seen","semantic":false}),
            json!({"inMailbox":"m","notKeyword":"$seen","after":"2026-01-01T00:00:00Z"}),
            json!({"inMailbox":"m","notKeyword":"$seen","hasCalendar":true}),
        ] {
            assert!(!is_unread_count_filter(&parsed(other.clone())), "{other}");
        }
    }

    /// The shortcut is decided inside `query_page`, never by routing: the
    /// unread-count shape and every neighbour still route to the index
    /// (t28-e1: a condition dropped from routing returned the whole mailbox).
    #[test]
    fn routing_is_unchanged_for_the_shape_and_its_neighbours() {
        for filter in [
            json!({"inMailbox":"m","notKeyword":"$seen"}),
            json!({"notKeyword":"$seen"}),
            json!({"inMailbox":"m","notKeyword":"$flagged"}),
            json!({"inMailbox":"m","notKeyword":"$seen","text":"x"}),
            json!({"inMailbox":"m","notKeyword":"$seen","hasKeyword":"$flagged"}),
            json!({"inMailbox":"m","hasKeyword":"$seen"}),
        ] {
            assert!(parsed(filter.clone()).needs_search(), "{filter}");
        }
        assert!(!parsed(json!({"inMailbox":"m"})).needs_search());
    }

    #[test]
    fn only_limit_zero_or_one_from_the_top_with_a_total() {
        assert_eq!(window(&json!({"limit":1,"calculateTotal":true}), false), Ok(1));
        assert_eq!(window(&json!({"limit":0,"position":0,"calculateTotal":true}), false), Ok(0));
        assert_eq!(window(&json!({"limit":1,"calculateTotal":true}), true), Err(Fallback::Sort));
        assert_eq!(window(&json!({"limit":1,"position":1,"calculateTotal":true}), false), Err(Fallback::Window));
        assert_eq!(window(&json!({"limit":1,"anchor":"x","calculateTotal":true}), false), Err(Fallback::Window));
        assert_eq!(window(&json!({"limit":2,"calculateTotal":true}), false), Err(Fallback::Limit));
        assert_eq!(window(&json!({"calculateTotal":true}), false), Err(Fallback::Limit));
        assert_eq!(window(&json!({"limit":1}), false), Err(Fallback::NoTotal));
        assert_eq!(window(&json!({"limit":1,"calculateTotal":false}), false), Err(Fallback::NoTotal));
    }
}
