//! User retention rules: the model, validation against the administrator's
//! limits, and the pure evaluation that decides what happens to one message.
//!
//! Nothing here touches the network or the database. [`plan_message`] takes the
//! message's facts, the account's folders and rules and the effective policy, and
//! answers with one [`Outcome`]. The dry run and the real run both call it, so
//! the preview and the run cannot disagree.
//!
//! Order of decisions for a message, first match wins:
//!
//! 1. A message in Trash that a retention rule put there carries the keyword
//!    `mw-ret-<day>` (days since the epoch). It is purged `undoDays` after that
//!    day. A Trash message without the keyword is left to the mandatory rule.
//! 2. Mandatory organisation rules (`retention.user.mandatory`), by folder role.
//!    They ignore the user's exclusions and pause switch.
//! 3. Protected folders (`retention.user.protectedFolders`) are never touched
//!    by anything below.
//! 4. The user's rules in order, then folder defaults from the most specific
//!    folder up the tree. A rule whose scope matches a message that is not yet
//!    old enough lets the next rule look at it. A `keep` rule, and an exclusion
//!    (flagged, unread, a tag such as `keep`), end the search with no change.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DAY: i64 = 86_400;
/// Prefix of the keyword that records the day a rule moved a message to Trash.
pub const TRASH_STAMP: &str = "mw-ret-";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ReadState {
    #[default]
    Any,
    Read,
    Unread,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionKind {
    /// Keep the matching mail for ever; later rules do not see it.
    Keep,
    /// Delete after the days: to Trash first unless permanent deletion is on.
    Delete,
    /// Move to the Archive folder.
    Archive,
    /// Move to the folder named by `target`.
    Move,
    /// Move to Trash; the undo window then purges it.
    Trash,
    /// Mark as read.
    MarkRead,
    /// Remove attachments. Not available yet; rules naming it are refused.
    StripAttachments,
}

impl ActionKind {
    pub fn destructive(self) -> bool {
        matches!(self, ActionKind::Delete | ActionKind::Trash)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scope {
    /// Folder ids, or `role:<role>` (trash, junk, inbox, archive, sent).
    /// Empty means every folder that is not protected.
    #[serde(default)]
    pub folders: Vec<String>,
    /// Whether the folders' sub-folders are included.
    #[serde(default = "yes")]
    pub subfolders: bool,
    /// Match a message that carries any of these labels (keywords).
    #[serde(default)]
    pub labels: Vec<String>,
    /// Sender addresses.
    #[serde(default)]
    pub senders: Vec<String>,
    /// Sender domains, without the `@`.
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub min_size_kb: Option<i64>,
    #[serde(default)]
    pub max_size_kb: Option<i64>,
    #[serde(default)]
    pub read: ReadState,
    #[serde(default)]
    pub has_attachment: Option<bool>,
}

fn yes() -> bool {
    true
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            folders: Vec::new(),
            subfolders: true,
            labels: Vec::new(),
            senders: Vec::new(),
            domains: Vec::new(),
            min_size_kb: None,
            max_size_kb: None,
            read: ReadState::Any,
            has_attachment: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Action {
    pub kind: ActionKind,
    /// Message age in days before the action applies.
    #[serde(default)]
    pub days: i64,
    /// Folder id or `role:<role>` for `move`.
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Exclude {
    /// Leave flagged (starred) messages alone.
    #[serde(default = "yes")]
    pub flagged: bool,
    /// Leave unread messages alone.
    #[serde(default)]
    pub unread: bool,
    /// Leave messages carrying any of these labels alone.
    #[serde(default = "keep_tag")]
    pub tags: Vec<String>,
}

fn keep_tag() -> Vec<String> {
    vec!["keep".into()]
}

impl Default for Exclude {
    fn default() -> Self {
        Self { flagged: true, unread: false, tags: keep_tag() }
    }
}

/// One rule as stored in `body_json` (name and enabled live in columns).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Body {
    #[serde(default)]
    pub scope: Scope,
    pub action: Action,
    #[serde(default)]
    pub exclude: Exclude,
}

/// A rule with its identity, ready to evaluate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub body: Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Msg {
    pub id: String,
    pub mailbox_id: String,
    /// Unix seconds.
    pub received_at: i64,
    pub size: i64,
    pub keywords: Vec<String>,
    /// Lower-case sender address.
    pub from: String,
    pub subject: String,
    pub has_attachment: bool,
}

impl Msg {
    pub fn has_keyword(&self, k: &str) -> bool {
        self.keywords.iter().any(|x| x.eq_ignore_ascii_case(k))
    }
    pub fn unread(&self) -> bool {
        !self.has_keyword("$seen")
    }
    pub fn trashed_day(&self) -> Option<i64> {
        self.keywords
            .iter()
            .find_map(|k| k.strip_prefix(TRASH_STAMP).and_then(|d| d.parse().ok()))
    }
}

/// The administrator's limits for one account, already resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub enabled: bool,
    pub min_days: i64,
    /// 0 sets no ceiling.
    pub max_days: i64,
    pub max_rules: i64,
    pub allow_permanent: bool,
    pub undo_days: i64,
    /// `(folder role, days)`.
    pub mandatory: Vec<(String, i64)>,
    pub protected: Vec<String>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            enabled: true,
            min_days: 7,
            max_days: 0,
            max_rules: 50,
            allow_permanent: false,
            undo_days: 7,
            mandatory: Vec::new(),
            protected: vec!["drafts".into()],
        }
    }
}

