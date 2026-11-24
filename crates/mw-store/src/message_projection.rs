//! Message metadata projection (t50hp-e30; design t50hp-e29 parts 4, 5, 7).
//!
//! `message_state` (one row per message: owner, folder, date, uid, size, seen,
//! flagged) and `message_keyword` (one row per JMAP keyword) are derived from
//! `messages` by the 0125 triggers, in the writing statement's transaction. No
//! Rust writer maintains them, so no writer can forget them: every statement on
//! `messages` (ingest, flag writes, moves, deletes, FK cascades, raw SQL, other
//! processes) keeps them current. The only Rust code that writes them is the
//! erasure registry (`data_map.rs`, by account after the sources); the guard test
//! in `message_projection_tests.rs` fails on any other.
//!
//! This module holds the two halves the planner builds on:
//! * [`ProjectionQuery`] with [`Store::projection_ids`], [`Store::projection_count`]
//!   and [`Store::projection_filter_ids`]: metadata queries answered from the
//!   tables, always scoped by account and by folder ownership. Partial indexes are
//!   only usable with a literal predicate, so seen/flagged are emitted as `seen=0`
//!   style literals (they are part of the statement shape), never as parameters.
//! * [`Store::audit_message_projection`]: recomputes both tables in Rust from
//!   `messages` and reports every difference.
//!
//! Derivation (the SQL in 0125 must equal [`projected_seen`] / [`projected_keywords`]):
//! * seen: NOT [`crate::cache::flags_json_is_unread`], the rule of the 0122 folder
//!   counters, so `COUNT(seen=0)` in a folder equals `mailboxes.unread`.
//! * keywords: what mw-engine shows as JMAP keywords for the value,
//!   `flags_to_keywords(flags_from_json(..))`. `flags_from_json` is serde's
//!   `Vec<Flag>` or nothing, so one element that is not a `Flag` leaves NO keyword.
//!   The two rules differ only on values mw-engine never writes (for example
//!   `["Seen", 5]`: seen, but no keyword).
//! * flagged: the keywords hold `$flagged`.
//!
//! Known differences between the SQL and Rust (values mw-engine never writes):
//! a number outside f64 (`1e400`) or nesting deeper than 127 levels inside an
//! element (serde refuses the whole value, SQL reads it); on PostgreSQL a string
//! holding the escape `\u0000` (text cannot hold it: the value counts as empty).
use crate::backend::{Arg, Dialect};
use crate::{Store, StoreError, q};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Mutex, OnceLock};

/// Mirror of `mw_engine::backend::Flag`'s serde form (mw-store cannot depend on
/// mw-engine). A guard test compares it with the engine's source.
#[derive(serde::Deserialize)]
enum ProjectedFlag {
    Seen,
    Answered,
    Flagged,
    Deleted,
    Draft,
    Recent,
    Keyword(String),
}

/// `seen` of a `flags_json` value, as the 0125 triggers derive it.
pub fn projected_seen(flags_json: &str) -> bool {
    !crate::cache::flags_json_is_unread(flags_json)
}

/// The keyword rows of a `flags_json` value, as the 0125 triggers derive them
/// (= `mw_engine::mapping::flags_to_keywords(&flags_from_json(..))`).
pub fn projected_keywords(flags_json: &str) -> BTreeSet<String> {
    serde_json::from_str::<Vec<ProjectedFlag>>(flags_json)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|flag| match flag {
            ProjectedFlag::Seen => Some("$seen".to_owned()),
            ProjectedFlag::Answered => Some("$answered".to_owned()),
            ProjectedFlag::Flagged => Some("$flagged".to_owned()),
            ProjectedFlag::Draft => Some("$draft".to_owned()),
            ProjectedFlag::Keyword(k) => Some(k),
            ProjectedFlag::Deleted | ProjectedFlag::Recent => None,
        })
        .collect()
}

/// Which folders a [`ProjectionQuery`] covers. Every form is limited to the
/// account; a folder id of another account matches nothing.
#[derive(Clone, Copy, Debug)]
pub enum MailboxScope<'a> {
    /// Every folder of the account.
    Account,
    /// Messages in any of these folders (inMailbox / inMailboxes). Empty matches nothing.
    In(&'a [String]),
    /// Messages in none of these folders (inMailboxOtherThan). Empty = [`Self::Account`].
    NotIn(&'a [String]),
}

/// Result order. `ReceivedDesc` is the folder-list order (cache.rs
/// `list_message_ids`): internaldate DESC NULLS LAST, uid DESC, stable_id. Every
/// `*Asc` order is the exact reverse of its `*Desc`, so one index serves both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionSort {
    ReceivedDesc,
    ReceivedAsc,
    SizeDesc,
    SizeAsc,
}

