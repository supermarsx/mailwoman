//! The `Email/query` planner, slice 1 of "mutable metadata out of the search
//! index" (t50hp-e31; design t50hp-e29 parts 6, 7, 11, 13).
//!
//! Every filter becomes ONE boolean AST (the scope, the filter fields and the
//! parsed `filter.text`), exactly as `search_index::build_search_query` builds
//! it. Each leaf is classified by an exhaustive `match` with no wildcard arm
//! ([`classify`]); the filter fields are read by a full destructuring with no
//! `..` ([`filter_conjuncts`]). A new `Clause` variant or `EmailFilter` field
//! therefore does not compile until it is routed here (the t28-e1 class: a
//! condition missing from routing was silently dropped).
//!
//! Leaf classes in this slice:
//! * Meta (SQL only, never read from the index): `Keyword` / `NotKeyword` of
//!   any non-derived keyword (`$seen`, `$flagged`, labels, ...), `Pinned`,
//!   `FollowUp`. `$mw-followup-active` keeps the index's meaning: a follow-up
//!   time is set, or the message carries a keyword of that name.
//! * Derived (decision P2): a keyword with a derived prefix (`$list:`,
//!   `$supersedes:`, `$mw-mention:`, `$priority-high/low`, the set
//!   `index_publication::patch_current` keeps) is the UNION of the index (its
//!   content-derived value) and SQL (a user keyword of that name).
//! * Content (index only): text, fuzzy, wildcard, sender domain, attachment,
//!   calendar, bulk, mention.
//! * Either: mailbox membership, dates, sizes. SQL on an SQL-only route, the
//!   index otherwise. Mailbox membership stays an index field in this slice and
//!   the index scope (the mailbox-OR guard of `search_index::scope_expr`) is
//!   unchanged; on the combined route the SQL side gets these conditions too,
//!   which only narrows its id set.
//! * Resolved (engine-built, index): `IdSet`, `Account`.
//!
//! Routes:
//! * R1 (`Route::Sql`): at least one Meta leaf, no Content / Derived leaf,
//!   every conjunct expressible as one `ProjectionQuery`, sort receivedAt or
//!   size without tie-breakers, no semantic re-rank. Answered from
//!   `message_state` / `message_keyword` / `message_meta`; the page is the SQL
//!   LIMIT/OFFSET, the total an exact COUNT.
//! * R2 (`Route::Index`): no Meta or Derived leaf. Today's index query, unchanged.
//! * R3 (`Route::Combined`): everything else. The top-level conjuncts that are
//!   pure metadata form one SQL condition (`top`); metadata under OR / NOT
//!   (text operators) becomes a nested SQL condition per maximal expressible
//!   subtree. Each is resolved to `IdSet(ids)` (inject) or, for a single
//!   negatable condition, `NOT IdSet(complement)`, whichever is within the cap;
//!   the whole expression then runs in the index with today's scope, sort,
//!   count and truncation. When `top` is above the cap the index answers the
//!   rest and SQL filters its hits in chunks (filter strategy).
//!
//! Fail closed (design part 6):
//! * an SQL error is an error (serverFail), never rows;
//! * the index side, when needed, keeps today's refusals (read fence);
//! * an id set (and its complement) above the cap is `unsupportedFilter`;
//! * no route drops a condition or falls back to an account-wide result;
//! * if the projection triggers are not installed or are disabled
//!   (`Store::message_projection_installed`), every route that needs SQL
//!   metadata refuses (serverFail); none falls back to the index.
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use mw_search::{Clause, Expr, IdSet, Sort, SortField, TextField};
use mw_store::message_projection::{MailboxScope, ProjectionQuery, ProjectionSort};

use crate::backend::EngineError;
use crate::query::EmailFilter;

/// Largest id set the planner hands the index, and largest index hit list the
/// filter strategy checks in SQL: the index's own enumeration cap.
pub(crate) const COMBINE_CAP: usize = mw_search::MAX_HITS;
/// Ids per SQL membership check in the filter strategy (design part 6 start value).
pub(crate) const FILTER_CHUNK: usize = COMBINE_CAP;
/// The `unsupportedFilter` description for a set above the cap.
pub(crate) const TOO_BROAD: &str = "filter too broad to combine";
/// Metadata subtrees one query may resolve (t49pg-e5b F2: each is an id set
/// held in memory and compiled into the index query).
pub(crate) const MAX_META_NODES: usize = 8;
/// The unsupportedFilter description for too many metadata subtrees.
pub(crate) const TOO_MANY: &str = "too many metadata conditions to combine";