/// Parse `trash:30` style entries; malformed entries are dropped.
pub fn parse_mandatory(items: &[String]) -> Vec<(String, i64)> {
    items
        .iter()
        .filter_map(|s| {
            let (role, days) = s.split_once(':')?;
            let role = role.trim().to_ascii_lowercase();
            let days: i64 = days.trim().parse().ok()?;
            (!role.is_empty() && (1..=3650).contains(&days)).then_some((role, days))
        })
        .collect()
}

/// The account's own switches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub enabled: bool,
    /// Delete rules move to Trash first (always true when permanent deletion is
    /// not allowed).
    pub trash_first: bool,
    pub paused: bool,
}

impl Default for Account {
    fn default() -> Self {
        Self { enabled: true, trash_first: true, paused: false }
    }
}

/// What happens to one message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing changes; the reason is for the preview.
    Skip(Skip),
    MarkRead,
    /// Move to the folder, optionally recording the Trash day.
    Move { to: String, stamp: Option<String> },
    /// Only record the Trash day (the message is already in Trash).
    Stamp(String),
    Destroy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    NoRule,
    Kept,
    Excluded,
    Protected,
    UndoWindow,
    NoTarget,
}

/// The decision for one message and the rule that made it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub outcome: Outcome,
    /// `rule:<id>`, `folder:<id>`, `mandatory:<role>` or `trash-undo`.
    pub source: String,
}

impl Decision {
    pub fn acts(&self) -> bool {
        !matches!(self.outcome, Outcome::Skip(_))
    }
}

fn skip(why: Skip, source: &str) -> Decision {
    Decision { outcome: Outcome::Skip(why), source: source.into() }
}

/// Folder lookup with the helpers rules need.
pub struct Tree<'a> {
    by_id: HashMap<&'a str, &'a Folder>,
}

impl<'a> Tree<'a> {
    pub fn new(folders: &'a [Folder]) -> Self {
        Self { by_id: folders.iter().map(|f| (f.id.as_str(), f)).collect() }
    }
    pub fn get(&self, id: &str) -> Option<&'a Folder> {
        self.by_id.get(id).copied()
    }
    /// The folder, then each parent up to the root.
    pub fn chain(&self, id: &str) -> Vec<&'a Folder> {
        let mut out = Vec::new();
        let mut cur = self.get(id);
        while let Some(f) = cur {
            if out.len() > 64 || out.iter().any(|x: &&Folder| x.id == f.id) {
                break;
            }
            out.push(f);
            cur = f.parent_id.as_deref().and_then(|p| self.get(p));
        }
        out
    }
    pub fn by_role(&self, role: &str) -> Option<&'a Folder> {
        self.by_id.values().find(|f| f.role.as_deref() == Some(role)).copied()
    }
    /// Resolve `role:x` or a folder id to a folder id.
    pub fn resolve(&self, spec: &str) -> Option<String> {
        match spec.strip_prefix("role:") {
            Some(role) => self.by_role(role).map(|f| f.id.clone()),
            None => self.get(spec).map(|f| f.id.clone()),
        }
    }
    fn is_protected(&self, folder_id: &str, protected: &[String]) -> bool {
        self.chain(folder_id).first().is_some_and(|f| {
            protected.iter().any(|p| {
                let p = p.trim();
                f.role.as_deref().is_some_and(|r| r.eq_ignore_ascii_case(p))
                    || f.name.eq_ignore_ascii_case(p)
                    || f.id == p
            })
        })
    }
}

fn scope_folder_matches(scope: &Scope, tree: &Tree, folder_id: &str) -> bool {
    if scope.folders.is_empty() {
        return true;
    }
    let chain = tree.chain(folder_id);
    scope.folders.iter().any(|spec| {
        let Some(want) = tree.resolve(spec) else { return false };
        if scope.subfolders {
            chain.iter().any(|f| f.id == want)
        } else {
            folder_id == want
        }
    })
}