/// A metadata query: the AND of every condition that is set.
///
/// `$seen` / `$flagged` in `keywords` / `not_keywords` are answered from the
/// `seen` / `flagged` columns (seen is the folder-counter rule, see the module
/// note). Dates are epoch seconds against `messages.internaldate` (UTC
/// `YYYY-MM-DDTHH:MM:SSZ`, the form cache.rs normalises to); a message without a
/// date counts as epoch 0, as the search index stores it. Size bounds are strict.
#[derive(Clone, Debug)]
pub struct ProjectionQuery<'a> {
    pub account_id: &'a str,
    pub mailboxes: MailboxScope<'a>,
    pub seen: Option<bool>,
    pub flagged: Option<bool>,
    /// All of these keywords (hasKeyword).
    pub keywords: &'a [String],
    /// None of these keywords (notKeyword).
    pub not_keywords: &'a [String],
    /// message_meta.pinned set (true) or not set (false).
    pub pinned: Option<bool>,
    /// The search index's FollowUp: (follow_up_at set OR $flagged OR
    /// `$mw-followup-active`) AND NOT `mw-followup-completed`.
    pub follow_up: bool,
    /// t50hp-e31: the search index's `$mw-followup-active` keyword: a follow-up
    /// time is set in message_meta, or the message carries a keyword of that
    /// name. `Some(true)` = it holds, `Some(false)` = it does not.
    pub follow_up_active: Option<bool>,
    /// internaldate >= this (inclusive).
    pub received_after: Option<i64>,
    /// internaldate < this (exclusive).
    pub received_before: Option<i64>,
    /// size > this.
    pub size_larger: Option<u64>,
    /// size < this.
    pub size_smaller: Option<u64>,
    /// Only these ids (checking an id set from the search index). Empty matches nothing.
    pub within: Option<&'a [String]>,
    pub sort: ProjectionSort,
}

