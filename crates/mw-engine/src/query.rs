//! The frozen `Email/query` filter + sort surface (§2.1) and saved searches.
//!
//! ## Scaffolder note (e0)
//! e0 freezes the supported filter/sort set and the search-routing predicate;
//! e9 owns evaluation — the SQL fast path for pure `inMailbox`, and routing to
//! `mw-search` for any full-text/attachment condition.

use serde::{Deserialize, Serialize};

/// The frozen `Email/query` filter (§2.1). Fields are optional; absent = no
/// constraint. `serde(default)` so partial JMAP filters deserialize.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EmailFilter {
    pub in_mailbox: Option<String>,
    pub in_mailbox_other_than: Option<Vec<String>>,
    pub text: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub cc: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub has_keyword: Option<String>,
    pub not_keyword: Option<String>,
    pub has_attachment: Option<bool>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    /// Attachment filename substring (routed to `mw-search`).
    pub filename: Option<String>,
    /// A8 (26.19, SPEC §10.4/§14.3): opt in to embedding re-ranking of this
    /// query's results. The web has sent this since V7; before 26.19 nothing
    /// server-side read it, so it deserialized into nothing.
    ///
    /// It is deliberately NOT part of [`EmailFilter::needs_search`]: on its own it
    /// adds no *retrieval* constraint, so a filter carrying only `inMailbox` +
    /// `semantic` still takes the SQL fast path and is unaffected — re-rank only
    /// ever re-orders a hit list the lexical index already produced. Absent or
    /// `false` means the query is byte-identical to 26.18.
    pub semantic: Option<bool>,
}

impl EmailFilter {
    /// Whether this filter must route to the full-text index (`mw-search`)
    /// rather than the SQL fast path: every condition except `inMailbox`,
    /// which is the only one the fast path evaluates. A condition left out of
    /// this list is dropped on that path and the whole mailbox comes back, as
    /// the keyword, date and size conditions did before 26.20 (t28-e1).
    ///
    /// `inMailboxOtherThan` is not listed: on its own it selects no route (see
    /// `query_route` in `jmap.rs`), and the index applies it whenever another
    /// condition routes there.
    pub fn needs_search(&self) -> bool {
        self.text.is_some()
            || self.from.is_some()
            || self.to.is_some()
            || self.cc.is_some()
            || self.subject.is_some()
            || self.body.is_some()
            || self.filename.is_some()
            || self.has_attachment.is_some()
            || self.has_keyword.is_some()
            || self.not_keyword.is_some()
            || self.before.is_some()
            || self.after.is_some()
            || self.min_size.is_some()
            || self.max_size.is_some()
    }
}

/// Frozen `Email/query` sort properties (§2.1). Default is `receivedAt` desc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortProperty {
    ReceivedAt,
    Size,
    From,
    Subject,
}

/// A JMAP sort comparator over a frozen [`SortProperty`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparator {
    pub property: SortProperty,
    #[serde(default)]
    pub is_ascending: bool,
}

/// A saved search surfaced as a virtual search folder (`role:null` +
/// `mailwomanSearchQuery`) in `Mailbox/get` (§2.1, §2.7 `saved_searches`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSearch {
    pub id: String,
    pub name: String,
    /// The frozen operator/filter query this folder materializes (JSON).
    pub query: String,
    /// Whether it appears as a virtual folder (vs a saved query only).
    pub as_folder: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_in_mailbox_stays_sql_fast_path() {
        let f = EmailFilter {
            in_mailbox: Some("mbox1".into()),
            ..Default::default()
        };
        assert!(!f.needs_search());
    }

    #[test]
    fn semantic_flag_alone_does_not_change_routing() {
        // A8: the flag re-orders results, it never adds a retrieval constraint —
        // so a pure `inMailbox` filter carrying it stays on the SQL fast path.
        let f = EmailFilter {
            in_mailbox: Some("mbox1".into()),
            semantic: Some(true),
            ..Default::default()
        };
        assert!(!f.needs_search());
    }

    #[test]
    fn semantic_flag_deserializes_from_the_wire_filter() {
        // The exact JSON `apps/web` has been sending since V7 (`jmap-types.ts`).
        let f: EmailFilter =
            serde_json::from_str(r#"{"inMailbox":"mb1","text":"invoice","semantic":true}"#)
                .expect("filter parses");
        assert_eq!(f.semantic, Some(true));
        // ...and an ordinary search still reads as "not semantic", not as `false`.
        let plain: EmailFilter =
            serde_json::from_str(r#"{"inMailbox":"mb1","text":"invoice"}"#).expect("parses");
        assert_eq!(plain.semantic, None);
    }

    #[test]
    fn keyword_date_and_size_conditions_route_to_search() {
        // Each of the six on its own, beside `inMailbox`: the fast path reads
        // only `inMailbox`, so any of these left there is silently dropped.
        let base = || EmailFilter {
            in_mailbox: Some("mbox1".into()),
            ..Default::default()
        };
        assert!(!base().needs_search());
        let cases: [(&str, EmailFilter); 6] = [
            (
                "hasKeyword",
                EmailFilter {
                    has_keyword: Some("$flagged".into()),
                    ..base()
                },
            ),
            (
                "notKeyword",
                EmailFilter {
                    not_keyword: Some("$seen".into()),
                    ..base()
                },
            ),
            (
                "before",
                EmailFilter {
                    before: Some("2026-01-01T00:00:00Z".into()),
                    ..base()
                },
            ),
            (
                "after",
                EmailFilter {
                    after: Some("2026-01-01T00:00:00Z".into()),
                    ..base()
                },
            ),
            (
                "minSize",
                EmailFilter {
                    min_size: Some(1),
                    ..base()
                },
            ),
            (
                "maxSize",
                EmailFilter {
                    max_size: Some(1),
                    ..base()
                },
            ),
        ];
        for (name, filter) in cases {
            assert!(filter.needs_search(), "{name} must route to the index");
        }
    }

    #[test]
    fn text_condition_routes_to_search() {
        let f = EmailFilter {
            in_mailbox: Some("mbox1".into()),
            subject: Some("invoice".into()),
            ..Default::default()
        };
        assert!(f.needs_search());
    }
}