fn scope_matches(scope: &Scope, tree: &Tree, m: &Msg) -> bool {
    if !scope_folder_matches(scope, tree, &m.mailbox_id) {
        return false;
    }
    if !scope.labels.is_empty() && !scope.labels.iter().any(|l| m.has_keyword(l)) {
        return false;
    }
    let sender_filter = !scope.senders.is_empty() || !scope.domains.is_empty();
    if sender_filter {
        let by_addr = scope.senders.iter().any(|s| s.eq_ignore_ascii_case(&m.from));
        let dom = m.from.rsplit_once('@').map(|x| x.1).unwrap_or("");
        let by_dom = scope
            .domains
            .iter()
            .any(|d| d.trim_start_matches('@').eq_ignore_ascii_case(dom));
        if !by_addr && !by_dom {
            return false;
        }
    }
    let kb = m.size / 1024;
    if scope.min_size_kb.is_some_and(|min| kb < min) || scope.max_size_kb.is_some_and(|max| kb > max) {
        return false;
    }
    match scope.read {
        ReadState::Any => {}
        ReadState::Read if m.unread() => return false,
        ReadState::Unread if !m.unread() => return false,
        _ => {}
    }
    if scope.has_attachment.is_some_and(|w| w != m.has_attachment) {
        return false;
    }
    true
}

fn excluded(ex: &Exclude, m: &Msg) -> bool {
    (ex.flagged && m.has_keyword("$flagged"))
        || (ex.unread && m.unread())
        || ex.tags.iter().any(|t| m.has_keyword(t))
}

/// The days a rule waits before acting, after the administrator's limits.
pub fn effective_days(kind: ActionKind, days: i64, p: &Policy) -> i64 {
    if !kind.destructive() {
        return days.max(0);
    }
    let mut d = days.max(p.min_days);
    if p.max_days > 0 {
        d = d.min(p.max_days.max(p.min_days));
    }
    d
}

/// Decide what happens to `m` at `now`. `folder_defaults` are rules whose scope
/// is a folder; they are tried after `rules`, deepest folder first.
pub fn plan_message(
    m: &Msg,
    now: i64,
    tree: &Tree,
    rules: &[Rule],
    folder_defaults: &[Rule],
    p: &Policy,
    acct: &Account,
) -> Decision {
    let Some(folder) = tree.get(&m.mailbox_id) else {
        return skip(Skip::NoTarget, "none");
    };
    let role = folder.role.as_deref().unwrap_or("");
    let today = now.div_euclid(DAY);

    // 1. The undo window of a message a rule put in Trash.
    if role == "trash" {
        if let Some(day) = m.trashed_day() {
            return if today - day >= p.undo_days {
                Decision { outcome: Outcome::Destroy, source: "trash-undo".into() }
            } else {
                skip(Skip::UndoWindow, "trash-undo")
            };
        }
    }

    // 2. Mandatory organisation rules.
    if let Some((_, days)) = p.mandatory.iter().find(|(r, _)| r == role) {
        if now - m.received_at >= days * DAY {
            return Decision { outcome: Outcome::Destroy, source: format!("mandatory:{role}") };
        }
    }

    // 3. Protected folders.
    if tree.is_protected(&m.mailbox_id, &p.protected) {
        return skip(Skip::Protected, "protected");
    }

    if !p.enabled || !acct.enabled || acct.paused {
        return skip(Skip::NoRule, "none");
    }

    // 4. The user's rules, then folder defaults.
    let age = now - m.received_at;
    let defaults = {
        let chain = tree.chain(&m.mailbox_id);
        let mut v: Vec<&Rule> = Vec::new();
        for f in chain {
            for r in folder_defaults.iter().filter(|r| r.enabled) {
                if r.body.scope.folders.iter().any(|s| tree.resolve(s).as_deref() == Some(f.id.as_str())) {
                    v.push(r);
                }
            }
        }
        v
    };
    let ordered = rules.iter().filter(|r| r.enabled).chain(defaults);
    for r in ordered {
        let b = &r.body;
        let source = if folder_defaults.iter().any(|d| d.id == r.id) {
            format!("folder:{}", r.id)
        } else {
            format!("rule:{}", r.id)
        };
        if !scope_matches(&b.scope, tree, m) {
            continue;
        }
        if b.action.kind == ActionKind::Keep {
            return skip(Skip::Kept, &source);
        }
        if age < effective_days(b.action.kind, b.action.days, p) * DAY {
            continue;
        }
        if excluded(&b.exclude, m) {
            return skip(Skip::Excluded, &source);
        }
        let outcome = match b.action.kind {
            ActionKind::Keep | ActionKind::StripAttachments => continue,
            ActionKind::MarkRead => {
                if m.unread() {
                    Outcome::MarkRead
                } else {
                    continue;
                }
            }
            ActionKind::Archive => match tree.by_role("archive") {
                Some(a) if a.id != m.mailbox_id => Outcome::Move { to: a.id.clone(), stamp: None },
                Some(_) => continue,
                None => return skip(Skip::NoTarget, &source),
            },
            ActionKind::Move => match b.action.target.as_deref().and_then(|t| tree.resolve(t)) {
                Some(to) if to != m.mailbox_id => Outcome::Move { to, stamp: None },
                Some(_) => continue,
                None => return skip(Skip::NoTarget, &source),
            },
            ActionKind::Trash | ActionKind::Delete => {
                let permanent = b.action.kind == ActionKind::Delete
                    && p.allow_permanent
                    && !acct.trash_first;
                if permanent {
                    Outcome::Destroy
                } else {
                    let stamp = format!("{TRASH_STAMP}{today}");
                    match tree.by_role("trash") {
                        Some(t) if t.id == m.mailbox_id => Outcome::Stamp(stamp),
                        Some(t) => Outcome::Move { to: t.id.clone(), stamp: Some(stamp) },
                        None => return skip(Skip::NoTarget, &source),
                    }
                }
            }
        };
        return Decision { outcome, source };
    }
    skip(Skip::NoRule, "none")
}