static IDS_MATERIALISED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Id rows the planner has read from SQL for id sets, process-wide (tests and
/// diagnostics: the F2 memory bound).
pub(crate) fn ids_materialised() -> u64 {
    IDS_MATERIALISED.load(std::sync::atomic::Ordering::Relaxed)
}

pub(crate) fn note_materialised(n: usize) {
    IDS_MATERIALISED.fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
}

/// Where a leaf condition is answered. See the module note.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LeafClass {
    Meta,
    Derived,
    Content,
    Either,
    Resolved,
}

/// The derived keyword names `index_publication::patch_current` carries over
/// from the stored document (a guard test compares the two sources).
pub(crate) fn is_derived_keyword(k: &str) -> bool {
    k.starts_with("$mw-mention:")
        || k.starts_with("$list:")
        || k.starts_with("$supersedes:")
        || k == mw_search::KW_PRIORITY_HIGH
        || k == mw_search::KW_PRIORITY_LOW
}

/// The class of one leaf. Exhaustive, no wildcard arm: a new `Clause` variant
/// fails to compile here until it is routed.
pub(crate) fn classify(c: &Clause) -> LeafClass {
    match c {
        Clause::Keyword(k) | Clause::NotKeyword(k) => {
            if is_derived_keyword(k) {
                LeafClass::Derived
            } else {
                LeafClass::Meta
            }
        }
        Clause::Pinned(_) => LeafClass::Meta,
        Clause::FollowUp => LeafClass::Meta,
        Clause::Text { .. } => LeafClass::Content,
        Clause::Fuzzy { .. } => LeafClass::Content,
        Clause::Wildcard { .. } => LeafClass::Content,
        Clause::FromDomain(_) => LeafClass::Content,
        Clause::HasAttachment(_) => LeafClass::Content,
        Clause::Mention(_) => LeafClass::Content,
        Clause::HasCalendar(_) => LeafClass::Content,
        Clause::Bulk(_) => LeafClass::Content,
        Clause::CalendarEndAfter(_) => LeafClass::Content,
        // Slice 2: folder membership is SQL (the index's mailbox field is
        // not read any more).
        Clause::Mailbox(_) => LeafClass::Meta,
        Clause::DateRange { .. } => LeafClass::Either,
        Clause::SizeRange { .. } => LeafClass::Either,
        Clause::IdSet(_) => LeafClass::Resolved,
        Clause::Account(_) => LeafClass::Resolved,
    }
}

/// Whether `e` holds a leaf that SQL must answer (Meta or Derived).
pub(crate) fn has_meta(e: &Expr) -> bool {
    match e {
        Expr::All => false,
        Expr::Clause(c) => matches!(classify(c), LeafClass::Meta | LeafClass::Derived),
        Expr::And(items) | Expr::Or(items) => items.iter().any(has_meta),
        Expr::Not(inner) => has_meta(inner),
    }
}

/// Parse a filter date bound (RFC3339, bare unix seconds or `YYYY-MM-DD`), as
/// `search_index` always has.
pub(crate) fn parse_date_bound(s: &str) -> Option<i64> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp());
    }
    if let Ok(secs) = s.parse::<i64>() {
        return Some(secs);
    }
    let d = chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()?;
    Some(d.and_hms_opt(0, 0, 0)?.and_utc().timestamp())
}

