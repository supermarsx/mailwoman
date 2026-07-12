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
}

impl EmailFilter {
    /// Whether this filter must route to the full-text index (`mw-search`)
    /// rather than the SQL fast path (frozen routing rule, §2.1): any
    /// text/from/to/cc/subject/body/filename/hasAttachment condition.
    pub fn needs_search(&self) -> bool {
        self.text.is_some()
            || self.from.is_some()
            || self.to.is_some()
            || self.cc.is_some()
            || self.subject.is_some()
            || self.body.is_some()
            || self.filename.is_some()
            || self.has_attachment.is_some()
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
    fn text_condition_routes_to_search() {
        let f = EmailFilter {
            in_mailbox: Some("mbox1".into()),
            subject: Some("invoice".into()),
            ..Default::default()
        };
        assert!(f.needs_search());
    }
}