/// The smallest age in days at which anything could act on a message in
/// `folder_id`, or `None` when nothing can. The job lists only messages older
/// than this, so a quiet folder is never read.
pub fn folder_horizon(
    folder_id: &str,
    tree: &Tree,
    rules: &[Rule],
    folder_defaults: &[Rule],
    p: &Policy,
    acct: &Account,
) -> Option<i64> {
    let folder = tree.get(folder_id)?;
    let role = folder.role.as_deref().unwrap_or("");
    let mut best: Option<i64> = None;
    let mut take = |d: i64| best = Some(best.map_or(d, |b| b.min(d)));
    if role == "trash" {
        take(0);
    }
    if let Some((_, days)) = p.mandatory.iter().find(|(r, _)| r == role) {
        take(*days);
    }
    if !tree.is_protected(folder_id, &p.protected) && p.enabled && acct.enabled && !acct.paused {
        for r in rules.iter().chain(folder_defaults).filter(|r| r.enabled) {
            if scope_folder_matches(&r.body.scope, tree, folder_id) {
                let k = r.body.action.kind;
                take(if k == ActionKind::Keep { 0 } else { effective_days(k, r.body.action.days, p) });
            }
        }
    }
    best
}

// ─── validation ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invalid {
    pub code: &'static str,
    pub detail: String,
}

fn bad(code: &'static str, detail: impl Into<String>) -> Result<(), Invalid> {
    Err(Invalid { code, detail: detail.into() })
}

/// Check a rule against the administrator's limits. `folders` are the account's
/// folders when known; a folder that is not found is only refused when the list
/// is not empty.
pub fn validate(name: &str, b: &Body, p: &Policy, folders: &[Folder]) -> Result<(), Invalid> {
    if name.trim().is_empty() || name.chars().count() > 80 {
        return bad("invalidName", "A rule name has 1 to 80 characters.");
    }
    let a = &b.action;
    if a.kind == ActionKind::StripAttachments {
        return bad("unsupportedAction", "Removing attachments is not available on this server.");
    }
    if a.days < 0 || a.days > 3650 {
        return bad("invalidDays", "Days must be between 0 and 3650.");
    }
    if a.kind.destructive() {
        if a.days < p.min_days {
            return bad("belowMinimum", format!("This server keeps mail at least {} days before a rule may delete it.", p.min_days));
        }
        if p.max_days > 0 && a.days > p.max_days {
            return bad("aboveMaximum", format!("This server allows a delete rule to wait at most {} days.", p.max_days));
        }
    }
    if a.kind == ActionKind::Move && a.target.as_deref().is_none_or(str::is_empty) {
        return bad("targetRequired", "Choose the folder to move mail to.");
    }
    let s = &b.scope;
    if [s.folders.len(), s.labels.len(), s.senders.len(), s.domains.len(), b.exclude.tags.len()]
        .iter()
        .any(|n| *n > 50)
    {
        return bad("tooMany", "A list in a rule holds at most 50 entries.");
    }
    if s.min_size_kb.is_some_and(|n| n < 0) || s.max_size_kb.is_some_and(|n| n < 0) {
        return bad("invalidSize", "Sizes cannot be negative.");
    }
    if let (Some(lo), Some(hi)) = (s.min_size_kb, s.max_size_kb)
        && lo > hi
    {
        return bad("invalidSize", "The smallest size is above the largest.");
    }
    if !folders.is_empty() {
        let tree = Tree::new(folders);
        for f in s.folders.iter().chain(a.target.iter()) {
            if tree.resolve(f).is_none() {
                return bad("unknownFolder", format!("Folder {f} was not found."));
            }
        }
        // A protected folder cannot be named as the place a rule works in.
        for f in &s.folders {
            if let Some(id) = tree.resolve(f)
                && tree.is_protected(&id, &p.protected)
            {
                return bad("folderProtected", "That folder is protected by the administrator.");
            }
        }
    }
    Ok(())
}