/// The filter's conditions other than the scope (`inMailbox`), as the AND
/// list the index query is built from; `None` when it matches nothing.
///
/// `owned`: the account's folders, to keep only owned `inMailboxes` (the
/// index needs that, it has no ownership of its own); `None` leaves the list
/// as given, for SQL, which checks ownership against `mailboxes.account_id`.
///
/// A full destructuring with no `..`: a new `EmailFilter` field does not
/// compile until it is read here (or declared as no retrieval condition).
pub(crate) fn filter_conjuncts(
    filter: &EmailFilter,
    owned: Option<&[String]>,
) -> Option<Vec<Expr>> {
    let EmailFilter {
        in_mailbox: _scope_see_search_index_scope_expr,
        in_mailboxes,
        in_mailbox_other_than,
        text,
        from,
        to,
        cc,
        subject,
        body,
        has_keyword,
        not_keyword,
        has_attachment,
        before,
        after,
        min_size,
        max_size,
        filename,
        semantic: _reorders_only_no_retrieval_condition,
        has_calendar,
        calendar_end_after,
    } = filter;
    let mut clauses: Vec<Expr> = Vec::new();
    if let Some(include) = in_mailboxes {
        let included: Vec<_> = include
            .iter()
            .filter(|id| owned.is_none_or(|o| o.iter().any(|m| m == *id)))
            .map(|id| Expr::Clause(Clause::Mailbox(id.clone())))
            .collect();
        if included.is_empty() {
            return None;
        }
        clauses.push(Expr::Or(included));
    }
    if let Some(exclude) = in_mailbox_other_than {
        for mb in exclude {
            clauses.push(Expr::Not(Box::new(Expr::Clause(Clause::Mailbox(
                mb.clone(),
            )))));
        }
    }
    let text_clause = |field: TextField, value: &str| {
        Expr::Clause(Clause::Text {
            field,
            value: value.to_string(),
            phrase: false,
        })
    };
    if let Some(v) = text {
        // The web packs the whole operator string (`subject:foo`, booleans,
        // quoted phrases) into `filter.text`; route it through the shared
        // operator parser. A bare word parses to an all-fields clause. On a
        // parse error, fall back to the literal all-fields term.
        match mw_search::parse_query(v) {
            Ok(parsed) if !matches!(parsed.expr, Expr::All) => clauses.push(parsed.expr),
            Ok(_) => {}
            Err(_) => clauses.push(text_clause(TextField::All, v)),
        }
    }
    for (value, field) in [
        (from, TextField::From),
        (to, TextField::To),
        (cc, TextField::Cc),
        (subject, TextField::Subject),
        (body, TextField::Body),
        (filename, TextField::Filename),
    ] {
        if let Some(v) = value {
            clauses.push(text_clause(field, v));
        }
    }
    if let Some(b) = has_attachment {
        clauses.push(Expr::Clause(Clause::HasAttachment(*b)));
    }
    if let Some(b) = has_calendar {
        clauses.push(Expr::Clause(Clause::HasCalendar(*b)));
    }
    if let Some(at) = calendar_end_after.as_deref().and_then(parse_date_bound) {
        clauses.push(Expr::Clause(Clause::CalendarEndAfter(at)));
    }
    if let Some(k) = has_keyword {
        clauses.push(Expr::Clause(Clause::Keyword(k.clone())));
    }
    if let Some(k) = not_keyword {
        clauses.push(Expr::Clause(Clause::NotKeyword(k.clone())));
    }
    if before.is_some() || after.is_some() {
        clauses.push(Expr::Clause(Clause::DateRange {
            after: after.as_deref().and_then(parse_date_bound),
            before: before.as_deref().and_then(parse_date_bound),
        }));
    }
    if min_size.is_some() || max_size.is_some() {
        clauses.push(Expr::Clause(Clause::SizeRange {
            larger: *min_size,
            smaller: *max_size,
        }));
    }
    Some(clauses)
}

/// The query's folder scope after the ownership guard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Scope {
    /// `inMailbox`: one owned folder.
    Folder(String),
    /// Every folder of the account.
    Account,
}

fn insert_sorted(v: &mut Vec<String>, value: &str) {
    if let Err(at) = v.binary_search_by(|x| x.as_str().cmp(value)) {
        v.insert(at, value.to_owned());
    }
}

/// One conjunction of metadata conditions that `ProjectionQuery` expresses.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MetaConj {
    /// Messages in one of these folders (intersection of every `in` seen).
    pub(crate) in_mailboxes: Option<Vec<String>>,
    /// Messages in none of these (only while `in_mailboxes` is `None`).
    pub(crate) not_in: Vec<String>,
    pub(crate) keywords: Vec<String>,
    pub(crate) not_keywords: Vec<String>,
    pub(crate) pinned: Option<bool>,
    pub(crate) follow_up: bool,
    pub(crate) follow_up_active: Option<bool>,
    pub(crate) after: Option<i64>,
    pub(crate) before: Option<i64>,
    pub(crate) larger: Option<u64>,
    pub(crate) smaller: Option<u64>,
    /// A contradiction or an empty condition: matches nothing.
    pub(crate) nothing: bool,
    /// Meta leaves absorbed (0 = only size conditions).
    pub(crate) meta_leaves: usize,
    /// `in_mailboxes` holds only the query's own folder scope (from
    /// [`MetaConj::scoped`] or the narrowing of a nested node), which the
    /// top-level condition enforces anyway; a complement may then drop it.
    pub(crate) in_from_scope: bool,
}

impl MetaConj {
    /// The scope as a condition.
    pub(crate) fn scoped(scope: &Scope) -> Self {
        let mut c = MetaConj::default();
        if let Scope::Folder(mb) = scope {
            c.intersect_in(std::slice::from_ref(mb));
            c.meta_leaves += 1;
            c.in_from_scope = true;
        }
        c
    }

    fn intersect_in(&mut self, ids: &[String]) {
        let mut set: Vec<String> = Vec::new();
        for id in ids {
            insert_sorted(&mut set, id);
        }
        self.in_mailboxes = Some(match self.in_mailboxes.take() {
            None => set,
            Some(old) => old.into_iter().filter(|m| set.contains(m)).collect(),
        });
        self.normalise();
    }

    fn normalise(&mut self) {
        if let Some(ins) = &mut self.in_mailboxes {
            ins.retain(|m| !self.not_in.contains(m));
            self.not_in.clear();
            if ins.is_empty() {
                self.nothing = true;
            }
        }
    }