impl<'a> ProjectionQuery<'a> {
    /// Every message of the account, in folder-list order.
    pub fn account(account_id: &'a str) -> Self {
        Self {
            account_id,
            mailboxes: MailboxScope::Account,
            seen: None,
            flagged: None,
            keywords: &[],
            not_keywords: &[],
            pinned: None,
            follow_up: false,
            follow_up_active: None,
            received_after: None,
            received_before: None,
            size_larger: None,
            size_smaller: None,
            within: None,
            sort: ProjectionSort::ReceivedDesc,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectionOp {
    Page,
    Count,
}

/// Bits that change the statement text. Everything else is a parameter.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Shape {
    pg: bool,
    op: u8,
    scope: u8,
    seen: u8,
    flagged: u8,
    keywords: u8,
    not_keywords: bool,
    within: bool,
    sort: u8,
}

/// The query after `$seen`/`$flagged` folding and de-duplication; `None` when
/// it can match nothing.
struct Normal<'a> {
    query: &'a ProjectionQuery<'a>,
    seen: Option<bool>,
    flagged: Option<bool>,
    keywords: Vec<&'a str>,
    not_keywords: Vec<&'a str>,
    mailboxes: Vec<&'a str>,
}

fn tri(value: Option<bool>) -> u8 {
    match value {
        None => 0,
        Some(false) => 1,
        Some(true) => 2,
    }
}

fn fold(slot: &mut Option<bool>, value: bool) -> bool {
    match *slot {
        Some(old) if old != value => false,
        _ => {
            *slot = Some(value);
            true
        }
    }
}

fn normalise<'a>(query: &'a ProjectionQuery<'a>) -> Option<Normal<'a>> {
    let mut seen = query.seen;
    let mut flagged = query.flagged;
    let mut keywords = BTreeSet::new();
    let mut not_keywords = BTreeSet::new();
    for k in query.keywords {
        match k.as_str() {
            "$seen" => fold(&mut seen, true).then_some(())?,
            "$flagged" => fold(&mut flagged, true).then_some(())?,
            other => {
                keywords.insert(other);
            }
        }
    }
    for k in query.not_keywords {
        match k.as_str() {
            "$seen" => fold(&mut seen, false).then_some(())?,
            "$flagged" => fold(&mut flagged, false).then_some(())?,
            other => {
                not_keywords.insert(other);
            }
        }
    }
    if !keywords.is_disjoint(&not_keywords) {
        return None;
    }
    let mailboxes: Vec<&str> = match query.mailboxes {
        MailboxScope::Account => Vec::new(),
        MailboxScope::In(ids) | MailboxScope::NotIn(ids) => ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    };
    if matches!(query.mailboxes, MailboxScope::In(_)) && mailboxes.is_empty() {
        return None;
    }
    if query.within.is_some_and(<[String]>::is_empty) {
        return None;
    }
    if query.size_smaller == Some(0) {
        return None;
    }
    Some(Normal {
        query,
        seen,
        flagged,
        keywords: keywords.into_iter().collect(),
        not_keywords: not_keywords.into_iter().collect(),
        mailboxes,
    })
}

fn rfc3339(seconds: i64) -> Result<String, StoreError> {
    chrono::DateTime::from_timestamp(seconds, 0)
        .map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .ok_or(StoreError::Invalid("received date bound out of range"))
}

fn json_list(values: &[&str]) -> Arg {
    Arg::Text(serde_json::to_string(values).unwrap_or_else(|_| "[]".to_owned()))
}

/// Builds the statement text and its parameters together so the two cannot
/// drift; the text depends only on the [`Shape`].
struct Builder {
    pg: bool,
    sql: String,
    args: Vec<Arg>,
}

impl Builder {
    fn arg(&mut self, value: Arg) -> String {
        self.args.push(value);
        format!("?{}", self.args.len())
    }
    fn list(&self, placeholder: &str) -> String {
        if self.pg {
            format!("(SELECT value FROM json_array_elements_text({placeholder}::json))")
        } else {
            format!("(SELECT value FROM json_each({placeholder}))")
        }
    }
}

fn build(
    n: &Normal<'_>,
    op: ProjectionOp,
    pg: bool,
) -> Result<(Shape, String, Vec<Arg>), StoreError> {
    let query = n.query;
    let scope = match query.mailboxes {
        MailboxScope::Account => 0,
        MailboxScope::In(_) if n.mailboxes.len() == 1 => 1,
        MailboxScope::In(_) => 2,
        MailboxScope::NotIn(_) if n.mailboxes.is_empty() => 0,
        MailboxScope::NotIn(_) => 3,
    };
    let shape = Shape {
        pg,
        op: op as u8,
        scope,
        seen: tri(n.seen),
        flagged: tri(n.flagged),
        keywords: match n.keywords.len() {
            0 => 0,
            1 => 1,
            _ => 2,
        },
        not_keywords: !n.not_keywords.is_empty(),
        within: query.within.is_some(),
        sort: query.sort as u8,
    };
    let received = matches!(
        query.sort,
        ProjectionSort::ReceivedDesc | ProjectionSort::ReceivedAsc
    );
    // A single-folder date walk without a seen/flagged/keyword driver reads the
    // folder-list index of `messages` (idx_messages_mailbox_page) and joins the
    // state row by key; every other shape starts from the projection's indexes.
    let via_messages =
        scope == 1 && shape.seen == 0 && shape.flagged == 0 && shape.keywords == 0 && received;
    let order_table = if via_messages { "m" } else { "s" };
    let mut b = Builder {
        pg,
        sql: String::with_capacity(1536),
        args: Vec::new(),
    };
    let account = b.arg(Arg::Text(query.account_id.to_owned()));
    b.sql.push_str(match op {
        ProjectionOp::Page => "SELECT s.stable_id FROM ",
        ProjectionOp::Count => "SELECT CAST(COUNT(*) AS BIGINT) AS n FROM ",
    });
    b.sql.push_str(if via_messages {
        "messages m JOIN message_state s ON s.stable_id=m.stable_id"
    } else {
        "message_state s"
    });
    b.sql.push_str(" WHERE ");
    match scope {
        1 => {
            // One folder: the account scope is the folder's ownership (one
            // mailboxes probe, evaluated once), as for the folder list. A second
            // `s.account_id=` term made both planners walk the account-wide index
            // with the folder as a filter; written as a non-indexable `||''` it
            // wrecked PostgreSQL's row estimate (0.5 %) and brought back a sort.
            // The row's own owner is the message's (trigger), and an owned folder
            // holds only its account's messages.
            let mailbox = b.arg(Arg::Text(n.mailboxes[0].to_owned()));
            let t = if via_messages { "m" } else { "s" };
            b.sql.push_str(&format!(
                "{t}.mailbox_id={mailbox} AND EXISTS(SELECT 1 FROM mailboxes b WHERE b.id={mailbox} AND b.account_id={account})"
            ));
        }
        2 => {
            let ids = b.arg(json_list(&n.mailboxes));
            let list = b.list(&ids);
            b.sql.push_str(&format!(
                "s.account_id={account} AND s.mailbox_id IN (SELECT b.id FROM mailboxes b WHERE b.account_id={account} AND b.id IN {list})"
            ));
        }
        3 => {
            let ids = b.arg(json_list(&n.mailboxes));
            let list = b.list(&ids);
            b.sql.push_str(&format!(
                "s.account_id={account} AND s.mailbox_id NOT IN {list}"
            ));
        }
        _ => b.sql.push_str(&format!("s.account_id={account}")),
    }
    // Literals, never parameters: the partial indexes need them.
    match n.seen {
        Some(false) => b.sql.push_str(" AND s.seen=0"),
        Some(true) => b.sql.push_str(" AND s.seen=1"),
        None => {}
    }
    match n.flagged {
        Some(true) => b.sql.push_str(" AND s.flagged=1"),
        Some(false) => b.sql.push_str(" AND s.flagged=0"),
        None => {}
    }
    match n.keywords.as_slice() {
        [] => {}
        [one] => {
            let k = b.arg(Arg::Text((*one).to_owned()));
            b.sql.push_str(&format!(
                " AND s.stable_id IN (SELECT k.stable_id FROM message_keyword k WHERE k.account_id={account} AND k.keyword={k})"
            ));
        }
        many => {
            let ks = b.arg(json_list(many));
            let list = b.list(&ks);
            let count = b.arg(Arg::Int(many.len() as i64));
            b.sql.push_str(&format!(
                " AND s.stable_id IN (SELECT k.stable_id FROM message_keyword k WHERE k.account_id={account} AND k.keyword IN {list} GROUP BY k.stable_id HAVING COUNT(*)={count})"
            ));
        }
    }
    if shape.not_keywords {
        let ks = b.arg(json_list(&n.not_keywords));
        let list = b.list(&ks);
        b.sql.push_str(&format!(
            " AND NOT EXISTS(SELECT 1 FROM message_keyword nk WHERE nk.stable_id=s.stable_id AND nk.keyword IN {list})"
        ));
    }
    let pinned = b.arg(Arg::OptInt(query.pinned.map(i64::from)));
    b.sql.push_str(&format!(
        " AND ({pinned} IS NULL OR (CASE WHEN EXISTS(SELECT 1 FROM message_meta mm WHERE mm.stable_id=s.stable_id AND mm.pinned<>0) THEN 1 ELSE 0 END)={pinned})"
    ));
    let follow = b.arg(Arg::Int(i64::from(query.follow_up)));
    b.sql.push_str(&format!(
        " AND ({follow}=0 OR ((s.flagged=1 OR EXISTS(SELECT 1 FROM message_meta mf WHERE mf.stable_id=s.stable_id AND mf.follow_up_at IS NOT NULL) OR EXISTS(SELECT 1 FROM message_keyword fa WHERE fa.stable_id=s.stable_id AND fa.keyword='$mw-followup-active')) AND NOT EXISTS(SELECT 1 FROM message_keyword fc WHERE fc.stable_id=s.stable_id AND fc.keyword='mw-followup-completed')))"
    ));
    let active = b.arg(Arg::OptInt(query.follow_up_active.map(i64::from)));
    b.sql.push_str(&format!(
        " AND ({active} IS NULL OR (CASE WHEN EXISTS(SELECT 1 FROM message_meta ma WHERE ma.stable_id=s.stable_id AND ma.follow_up_at IS NOT NULL) OR EXISTS(SELECT 1 FROM message_keyword ka WHERE ka.stable_id=s.stable_id AND ka.keyword='$mw-followup-active') THEN 1 ELSE 0 END)={active})"
    ));
    let undated =
        query.received_after.is_none_or(|a| a <= 0) && query.received_before.is_none_or(|b| b > 0);
    let undated = b.arg(Arg::Int(i64::from(undated)));
    let after = b.arg(Arg::OptText(query.received_after.map(rfc3339).transpose()?));
    let before = b.arg(Arg::OptText(
        query.received_before.map(rfc3339).transpose()?,
    ));
    b.sql.push_str(&format!(
        " AND (CASE WHEN s.internaldate IS NULL THEN {undated}=1 ELSE ({after} IS NULL OR s.internaldate>={after}) AND ({before} IS NULL OR s.internaldate<{before}) END)"
    ));
    let clamp = |v: u64| i64::try_from(v).unwrap_or(i64::MAX);
    let larger = b.arg(Arg::OptInt(query.size_larger.map(clamp)));
    let smaller = b.arg(Arg::OptInt(query.size_smaller.map(clamp)));
    b.sql.push_str(&format!(
        " AND ({larger} IS NULL OR s.size>{larger}) AND ({smaller} IS NULL OR s.size<{smaller})"
    ));
    if let Some(ids) = query.within {
        let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
        let ids = b.arg(json_list(&ids));
        let list = b.list(&ids);
        b.sql.push_str(&format!(" AND s.stable_id IN {list}"));
    }
    if op == ProjectionOp::Page {
        let t = order_table;
        b.sql.push_str(&match query.sort {
            ProjectionSort::ReceivedDesc => {
                format!(" ORDER BY {t}.internaldate DESC NULLS LAST, {t}.uid DESC, {t}.stable_id")
            }
            ProjectionSort::ReceivedAsc => format!(
                " ORDER BY {t}.internaldate ASC NULLS FIRST, {t}.uid ASC, {t}.stable_id DESC"
            ),
            ProjectionSort::SizeDesc => " ORDER BY s.size DESC, s.uid DESC, s.stable_id".to_owned(),
            ProjectionSort::SizeAsc => {
                " ORDER BY s.size ASC, s.uid ASC, s.stable_id DESC".to_owned()
            }
        });
        let limit = b.arg(Arg::Int(0));
        let offset = b.arg(Arg::Int(0));
        b.sql.push_str(&format!(" LIMIT {limit} OFFSET {offset}"));
    }
    Ok((shape, b.sql, b.args))
}

/// One `'static` text per shape. The shape space is finite (a few thousand at
/// most), so this cannot grow without bound.
fn interned(shape: Shape, sql: String) -> &'static str {
    static TEXTS: OnceLock<Mutex<HashMap<Shape, &'static str>>> = OnceLock::new();
    let mut texts = TEXTS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    texts
        .entry(shape)
        .or_insert_with(|| Box::leak(sql.into_boxed_str()))
}