// ─── templates ─────────────────────────────────────────────────────────────────

/// One template users can adopt.
pub struct Template {
    pub id: &'static str,
    pub name: &'static str,
    pub body: fn() -> Body,
}

fn labelled(label: &str, kind: ActionKind, days: i64) -> Body {
    Body {
        scope: Scope { labels: vec![label.into()], subfolders: true, ..Scope::default() },
        action: Action { kind, days, target: None },
        exclude: Exclude::default(),
    }
}

fn in_role(role: &str, kind: ActionKind, days: i64, read: ReadState) -> Body {
    Body {
        scope: Scope { folders: vec![format!("role:{role}")], read, subfolders: false, ..Scope::default() },
        action: Action { kind, days, target: None },
        exclude: Exclude::default(),
    }
}

pub const TEMPLATES: &[Template] = &[
    Template { id: "newsletters30", name: "Newsletters: delete after 30 days", body: || labelled("newsletter", ActionKind::Delete, 30) },
    Template { id: "receipts365", name: "Receipts: archive after 1 year", body: || labelled("receipt", ActionKind::Archive, 365) },
    Template { id: "notifications14", name: "Notifications: delete after 14 days", body: || labelled("notification", ActionKind::Delete, 14) },
    Template { id: "trash30", name: "Trash: delete after 30 days", body: || in_role("trash", ActionKind::Delete, 30, ReadState::Any) },
    Template { id: "junk30", name: "Junk: delete after 30 days", body: || in_role("junk", ActionKind::Delete, 30, ReadState::Any) },
    Template { id: "read-archive180", name: "Inbox: archive read mail after 180 days", body: || in_role("inbox", ActionKind::Archive, 180, ReadState::Read) },
];