    fn set_flag(slot: &mut Option<bool>, value: bool, nothing: &mut bool) {
        match *slot {
            Some(old) if old != value => *nothing = true,
            _ => *slot = Some(value),
        }
    }

    fn keyword(&mut self, k: &str, present: bool) {
        self.meta_leaves += 1;
        if k == mw_search::KW_FOLLOWUP_ACTIVE {
            Self::set_flag(&mut self.follow_up_active, present, &mut self.nothing);
        } else if present {
            if self.not_keywords.iter().any(|x| x == k) {
                self.nothing = true;
            }
            insert_sorted(&mut self.keywords, k);
        } else {
            if self.keywords.iter().any(|x| x == k) {
                self.nothing = true;
            }
            insert_sorted(&mut self.not_keywords, k);
        }
    }

    /// Absorb one leaf (`positive` = not under a NOT). `false` when this
    /// conjunction cannot express it. Exhaustive, no wildcard arm.
    fn absorb_clause(&mut self, c: &Clause, positive: bool) -> bool {
        match c {
            Clause::Keyword(k) => {
                if is_derived_keyword(k) {
                    return false;
                }
                self.keyword(k, positive);
                true
            }
            Clause::NotKeyword(k) => {
                if is_derived_keyword(k) {
                    return false;
                }
                self.keyword(k, !positive);
                true
            }
            Clause::Pinned(b) => {
                self.meta_leaves += 1;
                Self::set_flag(&mut self.pinned, *b == positive, &mut self.nothing);
                true
            }
            Clause::FollowUp => {
                if !positive {
                    return false;
                }
                self.meta_leaves += 1;
                self.follow_up = true;
                true
            }
            Clause::Mailbox(m) => {
                self.meta_leaves += 1;
                if positive {
                    self.intersect_in(std::slice::from_ref(m));
                    self.in_from_scope = false;
                } else {
                    if self.in_mailboxes.is_none() {
                        insert_sorted(&mut self.not_in, m);
                    }
                    self.normalise_not_in(m);
                }
                true
            }
            // Dates stay with the index (t49pg-e5b F3): its date is the
            // internaldate or, for a row stored without one, the Date header;
            // the projection only has internaldate (NULL there). R1 is
            // therefore never taken with a date condition.
            Clause::DateRange { .. } => false,
            Clause::SizeRange { larger, smaller } => {
                if !positive {
                    return false;
                }
                if larger.is_none() && smaller.is_none() {
                    self.nothing = true;
                }
                if let Some(l) = larger {
                    self.larger = Some(self.larger.map_or(*l, |x| x.max(*l)));
                }
                if let Some(s) = smaller {
                    self.smaller = Some(self.smaller.map_or(*s, |x| x.min(*s)));
                }
                true
            }
            Clause::Text { .. } => false,
            Clause::Fuzzy { .. } => false,
            Clause::Wildcard { .. } => false,
            Clause::FromDomain(_) => false,
            Clause::HasAttachment(_) => false,
            Clause::Mention(_) => false,
            Clause::HasCalendar(_) => false,
            Clause::Bulk(_) => false,
            Clause::CalendarEndAfter(_) => false,
            Clause::IdSet(_) => false,
            Clause::Account(_) => false,
        }
    }

    fn normalise_not_in(&mut self, m: &str) {
        if let Some(ins) = &mut self.in_mailboxes {
            ins.retain(|x| x != m);
            if ins.is_empty() {
                self.nothing = true;
            }
        }
    }

    fn absorb(&mut self, e: &Expr) -> bool {
        match e {
            Expr::All => true,
            Expr::And(items) => items.iter().all(|i| self.absorb(i)),
            Expr::Or(items)
                if !items.is_empty()
                    && items
                        .iter()
                        .all(|i| matches!(i, Expr::Clause(Clause::Mailbox(_)))) =>
            {
                let ids: Vec<String> = items
                    .iter()
                    .filter_map(|i| match i {
                        Expr::Clause(Clause::Mailbox(m)) => Some(m.clone()),
                        _ => None,
                    })
                    .collect();
                self.meta_leaves += 1;
                self.in_from_scope = false;
                self.intersect_in(&ids);
                true
            }
            Expr::Or(_) => false,
            Expr::Clause(c) => self.absorb_clause(c, true),
            Expr::Not(inner) => match inner.as_ref() {
                Expr::All => {
                    self.nothing = true;
                    true
                }
                Expr::Clause(c) => self.absorb_clause(c, false),
                Expr::Not(x) => self.absorb(x),
                Expr::And(_) | Expr::Or(_) => false,
            },
        }
    }

    /// `self` AND `e`, or `None` when the conjunction cannot express `e`.
    pub(crate) fn try_absorb(&self, e: &Expr) -> Option<Self> {
        let mut next = self.clone();
        next.absorb(e).then_some(next)
    }