/// The statement and parameters for `query`, or `None` when it matches nothing.
/// For a page, the last two parameters are LIMIT and OFFSET (set by the caller).
pub(crate) fn projection_statement(
    query: &ProjectionQuery<'_>,
    op: ProjectionOp,
    dialect: Dialect,
) -> Result<Option<(&'static str, Vec<Arg>)>, StoreError> {
    let Some(n) = normalise(query) else {
        return Ok(None);
    };
    let (shape, sql, args) = build(&n, op, dialect == Dialect::Postgres)?;
    Ok(Some((interned(shape, sql), args)))
}

/// One difference between the projection tables and `messages`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionDrift {
    /// A message without its `message_state` row.
    MissingState { stable_id: String },
    /// A `message_state` row whose message is gone or has another owner.
    OrphanState { stable_id: String },
    /// A `message_state` column that differs from the message.
    StateMismatch {
        stable_id: String,
        column: &'static str,
    },
    /// A keyword the message carries but `message_keyword` lacks.
    MissingKeyword { stable_id: String, keyword: String },
    /// A `message_keyword` row the message does not carry (or with another owner).
    ExtraKeyword { stable_id: String, keyword: String },
}

/// Result of [`Store::audit_message_projection`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectionAudit {
    /// Messages compared.
    pub messages: u64,
    /// Differences found (all of them, even beyond [`Self::sample`]).
    pub drift: u64,
    /// The first differences, at most [`AUDIT_SAMPLE`].
    pub sample: Vec<ProjectionDrift>,
}