pub fn template(id: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn folders() -> Vec<Folder> {
        let f = |id: &str, name: &str, role: Option<&str>, parent: Option<&str>| Folder {
            id: id.into(),
            name: name.into(),
            role: role.map(Into::into),
            parent_id: parent.map(Into::into),
        };
        vec![
            f("inbox", "Inbox", Some("inbox"), None),
            f("trash", "Trash", Some("trash"), None),
            f("junk", "Junk", Some("junk"), None),
            f("archive", "Archive", Some("archive"), None),
            f("drafts", "Drafts", Some("drafts"), None),
            f("sent", "Sent", Some("sent"), None),
            f("work", "Work", None, None),
            f("proj", "Project", None, Some("work")),
        ]
    }

    fn msg(id: &str, mailbox: &str, age_days: i64, kw: &[&str]) -> Msg {
        Msg {
            id: id.into(),
            mailbox_id: mailbox.into(),
            received_at: NOW - age_days * DAY,
            size: 10 * 1024,
            keywords: kw.iter().map(|s| s.to_string()).collect(),
            from: "news@shop.example".into(),
            subject: "s".into(),
            has_attachment: false,
        }
    }

    fn rule(id: &str, scope: Scope, kind: ActionKind, days: i64) -> Rule {
        Rule {
            id: id.into(),
            name: id.into(),
            enabled: true,
            body: Body { scope, action: Action { kind, days, target: None }, exclude: Exclude::default() },
        }
    }

    fn run(m: &Msg, rules: &[Rule], defaults: &[Rule], p: &Policy) -> Decision {
        let fs = folders();
        plan_message(m, NOW, &Tree::new(&fs), rules, defaults, p, &Account::default())
    }

    fn label(l: &str) -> Scope {
        Scope { labels: vec![l.into()], subfolders: true, ..Scope::default() }
    }

    #[test]
    fn age_gate_and_trash_first() {
        let p = Policy::default();
        let r = [rule("n", label("newsletter"), ActionKind::Delete, 30)];
        let young = run(&msg("a", "inbox", 10, &["newsletter", "$seen"]), &r, &[], &p);
        assert_eq!(young.outcome, Outcome::Skip(Skip::NoRule));
        let old = run(&msg("b", "inbox", 40, &["newsletter", "$seen"]), &r, &[], &p);
        match old.outcome {
            Outcome::Move { to, stamp } => {
                assert_eq!(to, "trash");
                assert!(stamp.unwrap().starts_with(TRASH_STAMP));
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(old.source, "rule:n");
    }

    #[test]
    fn permanent_delete_needs_policy_and_account_choice() {
        let r = [rule("n", label("x"), ActionKind::Delete, 30)];
        let m = msg("a", "inbox", 40, &["x"]);
        let fs = folders();
        let tree = Tree::new(&fs);
        let mut p = Policy { allow_permanent: true, ..Policy::default() };
        let acct = Account { trash_first: false, ..Account::default() };
        assert_eq!(plan_message(&m, NOW, &tree, &r, &[], &p, &acct).outcome, Outcome::Destroy);
        p.allow_permanent = false;
        assert!(matches!(plan_message(&m, NOW, &tree, &r, &[], &p, &acct).outcome, Outcome::Move { .. }));
        p.allow_permanent = true;
        assert!(matches!(
            plan_message(&m, NOW, &tree, &r, &[], &p, &Account::default()).outcome,
            Outcome::Move { .. }
        ));
    }

    #[test]
    fn floors_and_ceilings_apply_when_a_rule_runs() {
        let p = Policy { min_days: 30, max_days: 60, ..Policy::default() };
        assert_eq!(effective_days(ActionKind::Delete, 1, &p), 30);
        assert_eq!(effective_days(ActionKind::Delete, 900, &p), 60);
        assert_eq!(effective_days(ActionKind::Archive, 1, &p), 1, "archiving is not deleting");
        // A 5-day rule does not fire on a 10-day-old message under a 30-day floor.
        let r = [rule("n", label("x"), ActionKind::Delete, 5)];
        assert_eq!(run(&msg("a", "inbox", 10, &["x"]), &r, &[], &p).outcome, Outcome::Skip(Skip::NoRule));
        assert!(run(&msg("a", "inbox", 31, &["x"]), &r, &[], &p).acts());
    }

    #[test]
    fn exclusions_stop_the_search() {
        let p = Policy::default();
        let r = [
            rule("n", label("x"), ActionKind::Delete, 30),
            rule("all", Scope::default(), ActionKind::Delete, 30),
        ];
        for kw in [&["x", "$flagged"][..], &["x", "keep"][..], &["x"][..]] {
            let d = run(&msg("a", "inbox", 90, kw), &r, &[], &p);
            if kw.contains(&"$flagged") || kw.contains(&"keep") {
                assert_eq!(d.outcome, Outcome::Skip(Skip::Excluded), "{kw:?}");
                assert_eq!(d.source, "rule:n", "the later rule does not get a second look");
            } else {
                // unread is only excluded when the rule asks for it
                assert!(d.acts());
            }
        }
        let mut unread_rule = rule("u", label("x"), ActionKind::Delete, 30);
        unread_rule.body.exclude.unread = true;
        assert_eq!(
            run(&msg("a", "inbox", 90, &["x"]), &[unread_rule.clone()], &[], &p).outcome,
            Outcome::Skip(Skip::Excluded)
        );
        assert!(run(&msg("a", "inbox", 90, &["x", "$seen"]), &[unread_rule], &[], &p).acts());
    }

    #[test]
    fn keep_rule_shields_from_later_rules() {
        let p = Policy::default();
        let r = [
            rule("k", Scope { senders: vec!["boss@x.example".into()], ..Scope::default() }, ActionKind::Keep, 0),
            rule("all", Scope::default(), ActionKind::Delete, 30),
        ];
        let mut m = msg("a", "inbox", 400, &["$seen"]);
        m.from = "boss@x.example".into();
        let d = run(&m, &r, &[], &p);
        assert_eq!((d.outcome, d.source.as_str()), (Outcome::Skip(Skip::Kept), "rule:k"));
        assert!(run(&msg("b", "inbox", 400, &["$seen"]), &r, &[], &p).acts());
    }

    #[test]
    fn scope_matrix() {
        let p = Policy::default();
        let act = |scope: Scope, m: &Msg| run(m, &[rule("r", scope, ActionKind::Archive, 10)], &[], &p).acts();
        let m = msg("a", "proj", 20, &["$seen", "receipt"]);
        // Folder with and without sub-folders.
        assert!(act(Scope { folders: vec!["work".into()], subfolders: true, ..Scope::default() }, &m));
        assert!(!act(Scope { folders: vec!["work".into()], subfolders: false, ..Scope::default() }, &m));
        // Sender, domain.
        assert!(act(Scope { senders: vec!["NEWS@shop.example".into()], ..Scope::default() }, &m));
        assert!(act(Scope { domains: vec!["@shop.example".into()], ..Scope::default() }, &m));
        assert!(!act(Scope { domains: vec!["other.example".into()], ..Scope::default() }, &m));
        // Size.
        assert!(act(Scope { min_size_kb: Some(5), max_size_kb: Some(20), ..Scope::default() }, &m));
        assert!(!act(Scope { min_size_kb: Some(11), ..Scope::default() }, &m));
        assert!(!act(Scope { max_size_kb: Some(9), ..Scope::default() }, &m));
        // Read state.
        assert!(act(Scope { read: ReadState::Read, ..Scope::default() }, &m));
        assert!(!act(Scope { read: ReadState::Unread, ..Scope::default() }, &m));
        // Attachment.
        assert!(!act(Scope { has_attachment: Some(true), ..Scope::default() }, &m));
        assert!(act(Scope { has_attachment: Some(false), ..Scope::default() }, &m));
        // Label.
        assert!(act(label("receipt"), &m));
        assert!(!act(label("other"), &m));
    }

    #[test]
    fn archive_and_move_and_mark_read_are_idempotent() {
        let p = Policy::default();
        let fs = folders();
        let tree = Tree::new(&fs);
        let acct = Account::default();
        let mv = Rule {
            body: Body {
                scope: Scope::default(),
                action: Action { kind: ActionKind::Move, days: 5, target: Some("work".into()) },
                exclude: Exclude::default(),
            },
            ..rule("m", Scope::default(), ActionKind::Move, 5)
        };
        let m = msg("a", "inbox", 20, &["$seen"]);
        let d = plan_message(&m, NOW, &tree, &[mv.clone()], &[], &p, &acct);
        assert_eq!(d.outcome, Outcome::Move { to: "work".into(), stamp: None });
        // Already there: nothing to do.
        assert!(!plan_message(&msg("a", "work", 20, &["$seen"]), NOW, &tree, &[mv], &[], &p, &acct).acts());
        // Mark read acts once.
        let mr = [rule("r", Scope::default(), ActionKind::MarkRead, 1)];
        assert_eq!(run(&msg("a", "inbox", 5, &[]), &mr, &[], &p).outcome, Outcome::MarkRead);
        assert!(!run(&msg("a", "inbox", 5, &["$seen"]), &mr, &[], &p).acts());
    }

    #[test]
    fn protected_folders_and_drafts_are_untouched() {
        let p = Policy::default();
        let r = [rule("all", Scope::default(), ActionKind::Delete, 7)];
        let d = run(&msg("a", "drafts", 900, &["$seen"]), &r, &[], &p);
        assert_eq!(d.outcome, Outcome::Skip(Skip::Protected));
        // A rule cannot even be saved for a protected folder.
        let b = Body {
            scope: Scope { folders: vec!["role:drafts".into()], ..Scope::default() },
            action: Action { kind: ActionKind::Delete, days: 30, target: None },
            exclude: Exclude::default(),
        };
        assert_eq!(validate("x", &b, &p, &folders()).unwrap_err().code, "folderProtected");
        // Sent is eligible unless the administrator protects it.
        assert!(run(&msg("a", "sent", 900, &["$seen"]), &r, &[], &p).acts());
        let p2 = Policy { protected: vec!["drafts".into(), "Sent".into()], ..Policy::default() };
        assert_eq!(run(&msg("a", "sent", 900, &["$seen"]), &r, &[], &p2).outcome, Outcome::Skip(Skip::Protected));
    }

    #[test]
    fn trash_undo_window_and_mandatory_rules() {
        let p = Policy { undo_days: 7, mandatory: vec![("junk".into(), 30)], ..Policy::default() };
        let today = NOW.div_euclid(DAY);
        let fresh = format!("{TRASH_STAMP}{}", today - 3);
        let due = format!("{TRASH_STAMP}{}", today - 8);
        assert_eq!(run(&msg("a", "trash", 100, &[&fresh]), &[], &[], &p).outcome, Outcome::Skip(Skip::UndoWindow));
        let d = run(&msg("a", "trash", 100, &[&due]), &[], &[], &p);
        assert_eq!((d.outcome, d.source.as_str()), (Outcome::Destroy, "trash-undo"));
        // Mandatory: unread and flagged mail goes too; younger mail stays.
        let d = run(&msg("a", "junk", 31, &["$flagged"]), &[], &[], &p);
        assert_eq!((d.outcome, d.source.as_str()), (Outcome::Destroy, "mandatory:junk"));
        assert!(!run(&msg("a", "junk", 29, &[]), &[], &[], &p).acts());
        // A rule that deletes inside Trash only stamps, so the undo window applies.
        let r = [rule("t", Scope { folders: vec!["role:trash".into()], ..Scope::default() }, ActionKind::Delete, 10)];
        assert!(matches!(run(&msg("a", "trash", 100, &[]), &r, &[], &p).outcome, Outcome::Stamp(_)));
    }

    #[test]
    fn disabled_paused_and_off_switches_only_stop_user_rules() {
        let fs = folders();
        let tree = Tree::new(&fs);
        let r = [rule("all", Scope::default(), ActionKind::Delete, 7)];
        let m = msg("a", "inbox", 90, &["$seen"]);
        let mand = Policy { mandatory: vec![("inbox".into(), 365)], ..Policy::default() };
        let paused = Account { paused: true, ..Account::default() };
        assert!(!plan_message(&m, NOW, &tree, &r, &[], &mand, &paused).acts());
        let off = Policy { enabled: false, ..Policy::default() };
        assert!(!plan_message(&m, NOW, &tree, &r, &[], &off, &Account::default()).acts());
        let mut disabled = r.clone();
        disabled[0].enabled = false;
        assert!(!plan_message(&m, NOW, &tree, &disabled, &[], &Policy::default(), &Account::default()).acts());
        // Mandatory still applies while the user is paused.
        let old = msg("b", "inbox", 400, &["$seen"]);
        assert!(plan_message(&old, NOW, &tree, &r, &[], &mand, &paused).acts());
    }

    #[test]
    fn folder_defaults_inherit_down_the_tree() {
        let p = Policy::default();
        let parent = rule("work", Scope { folders: vec!["work".into()], subfolders: true, ..Scope::default() }, ActionKind::Archive, 100);
        let child = rule("proj", Scope { folders: vec!["proj".into()], subfolders: false, ..Scope::default() }, ActionKind::Delete, 30);
        let defaults = [parent, child];
        // The child folder's own default wins over its parent's.
        let d = run(&msg("a", "proj", 40, &["$seen"]), &[], &defaults, &p);
        assert_eq!(d.source, "folder:proj");
        // Another child of Work inherits the parent default; the Inbox has none.
        assert_eq!(run(&msg("a", "work", 120, &["$seen"]), &[], &defaults, &p).source, "folder:work");
        assert!(!run(&msg("a", "inbox", 900, &["$seen"]), &[], &defaults, &p).acts());
    }

    #[test]
    fn horizon_skips_quiet_folders() {
        let fs = folders();
        let tree = Tree::new(&fs);
        let p = Policy { mandatory: vec![("junk".into(), 30)], ..Policy::default() };
        let r = [rule("n", Scope { folders: vec!["work".into()], ..Scope::default() }, ActionKind::Delete, 45)];
        let h = |f: &str| folder_horizon(f, &tree, &r, &[], &p, &Account::default());
        assert_eq!(h("proj"), Some(45));
        assert_eq!(h("junk"), Some(30));
        assert_eq!(h("trash"), Some(0));
        assert_eq!(h("inbox"), None);
        assert_eq!(h("drafts"), None);
    }

    #[test]
    fn validation_enforces_floor_ceiling_and_shape() {
        let p = Policy { min_days: 30, max_days: 365, ..Policy::default() };
        let mk = |kind, days| Body { scope: Scope::default(), action: Action { kind, days, target: None }, exclude: Exclude::default() };
        let code = |b: Body| validate("r", &b, &p, &folders()).unwrap_err().code;
        assert_eq!(code(mk(ActionKind::Delete, 7)), "belowMinimum");
        assert_eq!(code(mk(ActionKind::Trash, 400)), "aboveMaximum");
        assert_eq!(code(mk(ActionKind::Move, 40)), "targetRequired");
        assert_eq!(code(mk(ActionKind::StripAttachments, 40)), "unsupportedAction");
        assert!(validate("r", &mk(ActionKind::Archive, 1), &p, &folders()).is_ok());
        assert!(validate("r", &mk(ActionKind::Delete, 30), &p, &folders()).is_ok());
        assert_eq!(validate("", &mk(ActionKind::Keep, 0), &p, &folders()).unwrap_err().code, "invalidName");
        let mut b = mk(ActionKind::Move, 40);
        b.action.target = Some("nope".into());
        assert_eq!(code(b), "unknownFolder");
    }

    #[test]
    fn templates_validate_under_default_policy_and_parse() {
        let p = Policy { min_days: 7, ..Policy::default() };
        for t in TEMPLATES {
            let b = (t.body)();
            validate(t.name, &b, &p, &folders()).unwrap_or_else(|e| panic!("{}: {e:?}", t.id));
            let j = serde_json::to_string(&b).unwrap();
            assert_eq!(serde_json::from_str::<Body>(&j).unwrap(), b);
        }
        assert!(serde_json::from_str::<Body>(r#"{"action":{"kind":"delete","days":3},"extra":1}"#).is_err());
    }

    #[test]
    fn mandatory_entries_parse() {
        let v = parse_mandatory(&["trash:30".into(), "junk: 15".into(), "bad".into(), "x:0".into(), ":5".into()]);
        assert_eq!(v, vec![("trash".to_string(), 30), ("junk".to_string(), 15)]);
    }
}