    /// The complement when this is exactly one negatable condition (a
    /// keyword, a pin, the follow-up marker), else `None`. A folder
    /// restriction (`in_mailboxes`) is kept: the complement is taken within
    /// those folders, so its index form must restrict to them too
    /// ([`complement_expr`]; t49pg-e5b F1).
    pub(crate) fn complement(&self) -> Option<Self> {
        let conditions = self.keywords.len()
            + self.not_keywords.len()
            + usize::from(self.pinned.is_some())
            + usize::from(self.follow_up_active.is_some());
        // Slice 2: a folder of the subtree's own (`in:M` nested in OR/NOT)
        // cannot be put back on the index side (its mailbox field is not
        // maintained), so such a subtree has no complement (t49pg-e5b F1).
        let plain = self.not_in.is_empty()
            && (self.in_mailboxes.is_none() || self.in_from_scope)
            && self.after.is_none()
            && self.before.is_none()
            && self.larger.is_none()
            && self.smaller.is_none()
            && !self.follow_up
            && !self.nothing
            && conditions == 1;
        if !plain {
            return None;
        }
        let mut c = MetaConj {
            meta_leaves: 1,
            in_mailboxes: self.in_mailboxes.clone(),
            in_from_scope: self.in_from_scope,
            ..MetaConj::default()
        };
        match (
            self.keywords.as_slice(),
            self.not_keywords.as_slice(),
            self.pinned,
            self.follow_up_active,
        ) {
            ([k], [], None, None) => c.not_keywords.push(k.clone()),
            ([], [k], None, None) => c.keywords.push(k.clone()),
            ([], [], Some(p), None) => c.pinned = Some(!p),
            ([], [], None, Some(a)) => c.follow_up_active = Some(!a),
            _ => return None,
        }
        Some(c)
    }

    /// The store query for this conjunction.
    pub(crate) fn projection<'a>(
        &'a self,
        account_id: &'a str,
        sort: ProjectionSort,
    ) -> ProjectionQuery<'a> {
        let mailboxes = match &self.in_mailboxes {
            Some(ids) => MailboxScope::In(ids),
            None => MailboxScope::NotIn(&self.not_in),
        };
        ProjectionQuery {
            mailboxes,
            keywords: &self.keywords,
            not_keywords: &self.not_keywords,
            pinned: self.pinned,
            follow_up: self.follow_up,
            follow_up_active: self.follow_up_active,
            received_after: self.after,
            received_before: self.before,
            size_larger: self.larger,
            size_smaller: self.smaller,
            sort,
            ..ProjectionQuery::account(account_id)
        }
    }
}

/// An index expression with metadata subtrees still to resolve in SQL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Node {
    Index(Expr),
    Meta(MetaConj),
    And(Vec<Node>),
    Or(Vec<Node>),
    Not(Box<Node>),
}

/// Rewrite one conjunct: each maximal expressible metadata subtree becomes a
/// `Meta` node, a derived keyword the union (or, negated, the intersection of
/// negations) of its index and SQL halves. Content stays in the index.
pub(crate) fn rewrite(e: &Expr) -> Result<Node, &'static str> {
    if !has_meta(e) {
        return Ok(Node::Index(e.clone()));
    }
    if let Some(c) = MetaConj::default().try_absorb(e)
        && c.meta_leaves > 0
    {
        return Ok(Node::Meta(c));
    }
    let derived = |k: &str| MetaConj {
        keywords: vec![k.to_owned()],
        meta_leaves: 1,
        ..MetaConj::default()
    };
    Ok(match e {
        Expr::And(items) => {
            // The absorbable metadata children form one SQL condition (one id
            // set instead of one per child); the rest is rewritten one by one.
            let mut meta = MetaConj::default();
            let mut rest = Vec::new();
            for item in items {
                match meta.try_absorb(item) {
                    Some(next) if has_meta(item) => meta = next,
                    _ => rest.push(rewrite(item)?),
                }
            }
            if meta.meta_leaves > 0 {
                rest.insert(0, Node::Meta(meta));
            }
            Node::And(rest)
        }
        Expr::Or(items) => Node::Or(items.iter().map(rewrite).collect::<Result<_, _>>()?),
        Expr::Not(inner) => Node::Not(Box::new(rewrite(inner)?)),
        Expr::Clause(Clause::Keyword(k)) if is_derived_keyword(k) => {
            Node::Or(vec![Node::Index(e.clone()), Node::Meta(derived(k))])
        }
        Expr::Clause(Clause::NotKeyword(k)) if is_derived_keyword(k) => Node::And(vec![
            Node::Index(e.clone()),
            Node::Not(Box::new(Node::Meta(derived(k)))),
        ]),
        // A metadata leaf the conjunction refused (NOT FollowUp is negated by
        // the caller's Not node; nothing else reaches here). Never read it
        // from the index.
        Expr::Clause(_) | Expr::All => return Err("metadata condition without a route"),
    })
}