/// Differences kept in [`ProjectionAudit::sample`].
pub const AUDIT_SAMPLE: usize = 64;

impl ProjectionAudit {
    pub fn is_clean(&self) -> bool {
        self.drift == 0
    }
    fn note(&mut self, drift: ProjectionDrift) {
        self.drift += 1;
        if self.sample.len() < AUDIT_SAMPLE {
            self.sample.push(drift);
        }
    }
}

const AUDIT_CHUNK: i64 = 500;

impl Store {
    /// Ids matching `query`, in its order, `limit` from `offset`.
    pub async fn projection_ids(
        &self,
        query: &ProjectionQuery<'_>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<String>, StoreError> {
        if limit <= 0 {
            return Ok(Vec::new());
        }
        let Some((sql, mut args)) =
            projection_statement(query, ProjectionOp::Page, self.backend.dialect())?
        else {
            return Ok(Vec::new());
        };
        let n = args.len();
        args[n - 2] = Arg::Int(limit);
        args[n - 1] = Arg::Int(offset.max(0));
        let mut statement = q(sql);
        for arg in args {
            statement = statement.bind(arg);
        }
        Ok(statement.fetch_all_scalar_string(&self.backend).await?)
    }

    /// Number of messages matching `query`.
    pub async fn projection_count(&self, query: &ProjectionQuery<'_>) -> Result<u64, StoreError> {
        let Some((sql, args)) =
            projection_statement(query, ProjectionOp::Count, self.backend.dialect())?
        else {
            return Ok(0);
        };
        let mut statement = q(sql);
        for arg in args {
            statement = statement.bind(arg);
        }
        Ok(u64::try_from(statement.fetch_scalar_i64(&self.backend).await?).unwrap_or(0))
    }