/// How a query is answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    /// Matches nothing (a contradiction); no statement.
    Nothing,
    /// R1: SQL only.
    Sql(MetaConj, ProjectionSort),
    /// R2: the index only, today's query.
    Index,
    /// R3: SQL id sets in the index query.
    Combined {
        /// The top-level metadata conjunction, if any (inject or filter).
        top: Option<MetaConj>,
        /// The other conjuncts (the scope excluded), metadata nested.
        nodes: Vec<Node>,
    },
}

#[cfg(test)]
impl Route {
    /// The design's route name, for tests.
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Route::Nothing => "nothing",
            Route::Sql(..) => "R1",
            Route::Index => "R2",
            Route::Combined { .. } => "R3",
        }
    }
}

/// The SQL order for an R1 sort, or `None` when only the index sorts it.
pub(crate) fn sql_sort(sort: &Sort, tie_breakers: &[Sort]) -> Option<ProjectionSort> {
    if !tie_breakers.is_empty() {
        return None;
    }
    match (sort.field, sort.ascending) {
        (SortField::ReceivedAt, false) => Some(ProjectionSort::ReceivedDesc),
        (SortField::ReceivedAt, true) => Some(ProjectionSort::ReceivedAsc),
        (SortField::Size, false) => Some(ProjectionSort::SizeDesc),
        (SortField::Size, true) => Some(ProjectionSort::SizeAsc),
        (SortField::SentAt, _)
        | (SortField::Importance, _)
        | (SortField::GroupReceived, _)
        | (SortField::GroupSent, _)
        | (SortField::GroupFrom, _)
        | (SortField::GroupSubject, _)
        | (SortField::GroupSize, _)
        | (SortField::From, _)
        | (SortField::Subject, _) => None,
    }
}

/// Plan one query. `conjuncts` = [`filter_conjuncts`] (scope excluded);
/// `semantic` = the request opted into the semantic re-rank (which reorders an
/// index hit list, so it keeps the index route).
pub(crate) fn plan(
    conjuncts: &[Expr],
    scope: &Scope,
    sort: &Sort,
    tie_breakers: &[Sort],
    semantic: bool,
) -> Result<Route, &'static str> {
    #[cfg(test)]
    if LEGACY.with(std::cell::Cell::get) {
        return Ok(Route::Index);
    }
    // Index only (R2) when no condition is metadata, the scope included: a
    // folder scope is a SQL condition since slice 2.
    if matches!(scope, Scope::Account) && !conjuncts.iter().any(has_meta) {
        return Ok(Route::Index);
    }
    // A top-level AND (also inside `filter.text`) is a list of conjuncts.
    let mut flat = Vec::with_capacity(conjuncts.len());
    flatten_and(conjuncts, &mut flat);
    let conjuncts = flat.as_slice();
    let mut all = Some(MetaConj::scoped(scope));
    for &e in conjuncts {
        all = all.and_then(|c| c.try_absorb(e));
    }
    if let (Some(c), Some(order), false) = (&all, sql_sort(sort, tie_breakers), semantic)
        && c.meta_leaves > 0
    {
        return Ok(if c.nothing {
            Route::Nothing
        } else {
            Route::Sql(c.clone(), order)
        });
    }
    let mut top = MetaConj::scoped(scope);
    // Whether `top` holds a condition (the folder scope or an absorbed
    // conjunct): then it must be applied, whatever its counters say.
    let mut top_holds = matches!(scope, Scope::Folder(_));
    let mut nodes = Vec::new();
    let mut either_only = Vec::new();
    for &e in conjuncts {
        if has_meta(e) {
            if let Some(t) = top.try_absorb(e) {
                top = t;
                top_holds = true;
                continue;
            }
            nodes.push(rewrite(e)?);
        } else {
            // Content, or mailbox/date/size: the index evaluates it as today.
            nodes.push(Node::Index(e.clone()));
            either_only.push(e);
        }
    }
    let top = if top_holds || top.meta_leaves > 0 {
        // Mailbox/date/size conjuncts narrow the SQL id set too (they also
        // stay in the index query).
        for e in either_only {
            if let Some(t) = top.try_absorb(e) {
                top = t;
            }
        }
        if top.nothing {
            return Ok(Route::Nothing);
        }
        Some(top)
    } else {
        None
    };
    // Nested metadata in a one-folder query: every SQL condition is narrowed
    // to that folder (the whole index expression is ANDed with it, so the
    // answer is the same and the id sets are smaller).
    let mut nodes = nodes;
    if let Scope::Folder(mb) = scope {
        for node in &mut nodes {
            narrow(node, mb);
        }
    }
    Ok(Route::Combined { top, nodes })
}

fn narrow(node: &mut Node, mb: &str) {
    match node {
        Node::Index(_) => {}
        Node::Meta(c) => {
            let own = c.in_mailboxes.is_some();
            c.intersect_in(&[mb.to_owned()]);
            if !own {
                c.in_from_scope = true;
            }
        }
        Node::And(items) | Node::Or(items) => items.iter_mut().for_each(|i| narrow(i, mb)),
        Node::Not(inner) => narrow(inner, mb),
    }
}

fn flatten_and<'a>(items: &'a [Expr], out: &mut Vec<&'a Expr>) {
    for e in items {
        match e {
            Expr::And(inner) => flatten_and(inner, out),
            other => out.push(other),
        }
    }
}

/// Every `Meta` node of `node`, in a fixed (pre-order) walk.
pub(crate) fn meta_nodes<'a>(node: &'a Node, out: &mut Vec<&'a MetaConj>) {
    match node {
        Node::Index(_) => {}
        Node::Meta(c) => out.push(c),
        Node::And(items) | Node::Or(items) => items.iter().for_each(|i| meta_nodes(i, out)),
        Node::Not(inner) => meta_nodes(inner, out),
    }
}

/// `node` as an index expression, with each `Meta` node replaced, in the walk
/// order of [`meta_nodes`], by the next resolved expression.
pub(crate) fn assemble(node: &Node, resolved: &mut std::vec::IntoIter<Expr>) -> Expr {
    match node {
        Node::Index(e) => e.clone(),
        Node::Meta(_) => resolved.next().expect("one resolution per metadata node"),
        Node::And(items) => Expr::And(items.iter().map(|i| assemble(i, resolved)).collect()),
        Node::Or(items) => Expr::Or(items.iter().map(|i| assemble(i, resolved)).collect()),
        Node::Not(inner) => Expr::Not(Box::new(assemble(inner, resolved))),
    }
}

/// `conj` as an index condition from the ids of its complement (see
/// [`MetaConj::complement`]): NOT IdSet(complement), and, when `conj` is
/// restricted to folders, those folders as well. The complement is taken
/// within the folders, so without them the negation would also select every
/// message of every other folder (t49pg-e5b F1).
pub(crate) fn complement_expr(conj: &MetaConj, complement_ids: Vec<String>) -> Expr {
    // Only a subtree without a folder of its own has a complement (slice 2):
    // no folder, or the query's scope folder, which the top-level condition
    // enforces on the same query.
    debug_assert!(conj.in_mailboxes.is_none() || conj.in_from_scope);
    Expr::Not(Box::new(id_set(complement_ids)))
}

impl MetaConj {
    /// Only folder conditions (no keyword, pin, follow-up or size): the
    /// combined route checks the index's hits against it in SQL rather than
    /// reading the folder's ids.
    pub(crate) fn is_folder_only(&self) -> bool {
        self.keywords.is_empty()
            && self.not_keywords.is_empty()
            && self.pinned.is_none()
            && !self.follow_up
            && self.follow_up_active.is_none()
            && self.larger.is_none()
            && self.smaller.is_none()
            && (self.in_mailboxes.is_some() || !self.not_in.is_empty())
    }
}

/// An id set as an index condition.
pub(crate) fn id_set(ids: Vec<String>) -> Expr {
    Expr::Clause(Clause::IdSet(IdSet::new(ids)))
}

/// Why the planner could not answer.
#[derive(Debug)]
pub(crate) enum PlanFail {
    /// An id set and its complement are both above the cap: `unsupportedFilter`.
    TooBroad,
    /// A store or index failure, or the projection is not maintained: serverFail.
    Engine(EngineError),
}

impl From<EngineError> for PlanFail {
    fn from(e: EngineError) -> Self {
        PlanFail::Engine(e)
    }
}
impl From<mw_store::StoreError> for PlanFail {
    fn from(e: mw_store::StoreError) -> Self {
        PlanFail::Engine(EngineError::Store(e))
    }
}

/// How long a positive `message_projection_installed` answer is reused per
/// store. A negative answer is never cached.
const PROJECTION_CHECK_TTL: Duration = Duration::from_secs(5);

fn projection_checks() -> &'static Mutex<std::collections::HashMap<u128, Instant>> {
    static CHECKS: OnceLock<Mutex<std::collections::HashMap<u128, Instant>>> = OnceLock::new();
    CHECKS.get_or_init(Default::default)
}

#[cfg(test)]
thread_local! {
    /// Run today's index-only path for every query (the differential test's
    /// "old path"; design part 11 keeps it until slice 2 ends).
    pub(crate) static LEGACY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Ask the store on every query instead of reusing a positive answer.
    pub(crate) static UNCACHED_PROJECTION_CHECK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// A smaller id-set cap, so the filter strategy, the complement and
    /// `unsupportedFilter` run on a small corpus.
    pub(crate) static CAP: std::cell::Cell<usize> = const { std::cell::Cell::new(COMBINE_CAP) };
}