    /// The members of `ids` that match `query`, in the query's order. For an id
    /// set from the search index; the caller chunks large sets.
    pub async fn projection_filter_ids(
        &self,
        query: &ProjectionQuery<'_>,
        ids: &[String],
    ) -> Result<Vec<String>, StoreError> {
        let mut query = query.clone();
        query.within = Some(ids);
        self.projection_ids(&query, i64::MAX, 0).await
    }

    /// Whether the three 0125 triggers are installed (and, on PostgreSQL,
    /// enabled in origin mode, tgenabled='O'). A metadata route must refuse
    /// rather than answer from tables nobody maintains.
    pub async fn message_projection_installed(&self) -> Result<bool, StoreError> {
        let n = match self.backend.dialect() {
            Dialect::Sqlite => q("SELECT CAST(COUNT(*) AS BIGINT) FROM sqlite_master WHERE type='trigger' AND tbl_name='messages'
                     AND name IN ('message_projection_insert','message_projection_update','message_projection_delete')"),
            Dialect::Postgres => q("SELECT CAST(COUNT(*) AS BIGINT) FROM pg_catalog.pg_trigger t
                     JOIN pg_catalog.pg_class c ON c.oid = t.tgrelid JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                     WHERE n.nspname = current_schema() AND c.relname = 'messages' AND t.tgenabled = 'O' AND NOT t.tgisinternal
                     AND t.tgname IN ('message_projection_insert','message_projection_update','message_projection_delete')"),
        }
        .fetch_scalar_i64(&self.backend)
        .await?;
        Ok(n == 3)
    }

    /// Recompute the projection of one account (or of every account) in Rust
    /// from `messages` and compare it with both tables. Reads in chunks of 500
    /// messages; for tests after every writer and for an admin diagnostic.
    pub async fn audit_message_projection(
        &self,
        account_id: Option<&str>,
    ) -> Result<ProjectionAudit, StoreError> {
        let mut audit = ProjectionAudit::default();
        let account = account_id.map(str::to_owned);
        let mut after = String::new();
        let keyword_sql = match self.backend.dialect() {
            Dialect::Sqlite => {
                "SELECT stable_id, account_id, keyword FROM message_keyword WHERE stable_id IN (SELECT value FROM json_each(?1))"
            }
            Dialect::Postgres => {
                "SELECT stable_id, account_id, keyword FROM message_keyword WHERE stable_id IN (SELECT value FROM json_array_elements_text(?1::json))"
            }
        };
        loop {
            let rows = q("SELECT m.stable_id, m.account_id, m.mailbox_id, m.internaldate, m.uid, m.size, m.flags_json,
                    CAST(CASE WHEN s.stable_id IS NULL THEN 0 ELSE 1 END AS BIGINT) AS has_state,
                    s.account_id AS s_account, s.mailbox_id AS s_mailbox, s.internaldate AS s_date,
                    s.uid AS s_uid, s.size AS s_size, s.seen AS s_seen, s.flagged AS s_flagged
                 FROM messages m LEFT JOIN message_state s ON s.stable_id = m.stable_id
                 WHERE (?1 IS NULL OR m.account_id = ?1) AND m.stable_id > ?2
                 ORDER BY m.stable_id LIMIT ?3")
            .bind(account.clone())
            .bind(after.clone())
            .bind(AUDIT_CHUNK)
            .fetch_all(&self.backend)
            .await?;
            let Some(last) = rows.last() else { break };
            after = last.get_string("stable_id");
            let ids: Vec<String> = rows.iter().map(|r| r.get_string("stable_id")).collect();
            let mut stored: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
            for r in q(keyword_sql)
                .bind(serde_json::to_string(&ids).unwrap_or_default())
                .fetch_all(&self.backend)
                .await?
            {
                stored
                    .entry(r.get_string("stable_id"))
                    .or_default()
                    .push((r.get_string("keyword"), r.get_string("account_id")));
            }
            for r in &rows {
                audit.messages += 1;
                let id = r.get_string("stable_id");
                let owner = r.get_string("account_id");
                let flags = r.get_string("flags_json");
                let keywords = projected_keywords(&flags);
                if r.get_i64("has_state") == 0 {
                    audit.note(ProjectionDrift::MissingState {
                        stable_id: id.clone(),
                    });
                } else {
                    let checks: [(&'static str, bool); 7] = [
                        (
                            "account_id",
                            r.get_opt_string("s_account").as_deref() == Some(owner.as_str()),
                        ),
                        (
                            "mailbox_id",
                            r.get_opt_string("s_mailbox") == r.get_opt_string("mailbox_id"),
                        ),
                        (
                            "internaldate",
                            r.get_opt_string("s_date") == r.get_opt_string("internaldate"),
                        ),
                        ("uid", r.get_opt_i64("s_uid") == Some(r.get_i64("uid"))),
                        ("size", r.get_opt_i64("s_size") == Some(r.get_i64("size"))),
                        (
                            "seen",
                            r.get_opt_i64("s_seen") == Some(i64::from(projected_seen(&flags))),
                        ),
                        (
                            "flagged",
                            r.get_opt_i64("s_flagged")
                                == Some(i64::from(keywords.contains("$flagged"))),
                        ),
                    ];
                    for (column, ok) in checks {
                        if !ok {
                            audit.note(ProjectionDrift::StateMismatch {
                                stable_id: id.clone(),
                                column,
                            });
                        }
                    }
                }
                let have = stored.remove(&id).unwrap_or_default();
                let mut present = BTreeSet::new();
                for (keyword, row_owner) in have {
                    if row_owner == owner && keywords.contains(&keyword) {
                        present.insert(keyword);
                    } else {
                        audit.note(ProjectionDrift::ExtraKeyword {
                            stable_id: id.clone(),
                            keyword,
                        });
                    }
                }
                for keyword in keywords.difference(&present) {
                    audit.note(ProjectionDrift::MissingKeyword {
                        stable_id: id.clone(),
                        keyword: keyword.clone(),
                    });
                }
            }
            if (rows.len() as i64) < AUDIT_CHUNK {
                break;
            }
        }
        // Rows whose message is gone or belongs to another account.
        for r in q("SELECT s.stable_id FROM message_state s WHERE (?1 IS NULL OR s.account_id = ?1)
                 AND NOT EXISTS(SELECT 1 FROM messages m WHERE m.stable_id = s.stable_id AND m.account_id = s.account_id)
                 ORDER BY s.stable_id")
            .bind(account.clone())
            .fetch_all(&self.backend)
            .await?
        {
            audit.note(ProjectionDrift::OrphanState { stable_id: r.get_string("stable_id") });
        }
        for r in q("SELECT k.stable_id, k.keyword FROM message_keyword k WHERE (?1 IS NULL OR k.account_id = ?1)
                 AND NOT EXISTS(SELECT 1 FROM messages m WHERE m.stable_id = k.stable_id)
                 ORDER BY k.stable_id, k.keyword")
            .bind(account)
            .fetch_all(&self.backend)
            .await?
        {
            audit.note(ProjectionDrift::ExtraKeyword {
                stable_id: r.get_string("stable_id"),
                keyword: r.get_string("keyword"),
            });
        }
        Ok(audit)
    }
}

#[cfg(test)]
#[path = "message_projection_tests.rs"]
mod tests;