/// The id-set cap of the combined route ([`COMBINE_CAP`]; tests may lower it).
pub(crate) fn combine_cap() -> usize {
    #[cfg(test)]
    {
        CAP.with(std::cell::Cell::get)
    }
    #[cfg(not(test))]
    {
        COMBINE_CAP
    }
}

/// Whether this thread runs today's index-only path (the differential test's
/// old path; always `false` outside tests).
pub(crate) fn legacy() -> bool {
    #[cfg(test)]
    {
        LEGACY.with(std::cell::Cell::get)
    }
    #[cfg(not(test))]
    {
        false
    }
}

/// Process-wide counters of the routes taken (read-only diagnostics).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryPlanStatus {
    pub sql: u64,
    pub index: u64,
    pub inject: u64,
    pub filter: u64,
    pub nothing: u64,
    pub too_broad: u64,
}

static COUNTS: [std::sync::atomic::AtomicU64; 6] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 6];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Taken {
    Sql = 0,
    Index = 1,
    Inject = 2,
    Filter = 3,
    Nothing = 4,
    TooBroad = 5,
}

#[cfg(test)]
thread_local! {
    /// The strategies taken on this thread, most recent last (tests).
    pub(crate) static TAKEN: std::cell::RefCell<Vec<Taken>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub(crate) fn taken(t: Taken) {
    COUNTS[t as usize].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    TAKEN.with(|v| v.borrow_mut().push(t));
}

impl crate::Engine {
    /// How often each planner route was taken. Process-wide, monotonic.
    pub fn query_plan_status(&self) -> QueryPlanStatus {
        let c = |t: Taken| COUNTS[t as usize].load(std::sync::atomic::Ordering::Relaxed);
        QueryPlanStatus {
            sql: c(Taken::Sql),
            index: c(Taken::Index),
            inject: c(Taken::Inject),
            filter: c(Taken::Filter),
            nothing: c(Taken::Nothing),
            too_broad: c(Taken::TooBroad),
        }
    }

    /// Refuse unless the 0125 projection triggers are installed and enabled:
    /// a metadata route never answers from tables nobody maintains, and never
    /// falls back to the index.
    pub(crate) async fn projection_ready(&self) -> Result<(), PlanFail> {
        let id = self.store().instance_id().as_u128();
        #[cfg(test)]
        let cached_ok = !UNCACHED_PROJECTION_CHECK.with(std::cell::Cell::get);
        #[cfg(not(test))]
        let cached_ok = true;
        if cached_ok
            && projection_checks()
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .get(&id)
                .is_some_and(|at| at.elapsed() < PROJECTION_CHECK_TTL)
        {
            return Ok(());
        }
        if self.store().message_projection_installed().await? {
            projection_checks()
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .insert(id, Instant::now());
            Ok(())
        } else {
            projection_checks()
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(&id);
            Err(PlanFail::Engine(EngineError::Protocol(
                "message metadata projection is not maintained (0125 triggers missing or disabled)"
                    .into(),
            )))
        }
    }

    /// The ids of `conj` in the account, or `None` when there are more than
    /// `limit`. One statement (LIMIT limit+1); the rows read are counted.
    pub(crate) async fn meta_ids_within(
        &self,
        account_id: &str,
        conj: &MetaConj,
        limit: usize,
    ) -> Result<Option<Vec<String>>, PlanFail> {
        if conj.nothing {
            return Ok(Some(Vec::new()));
        }
        let q = conj.projection(account_id, ProjectionSort::ReceivedDesc);
        let ids = self.store().projection_ids(&q, limit as i64 + 1, 0).await?;
        note_materialised(ids.len());
        Ok((ids.len() <= limit).then_some(ids))
    }

    /// A nested metadata condition as an index condition: its id set, or the
    /// negation of its complement's, whichever fits the query's remaining id
    /// `budget` (shared by every set of the query, t49pg-e5b F2), which it
    /// spends.
    pub(crate) async fn resolve_meta(
        &self,
        account_id: &str,
        conj: &MetaConj,
        budget: &mut usize,
    ) -> Result<Expr, PlanFail> {
        if let Some(ids) = self.meta_ids_within(account_id, conj, *budget).await? {
            *budget -= ids.len();
            return Ok(id_set(ids));
        }
        if let Some(complement) = conj.complement()
            && let Some(ids) = self.meta_ids_within(account_id, &complement, *budget).await?
        {
            *budget -= ids.len();
            return Ok(complement_expr(conj, ids));
        }
        taken(Taken::TooBroad);
        Err(PlanFail::TooBroad)
    }
}

#[cfg(test)]
#[path = "query_plan_tests.rs"]
mod tests;
