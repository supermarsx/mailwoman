//! User retention: the job that applies each account's rules, the mail port it
//! uses, and the webmail routes (`/api/retention/*`).
//!
//! The decisions come from [`crate::retention_rules`]. This module only
//! gathers messages, runs the plan through the engine's JMAP surface (so it
//! works for proxied IMAP accounts too: the engine moves and destroys through
//! the same backend calls the web client uses) and records counts.
//!
//! * **Dry run equals real run.** Both build the same plan with
//!   [`build_plan`]; the real run executes it.
//! * **Batched and paced.** At most `retention.user.batchMessages` messages
//!   per call, `retention.user.pauseMs` between calls, and at most
//!   `retention.user.maxPerRun` messages changed per account per run. What is
//!   left waits for the next run.
//! * **Idempotent.** A message already moved, read or stamped no longer
//!   matches, so a second run finds nothing.
//! * **Held and restricted accounts** (`dsr.legalHold.subjects`, an active
//!   restriction) are skipped whole, unless `retention.user.runForRestricted`.
//! * **Counts only** are written to the run history and the audit log.

use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability,
    jobs::{JobCtx, JobOutput},
    retention_rules::{self as rr, Account, Action, ActionKind, Body, Decision, Exclude, Folder, Msg, Outcome, Policy, Rule, Scope},
    settings_registry,
};
use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use mw_store::{
    Store,
    settings_policy::SettingSubject,
    user_retention::{AccountRow, RuleRow, RunRow},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc, time::Duration};

mod pim;

pub(crate) const JOB_KIND: &str = "user_retention";
/// Messages one folder scan reads at most; the rest waits for the next run.
const SCAN_CAP: usize = 20_000;
const PAGE: usize = 200;
const SAMPLE: usize = 20;
/// Run history kept per server.
const HISTORY_DAYS: i64 = 400;

// ─── the mail port ─────────────────────────────────────────────────────────────

/// What the job needs from a mailbox. The real implementation speaks JMAP to the
/// engine; tests use an in-memory one.
#[async_trait]
pub(crate) trait MailPort: Send + Sync {
    async fn folders(&self) -> Result<Vec<Folder>, String>;
    /// Messages of `folder` received before `before` (unix), oldest first.
    async fn list(&self, folder: &str, before: i64, position: usize, limit: usize) -> Result<Vec<Msg>, String>;
    /// Move to `to`, adding the keyword `stamp` when given. Returns how many moved.
    async fn move_to(&self, ids: &[String], to: &str, stamp: Option<&str>) -> Result<usize, String>;
    async fn stamp(&self, ids: &[String], keyword: &str) -> Result<usize, String>;
    async fn mark_read(&self, ids: &[String]) -> Result<usize, String>;
    async fn destroy(&self, ids: &[String]) -> Result<usize, String>;
    /// Replace each message by a copy without its attachments (the copy keeps
    /// flags, keywords and the received date). With `dry` nothing changes and
    /// the report says what would. The originals are left for the caller.
    async fn strip(&self, _ids: &[String], _p: &StripParams, _dry: bool) -> Result<StripReport, String> {
        Err("removing attachments is not available on this mail port".into())
    }
}

/// What a strip needs besides the message ids.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct StripParams {
    pub min_bytes: u64,
    pub keep_inline: bool,
    /// `YYYY-MM-DD`.
    pub date: String,
}

/// The mail port's answer to a strip call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct StripReport {
    /// `(original id, new id, bytes reclaimed)`; the new id is empty on a dry run.
    pub replaced: Vec<(String, String, u64)>,
    /// Messages with nothing to remove or that must not be rewritten (signed,
    /// encrypted, already stripped).
    pub checked: Vec<String>,
    pub failed: u64,
}

use crate::retention_rules::CHECKED_KEYWORD;
/// Most messages one run rewrites per account; the rest waits for the next run.
const STRIP_RUN_CAP: usize = 300;
/// Messages one strip call carries.
const STRIP_BATCH: usize = 20;
/// Messages a preview opens to count exactly what a strip would reclaim.
const STRIP_VERIFY: usize = 100;

pub(crate) struct JmapPort {
    engine: Arc<mw_engine::Engine>,
    account: String,
}

impl JmapPort {
    pub(crate) async fn connect(state: &AppState, account: &str) -> Result<Self, String> {
        let engine = state.engine.as_ref().ok_or("mail engine is not running")?.clone();
        crate::engine_mode::ensure_account(&engine, account).await?;
        Ok(Self { engine, account: account.to_string() })
    }

    async fn call(&self, method: &str, mut args: Value) -> Result<Value, String> {
        args["accountId"] = json!(self.account);
        let request = json!({"methodCalls": [[method, args, "r0"]]});
        let resp = self.engine.handle_jmap(&self.account, &request).await;
        let call = resp
            .get("methodResponses")
            .and_then(Value::as_array)
            .and_then(|r| r.first())
            .and_then(Value::as_array)
            .ok_or_else(|| "the mail engine gave no answer".to_string())?;
        if call.first().and_then(Value::as_str) == Some("error") {
            let kind = call.get(1).and_then(|a| a.get("type")).and_then(Value::as_str).unwrap_or("error");
            return Err(format!("{method} failed: {kind}"));
        }
        Ok(call.get(1).cloned().unwrap_or(Value::Null))
    }

    async fn set(&self, args: Value) -> Result<Value, String> {
        self.call("Email/set", args).await
    }
}

fn unix_of(v: &Value) -> i64 {
    v.as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.timestamp())
        .unwrap_or(0)
}

#[async_trait]
impl MailPort for JmapPort {
    async fn folders(&self) -> Result<Vec<Folder>, String> {
        let r = self.call("Mailbox/get", json!({"ids": null})).await?;
        Ok(r["list"]
            .as_array()
            .map(|l| {
                l.iter()
                    .filter_map(|m| {
                        Some(Folder {
                            id: m["id"].as_str()?.to_string(),
                            name: m["name"].as_str().unwrap_or_default().to_string(),
                            role: m["role"].as_str().map(str::to_ascii_lowercase),
                            parent_id: m["parentId"].as_str().map(str::to_string),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn list(&self, folder: &str, before: i64, position: usize, limit: usize) -> Result<Vec<Msg>, String> {
        let before = chrono::DateTime::from_timestamp(before.max(0), 0)
            .unwrap_or_default()
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let q = self
            .call(
                "Email/query",
                json!({
                    "filter": {"inMailbox": folder, "before": before},
                    "sort": [{"property": "receivedAt", "isAscending": true}],
                    "position": position,
                    "limit": limit,
                }),
            )
            .await?;
        let ids: Vec<Value> = q["ids"].as_array().cloned().unwrap_or_default();
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let g = self
            .call(
                "Email/get",
                json!({"ids": ids, "properties": ["id", "mailboxIds", "receivedAt", "size", "keywords", "from", "hasAttachment", "subject"]}),
            )
            .await?;
        Ok(g["list"]
            .as_array()
            .map(|l| {
                l.iter()
                    .filter_map(|e| {
                        let mailbox = e["mailboxIds"].as_object()?.keys().next()?.clone();
                        Some(Msg {
                            id: e["id"].as_str()?.to_string(),
                            mailbox_id: mailbox,
                            received_at: unix_of(&e["receivedAt"]),
                            size: e["size"].as_i64().unwrap_or(0),
                            keywords: e["keywords"]
                                .as_object()
                                .map(|k| k.iter().filter(|(_, v)| v.as_bool() == Some(true)).map(|(k, _)| k.clone()).collect())
                                .unwrap_or_default(),
                            from: e["from"][0]["email"].as_str().unwrap_or_default().to_ascii_lowercase(),
                            subject: e["subject"].as_str().unwrap_or_default().chars().take(120).collect(),
                            has_attachment: e["hasAttachment"].as_bool().unwrap_or(false),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn move_to(&self, ids: &[String], to: &str, stamp: Option<&str>) -> Result<usize, String> {
        let mut update = serde_json::Map::new();
        for id in ids {
            let mut patch = serde_json::Map::new();
            patch.insert("mailboxIds".into(), json!({ to: true }));
            if let Some(k) = stamp {
                patch.insert(format!("keywords/{k}"), json!(true));
            }
            update.insert(id.clone(), Value::Object(patch));
        }
        let r = self.set(json!({"update": update})).await?;
        Ok(r["updated"].as_object().map_or(0, |u| u.len()))
    }

    async fn stamp(&self, ids: &[String], keyword: &str) -> Result<usize, String> {
        let update: serde_json::Map<String, Value> = ids
            .iter()
            .map(|id| (id.clone(), json!({ format!("keywords/{keyword}"): true })))
            .collect();
        let r = self.set(json!({"update": update})).await?;
        Ok(r["updated"].as_object().map_or(0, |u| u.len()))
    }

    async fn mark_read(&self, ids: &[String]) -> Result<usize, String> {
        self.stamp(ids, "$seen").await
    }

    async fn destroy(&self, ids: &[String]) -> Result<usize, String> {
        let r = self.set(json!({"destroy": ids})).await?;
        Ok(r["destroyed"].as_array().map_or(0, Vec::len))
    }

    async fn strip(&self, ids: &[String], p: &StripParams, dry: bool) -> Result<StripReport, String> {
        let r = self
            .call(
                "Email/mailwomanStrip",
                json!({"ids": ids, "minAttachmentBytes": p.min_bytes, "keepInlineImages": p.keep_inline, "date": p.date, "dryRun": dry}),
            )
            .await?;
        let mut out = StripReport::default();
        if let Some(m) = r["replaced"].as_object() {
            for (old, v) in m {
                out.replaced.push((old.clone(), v["newId"].as_str().unwrap_or_default().to_string(), v["reclaimedBytes"].as_u64().unwrap_or(0)));
            }
        }
        out.checked.extend(r["unchanged"].as_array().into_iter().flatten().filter_map(|v| v.as_str().map(str::to_string)));
        out.checked.extend(r["skipped"].as_object().into_iter().flat_map(|m| m.keys().cloned()));
        out.failed = r["notReplaced"].as_object().map_or(0, |m| m.len()) as u64;
        Ok(out)
    }
}

// ─── plan and execute ──────────────────────────────────────────────────────────

/// Everything the evaluation reads.
pub(crate) struct Inputs<'a> {
    pub now: i64,
    pub rules: &'a [Rule],
    pub defaults: &'a [Rule],
    pub policy: &'a Policy,
    pub acct: &'a Account,
}

#[derive(Debug, Default)]
pub(crate) struct Plan {
    /// Messages something will happen to, oldest first per folder.
    pub entries: Vec<(Msg, Decision)>,
    pub scanned: usize,
    /// A folder scan stopped at the cap.
    pub truncated: bool,
    /// Folder reads that failed.
    pub errors: usize,
    /// How strip outcomes are carried out.
    pub strip: StripParams,
}

/// The sender addresses in a `mail.list.keep` list. Entries are `sender:<address>`
/// or `list:<List-Id>`; retention sees only the sender, so list entries do not apply here.
pub(crate) fn kept_senders(entries: &[String]) -> Vec<String> {
    entries
        .iter()
        .filter_map(|e| e.strip_prefix("sender:"))
        .map(|a| a.trim().to_ascii_lowercase())
        .filter(|a| !a.is_empty())
        .collect()
}

/// Remove from the plan every message from a kept sender; returns how many.
pub(crate) fn drop_kept(plan: &mut Plan, kept: &[String]) -> usize {
    if kept.is_empty() {
        return 0;
    }
    let before = plan.entries.len();
    plan.entries.retain(|(m, _)| !kept.iter().any(|k| k.eq_ignore_ascii_case(&m.from)));
    before - plan.entries.len()
}

pub(crate) async fn build_plan(port: &dyn MailPort, folders: &[Folder], i: &Inputs<'_>) -> Plan {
    let tree = rr::Tree::new(folders);
    let mut plan = Plan {
        strip: StripParams {
            min_bytes: (i.policy.strip_min_attachment_kb.max(1) as u64) * 1024,
            keep_inline: i.policy.strip_keep_inline,
            date: chrono::DateTime::from_timestamp(i.now, 0).unwrap_or_default().format("%Y-%m-%d").to_string(),
        },
        ..Plan::default()
    };
    for f in folders {
        let Some(horizon) = rr::folder_horizon(&f.id, &tree, i.rules, i.defaults, i.policy, i.acct) else {
            continue;
        };
        let before = i.now - horizon * rr::DAY;
        let mut position = 0;
        let mut seen = 0;
        loop {
            let page = match port.list(&f.id, before, position, PAGE).await {
                Ok(p) => p,
                Err(_) => {
                    plan.errors += 1;
                    break;
                }
            };
            let n = page.len();
            for m in page {
                seen += 1;
                let d = rr::plan_message(&m, i.now, &tree, i.rules, i.defaults, i.policy, i.acct);
                if d.acts() {
                    plan.entries.push((m, d));
                }
            }
            position += n;
            if n < PAGE {
                break;
            }
            if seen >= SCAN_CAP {
                plan.truncated = true;
                break;
            }
        }
        plan.scanned += seen;
    }
    plan
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Counts {
    pub moved: u64,
    pub trashed: u64,
    pub purged: u64,
    pub marked_read: u64,
    pub stamped: u64,
    /// Messages rewritten without their attachments.
    pub stripped: u64,
    /// Bytes those rewrites freed (an estimate in a preview beyond the checked sample).
    pub reclaimed_bytes: u64,
    /// Messages a strip looked at and left alone.
    pub strip_skipped: u64,
}

impl Counts {
    pub fn total(&self) -> u64 {
        self.moved + self.trashed + self.purged + self.marked_read + self.stamped + self.stripped
    }
}

fn count_of(d: &Decision) -> Counts {
    let mut c = Counts::default();
    match &d.outcome {
        Outcome::Move { stamp: Some(_), .. } => c.trashed = 1,
        Outcome::Move { stamp: None, .. } => c.moved = 1,
        Outcome::Destroy => c.purged = 1,
        Outcome::MarkRead => c.marked_read = 1,
        Outcome::Stamp(_) => c.stamped = 1,
        Outcome::Strip { .. } => c.stripped = 1,
        Outcome::Skip(_) => {}
    }
    c
}

/// What the plan would do, by kind, without doing it.
pub(crate) fn tally(plan: &Plan, cap: usize) -> Counts {
    let mut c = Counts::default();
    for (_, d) in plan.entries.iter().take(cap) {
        let x = count_of(d);
        c.moved += x.moved;
        c.trashed += x.trashed;
        c.purged += x.purged;
        c.marked_read += x.marked_read;
        c.stamped += x.stamped;
        c.stripped += x.stripped;
    }
    c.reclaimed_bytes = plan
        .entries
        .iter()
        .take(cap)
        .filter(|(_, d)| matches!(d.outcome, Outcome::Strip { .. }))
        .map(|(m, _)| m.size.max(0) as u64)
        .sum();
    c
}

#[derive(Debug, Default)]
pub(crate) struct Executed {
    pub counts: Counts,
    pub errors: u64,
}

/// Carry the plan out. `cap` limits the messages changed; `batch` the size of
/// one call; `pause` the wait between calls.
pub(crate) async fn execute(port: &dyn MailPort, plan: &Plan, cap: usize, batch: usize, pause: Duration) -> Executed {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
    enum Key {
        Move(String, Option<String>),
        Stamp(String),
        Read,
        Destroy,
        Strip(Option<(String, String)>),
    }
    let mut groups: BTreeMap<Key, Vec<String>> = BTreeMap::new();
    for (m, d) in plan.entries.iter().take(cap) {
        let key = match &d.outcome {
            Outcome::Move { to, stamp } => Key::Move(to.clone(), stamp.clone()),
            Outcome::Stamp(s) => Key::Stamp(s.clone()),
            Outcome::MarkRead => Key::Read,
            Outcome::Destroy => Key::Destroy,
            Outcome::Strip { hold } => Key::Strip(hold.clone()),
            Outcome::Skip(_) => continue,
        };
        groups.entry(key).or_default().push(m.id.clone());
    }
    let mut out = Executed::default();
    let batch = batch.clamp(1, 200);
    let mut strip_budget = STRIP_RUN_CAP;
    for (key, ids) in groups {
        if let Key::Strip(hold) = &key {
            for chunk in ids.chunks(STRIP_BATCH) {
                if strip_budget == 0 {
                    break;
                }
                let chunk = &chunk[..chunk.len().min(strip_budget)];
                strip_budget -= chunk.len();
                strip_chunk(port, &plan.strip, hold.as_ref(), chunk, &mut out).await;
                if !pause.is_zero() {
                    tokio::time::sleep(pause).await;
                }
            }
            continue;
        }
        for chunk in ids.chunks(batch) {
            let r = match &key {
                Key::Move(to, stamp) => port.move_to(chunk, to, stamp.as_deref()).await,
                Key::Stamp(s) => port.stamp(chunk, s).await,
                Key::Read => port.mark_read(chunk).await,
                Key::Destroy => port.destroy(chunk).await,
                Key::Strip(_) => unreachable!("strips are carried out above"),
            };
            match r {
                Ok(n) => {
                    let n = n as u64;
                    match &key {
                        Key::Move(_, Some(_)) => out.counts.trashed += n,
                        Key::Move(_, None) => out.counts.moved += n,
                        Key::Stamp(_) => out.counts.stamped += n,
                        Key::Read => out.counts.marked_read += n,
                        Key::Destroy => out.counts.purged += n,
                        Key::Strip(_) => {}
                    }
                    out.errors += (chunk.len() as u64).saturating_sub(n);
                }
                Err(_) => out.errors += chunk.len() as u64,
            }
            if !pause.is_zero() {
                tokio::time::sleep(pause).await;
            }
        }
    }
    out
}

/// Rewrite one chunk without attachments, then put the originals aside (Trash
/// with the dated keyword) or delete them. A copy is acknowledged before its
/// original is touched, so a failure leaves a duplicate, never a loss.
async fn strip_chunk(port: &dyn MailPort, p: &StripParams, hold: Option<&(String, String)>, ids: &[String], out: &mut Executed) {
    let report = match port.strip(ids, p, false).await {
        Ok(r) => r,
        Err(_) => {
            out.errors += ids.len() as u64;
            return;
        }
    };
    out.errors += report.failed;
    if !report.checked.is_empty() {
        out.counts.strip_skipped += report.checked.len() as u64;
        if port.stamp(&report.checked, CHECKED_KEYWORD).await.is_err() {
            out.errors += 1;
        }
    }
    let originals: Vec<String> = report.replaced.iter().map(|r| r.0.clone()).collect();
    if originals.is_empty() {
        return;
    }
    let settled = match hold {
        Some((trash, stamp)) => port.move_to(&originals, trash, Some(stamp)).await,
        None => port.destroy(&originals).await,
    };
    let settled = settled.unwrap_or(0);
    out.counts.stripped += originals.len() as u64;
    out.counts.reclaimed_bytes += report.replaced.iter().map(|r| r.2).sum::<u64>();
    // An original that stayed behind is a duplicate the user can remove.
    out.errors += (originals.len() as u64).saturating_sub(settled as u64);
}

// ─── settings and rows ─────────────────────────────────────────────────────────

struct Who {
    account_id: String,
    address: String,
    domain: String,
}

impl Who {
    fn of(account_id: &str, username: &str) -> Self {
        let address = username.trim().to_ascii_lowercase();
        let domain = address.rsplit_once('@').map(|x| x.1.to_string()).unwrap_or_default();
        Self { account_id: account_id.to_string(), address, domain }
    }
    fn subject(&self) -> SettingSubject<'_> {
        SettingSubject { personal_owner: None, domain: &self.domain, account_id: &self.account_id, address: &self.address }
    }
}

async fn setting(store: &Store, key: &str, who: &Who) -> Value {
    match settings_registry::effective_for(store, key, &who.subject()).await {
        Ok(Some(v)) => v.value,
        _ => settings_registry::lookup(key).map(|d| d.default_value()).unwrap_or(Value::Null),
    }
}

async fn list_setting(store: &Store, key: &str, who: &Who) -> Vec<String> {
    setting(store, key, who)
        .await
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// The administrator's limits for one account.
pub(crate) async fn policy_for(store: &Store, who_account: &str, username: &str) -> Policy {
    let who = Who::of(who_account, username);
    let n = |v: Value, d: i64| v.as_i64().unwrap_or(d);
    Policy {
        enabled: setting(store, "retention.user.enabled", &who).await.as_bool().unwrap_or(true),
        min_days: n(setting(store, "retention.user.minDays", &who).await, 7),
        max_days: n(setting(store, "retention.user.maxDays", &who).await, 0),
        max_rules: n(setting(store, "retention.user.maxRules", &who).await, 50),
        allow_permanent: setting(store, "retention.user.allowPermanentDelete", &who).await.as_bool().unwrap_or(false),
        undo_days: n(setting(store, "retention.user.undoDays", &who).await, 7),
        mandatory: rr::parse_mandatory(&list_setting(store, "retention.user.mandatory", &who).await),
        protected: list_setting(store, "retention.user.protectedFolders", &who).await,
        allow_strip: setting(store, "retention.user.allowStripAttachments", &who).await.as_bool().unwrap_or(false),
        strip_min_age_days: n(setting(store, "retention.user.strip.minAgeDays", &who).await, 90).max(7),
        strip_min_attachment_kb: n(setting(store, "retention.user.strip.minAttachmentKb", &who).await, 100).max(1),
        strip_keep_inline: setting(store, "retention.user.strip.keepInlineImages", &who).await.as_bool().unwrap_or(true),
        strip_undo_days: n(setting(store, "retention.user.strip.undoDays", &who).await, -1),
    }
}

fn rule_of(r: &RuleRow) -> Option<Rule> {
    let body: Body = serde_json::from_str(&r.body_json).ok()?;
    Some(Rule { id: r.id.clone(), name: r.name.clone(), enabled: r.enabled, body })
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultRow {
    id: String,
    name: String,
    #[serde(default = "yes")]
    enabled: bool,
    body: Body,
}

fn yes() -> bool {
    true
}

fn defaults_of(a: &AccountRow) -> Vec<Rule> {
    serde_json::from_str::<Vec<DefaultRow>>(&a.folder_defaults_json)
        .unwrap_or_default()
        .into_iter()
        .map(|d| Rule { id: d.id, name: d.name, enabled: d.enabled, body: d.body })
        .collect()
}

fn account_of(a: &AccountRow, p: &Policy, now: i64) -> (Account, Policy) {
    let mut p = p.clone();
    if a.undo_days >= 0 {
        p.undo_days = a.undo_days.min(p.undo_days);
    }
    (
        Account { enabled: a.enabled, trash_first: a.trash_first || !p.allow_permanent, paused: a.paused_until > now },
        p,
    )
}

// ─── running one account ───────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub(crate) struct Summary {
    pub skipped: Option<&'static str>,
    pub scanned: usize,
    pub matched: usize,
    pub would: Counts,
    pub done: Counts,
    pub errors: u64,
    pub truncated: bool,
    pub by_source: BTreeMap<String, u64>,
    pub sample: Vec<Value>,
    /// Enabled strip rules left unused because the administrator turned the
    /// action off.
    pub strip_rules_skipped: u64,
    /// The preview's reclaimed bytes include a size-based guess.
    pub strip_estimate: bool,
}

impl Summary {
    pub fn json(&self) -> Value {
        json!({
            "skipped": self.skipped,
            "scanned": self.scanned,
            "matched": self.matched,
            "would": self.would,
            "done": self.done,
            "errors": self.errors,
            "truncated": self.truncated,
            "bySource": self.by_source,
            "sample": self.sample,
            "stripRulesSkipped": self.strip_rules_skipped,
            "stripEstimate": self.strip_estimate,
        })
    }
}

fn describe(d: &Decision) -> &'static str {
    match &d.outcome {
        Outcome::Move { stamp: Some(_), .. } => "trash",
        Outcome::Move { .. } => "move",
        Outcome::Stamp(_) => "trash",
        Outcome::Destroy => "delete",
        Outcome::MarkRead => "markRead",
        Outcome::Strip { .. } => "stripAttachments",
        Outcome::Skip(_) => "none",
    }
}

pub(crate) async fn is_held(store: &Store, account: &str) -> bool {
    let who = Who::of("", "");
    let subjects = list_setting(store, "dsr.legalHold.subjects", &who).await;
    if subjects.is_empty() {
        return false;
    }
    store.retention_held_accounts(&subjects).await.map(|h| h.iter().any(|a| a == account)).unwrap_or(true)
}

/// Run (or preview) one account's rules. `only` evaluates a draft rule alone.
pub(crate) async fn run_account(
    state: &AppState,
    port: Option<&dyn MailPort>,
    account: &str,
    username: &str,
    dry: bool,
    trigger: &str,
    only: Option<Body>,
) -> Result<Summary, String> {
    let store = &state.store;
    let now = chrono::Utc::now().timestamp();
    let mut summary = Summary::default();
    if is_held(store, account).await {
        summary.skipped = Some("legalHold");
        return Ok(summary);
    }
    let who = Who::of(account, username);
    if !setting(store, "retention.user.runForRestricted", &who).await.as_bool().unwrap_or(false)
        && matches!(store.restriction_active(account, now).await, Ok(Some(_)))
    {
        summary.skipped = Some("restricted");
        return Ok(summary);
    }
    let base = policy_for(store, account, username).await;
    let row = store.retention_account(account).await.map_err(|e| e.to_string())?;
    let (acct, policy) = account_of(&row, &base, now);
    let (rules, defaults): (Vec<Rule>, Vec<Rule>) = match only {
        Some(body) => (vec![Rule { id: "draft".into(), name: "draft".into(), enabled: true, body }], Vec::new()),
        None => (
            store.retention_rules(account).await.map_err(|e| e.to_string())?.iter().filter_map(rule_of).collect(),
            defaults_of(&row),
        ),
    };
    let user_active = policy.enabled && acct.enabled && !acct.paused && rules.iter().any(|r| r.enabled);
    if !user_active && policy.mandatory.is_empty() && defaults.iter().all(|d| !d.enabled) {
        summary.skipped = Some("noRules");
        return Ok(summary);
    }
    let owned;
    let port: &dyn MailPort = match port {
        Some(p) => p,
        None => {
            owned = JmapPort::connect(state, account).await?;
            &owned
        }
    };
    let folders = port.folders().await?;
    let inputs = Inputs { now, rules: &rules, defaults: &defaults, policy: &policy, acct: &acct };
    let mut plan = build_plan(port, &folders, &inputs).await;
    // Senders the user chose to always keep (`mail.list.keep`) are left alone by every rule.
    let kept = kept_senders(&list_setting(store, "mail.list.keep", &who).await);
    drop_kept(&mut plan, &kept);
    let cap = setting(store, "retention.user.maxPerRun", &who).await.as_i64().unwrap_or(5000).max(1) as usize;
    summary.scanned = plan.scanned;
    summary.matched = plan.entries.len();
    summary.truncated = plan.truncated || plan.entries.len() > cap;
    summary.would = tally(&plan, cap);
    for (_, d) in plan.entries.iter().take(cap) {
        *summary.by_source.entry(d.source.clone()).or_default() += 1;
    }
    summary.sample = plan
        .entries
        .iter()
        .take(SAMPLE)
        .map(|(m, d)| json!({"id": m.id, "folder": m.mailbox_id, "subject": m.subject, "from": m.from, "receivedAt": m.received_at, "action": describe(d), "source": d.source}))
        .collect();
    summary.errors = plan.errors as u64;
    if !policy.allow_strip {
        summary.strip_rules_skipped =
            rules.iter().filter(|r| r.enabled && r.body.action.kind == ActionKind::StripAttachments).count() as u64;
    }
    if dry {
        verify_strips(port, &plan, cap, &mut summary).await;
        return Ok(summary);
    }
    let batch = setting(store, "retention.user.batchMessages", &who).await.as_i64().unwrap_or(200).max(1) as usize;
    let pause = setting(store, "retention.user.pauseMs", &who).await.as_i64().unwrap_or(100).max(0) as u64;
    let started = now;
    let done = execute(port, &plan, cap, batch, Duration::from_millis(pause)).await;
    summary.done = done.counts;
    summary.errors += done.errors;
    let finished = chrono::Utc::now().timestamp();
    let run = RunRow {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account.to_string(),
        started_at: started,
        finished_at: finished,
        dry_run: false,
        triggered_by: trigger.to_string(),
        matched: summary.matched as i64,
        acted: summary.done.total() as i64,
        errors: summary.errors as i64,
        counts_json: json!({"done": summary.done, "bySource": summary.by_source, "stripRulesSkipped": summary.strip_rules_skipped}).to_string(),
    };
    store.retention_run_put(&run).await.map_err(|e| e.to_string())?;
    let mut row = row;
    row.last_run_at = finished;
    row.updated_at = row.updated_at.max(finished);
    store.retention_account_put(&row).await.map_err(|e| e.to_string())?;
    if summary.done.total() > 0 || summary.errors > 0 {
        audit(state, &who, trigger, &summary).await;
    }
    Ok(summary)
}

/// Open the first planned strips (nothing is changed) so the preview reports
/// the messages that really qualify and the bytes they free. Beyond the sample
/// the planner's count and the messages' sizes stand in, and `strip_estimate`
/// says so.
async fn verify_strips(port: &dyn MailPort, plan: &Plan, cap: usize, summary: &mut Summary) {
    let planned: Vec<&Msg> = plan
        .entries
        .iter()
        .take(cap)
        .filter(|(_, d)| matches!(d.outcome, Outcome::Strip { .. }))
        .map(|(m, _)| m)
        .collect();
    if planned.is_empty() {
        return;
    }
    let (mut messages, mut bytes, mut checked_n) = (0u64, 0u64, 0usize);
    let sample = planned.len().min(STRIP_VERIFY);
    for chunk in planned[..sample].chunks(STRIP_BATCH) {
        let ids: Vec<String> = chunk.iter().map(|m| m.id.clone()).collect();
        match port.strip(&ids, &plan.strip, true).await {
            Ok(r) => {
                checked_n += chunk.len();
                messages += r.replaced.len() as u64;
                bytes += r.replaced.iter().map(|x| x.2).sum::<u64>();
            }
            Err(_) => {
                summary.strip_estimate = true;
                summary.errors += 1;
                // Count the unreadable chunk by its planned size.
                messages += chunk.len() as u64;
                bytes += chunk.iter().map(|m| m.size.max(0) as u64).sum::<u64>();
                checked_n += chunk.len();
            }
        }
    }
    let rest = &planned[checked_n.min(planned.len())..];
    if !rest.is_empty() {
        summary.strip_estimate = true;
        messages += rest.len() as u64;
        bytes += rest.iter().map(|m| m.size.max(0) as u64).sum::<u64>();
    }
    summary.would.stripped = messages;
    summary.would.reclaimed_bytes = bytes;
}

async fn audit(state: &AppState, who: &Who, trigger: &str, s: &Summary) {
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: if trigger == "scheduler" { "system".into() } else { who.address.clone() },
        actor_kind: if trigger == "scheduler" { mw_admin::ActorKind::System } else { mw_admin::ActorKind::User },
        action: "user-retention-run".into(),
        target: Some(who.account_id.clone()),
        detail_json: mw_admin::redact_detail(&json!({
            "trigger": trigger, "matched": s.matched, "done": s.done, "errors": s.errors,
        })),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("user retention audit entry not recorded: {e}");
    }
}

// ─── the job ───────────────────────────────────────────────────────────────────

pub(crate) fn default_schedule() -> crate::jobs::Schedule {
    crate::jobs::Schedule::Daily { daily_at: "04:20".into(), tz: "UTC".into() }
}

pub(crate) async fn run_job(ctx: JobCtx) -> Result<JobOutput, String> {
    let state = ctx.state()?;
    let store = &state.store;
    let now = chrono::Utc::now().timestamp();
    let global = Who::of("", "");
    let interval = setting(store, "retention.user.intervalHours", &global).await.as_i64().unwrap_or(24).max(1) * 3600;
    let accounts = store.list_accounts().await.map_err(|e| e.to_string())?;
    let (mut ran, mut changed, mut errors, mut skipped) = (0u64, 0u64, 0u64, 0u64);
    for a in accounts {
        let row = store.retention_account(&a.id).await.map_err(|e| e.to_string())?;
        if row.last_run_at + interval > now && ctx.triggered_by == "scheduler" {
            skipped += 1;
            continue;
        }
        match run_account(state, None, &a.id, &a.username, false, &ctx.triggered_by, None).await {
            Ok(s) if s.skipped.is_some() => skipped += 1,
            Ok(s) => {
                ran += 1;
                changed += s.done.total();
                errors += s.errors;
            }
            Err(e) => {
                errors += 1;
                tracing::debug!("user retention for {} not run: {e}", a.id);
            }
        }
        // Calendar events and notes, for people who opted in.
        match pim::run_pim(state, &a.id, &a.username, false, &ctx.triggered_by).await {
            Ok(s) if s.skipped.is_none() => {
                changed += s.changed();
                errors += s.errors;
            }
            Ok(_) => {}
            Err(e) => tracing::debug!("pim retention for {} not run: {e}", a.id),
        }
    }
    let _ = store.retention_runs_prune(now - HISTORY_DAYS * rr::DAY).await;
    // Notes whose undo window has ended are gone for good.
    let _ = store.note_holds_purge(now).await;
    let _ = store
        .set_setting(
            "retention.user.lastJob",
            &json!({"at": now, "accounts": ran, "changed": changed, "errors": errors, "skipped": skipped}).to_string(),
        )
        .await;
    Ok(JobOutput::new(
        format!("{ran} account(s) processed, {changed} message(s) changed, {skipped} skipped"),
        json!({"accounts": ran, "changed": changed, "errors": errors, "skipped": skipped}),
    ))
}

// ─── webmail routes ────────────────────────────────────────────────────────────

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/retention/rules", get(list_rules).post(create_rule))
        .route("/api/retention/rules/{id}", put(update_rule).delete(delete_rule))
        .route("/api/retention/settings", put(put_settings))
        .route("/api/retention/preview", post(preview))
        .route("/api/retention/run", post(run_now))
        .route("/api/retention/templates/{id}/adopt", post(adopt_template))
        .route("/api/retention/effective", get(effective))
        .merge(pim::router())
}

struct Me {
    account_id: String,
    username: String,
}

impl Me {
    fn who(&self) -> Who {
        Who::of(&self.account_id, &self.username)
    }
}

async fn me(state: &AppState, headers: &HeaderMap) -> Result<Me, Response> {
    let s = crate::authed(state, headers).await?;
    Ok(Me { account_id: s.account_id, username: s.username })
}

fn no_store(r: impl IntoResponse) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], r).into_response()
}

fn fail(status: StatusCode, code: &str, detail: impl Into<String>) -> Response {
    no_store((status, Json(json!({"error": code, "detail": detail.into()}))))
}

fn store_fail(e: impl std::fmt::Display) -> Response {
    tracing::warn!("user retention store error: {e}");
    fail(StatusCode::SERVICE_UNAVAILABLE, "unavailable", "The retention settings are not available right now.")
}

fn rule_json(r: &RuleRow) -> Value {
    let body: Value = serde_json::from_str(&r.body_json).unwrap_or(Value::Null);
    json!({
        "id": r.id, "name": r.name, "enabled": r.enabled, "order": r.sort_order, "template": r.template,
        "scope": body["scope"], "action": body["action"], "exclude": body["exclude"],
        "createdAt": r.created_at, "updatedAt": r.updated_at,
    })
}

fn policy_json(p: &Policy) -> Value {
    json!({
        "enabled": p.enabled, "minDays": p.min_days, "maxDays": p.max_days, "maxRules": p.max_rules,
        "allowPermanentDelete": p.allow_permanent, "undoDays": p.undo_days,
        "mandatory": p.mandatory.iter().map(|(r, d)| json!({"role": r, "days": d})).collect::<Vec<_>>(),
        "protectedFolders": p.protected,
        "allowStripAttachments": p.allow_strip,
        "strip": {
            "minAgeDays": p.strip_min_age_days, "minAttachmentKb": p.strip_min_attachment_kb,
            "keepInlineImages": p.strip_keep_inline, "undoDays": p.strip_undo(),
        },
    })
}

async fn template_ids(store: &Store, me: &Me) -> Vec<String> {
    list_setting(store, "retention.user.templates", &me.who()).await
}

async fn list_rules(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let me = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let store = &state.store;
    let result: Result<Value, mw_store::StoreError> = async {
        let policy = policy_for(store, &me.account_id, &me.username).await;
        let rules = store.retention_rules(&me.account_id).await?;
        let row = store.retention_account(&me.account_id).await?;
        let runs = store.retention_runs(&me.account_id, 10).await?;
        let allowed = template_ids(store, &me).await;
        let now = chrono::Utc::now().timestamp();
        let interval = setting(store, "retention.user.intervalHours", &Who::of("", "")).await.as_i64().unwrap_or(24) * 3600;
        let defaults: Value = serde_json::from_str(&row.folder_defaults_json).unwrap_or(json!([]));
        Ok(json!({
            "policy": policy_json(&policy),
            "settings": {
                "enabled": row.enabled, "trashFirst": row.trash_first || !policy.allow_permanent,
                "undoDays": if row.undo_days < 0 { policy.undo_days } else { row.undo_days.min(policy.undo_days) },
                "pausedUntil": row.paused_until, "paused": row.paused_until > now,
                "lastRunAt": row.last_run_at,
                "nextRunAt": if row.last_run_at == 0 { Value::Null } else { json!(row.last_run_at + interval) },
            },
            "folderDefaults": defaults,
            "rules": rules.iter().map(rule_json).collect::<Vec<_>>(),
            "templates": rr::TEMPLATES.iter().filter(|t| allowed.iter().any(|a| a == t.id)).map(|t| json!({"id": t.id, "name": t.name, "rule": (t.body)()})).collect::<Vec<_>>(),
            "runs": runs.iter().map(|r| json!({
                "id": r.id, "startedAt": r.started_at, "finishedAt": r.finished_at, "trigger": r.triggered_by,
                "matched": r.matched, "acted": r.acted, "errors": r.errors,
                "counts": serde_json::from_str::<Value>(&r.counts_json).unwrap_or(Value::Null),
            })).collect::<Vec<_>>(),
        }))
    }
    .await;
    match result {
        Ok(v) => no_store(Json(v)),
        Err(e) => store_fail(e),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuleIn {
    name: String,
    #[serde(default = "yes")]
    enabled: bool,
    #[serde(default)]
    scope: Scope,
    action: Action,
    #[serde(default)]
    exclude: Exclude,
}

async fn folders_of(state: &AppState, account: &str) -> Vec<Folder> {
    match JmapPort::connect(state, account).await {
        Ok(p) => p.folders().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

async fn save_rule(state: &AppState, me: &Me, id: Option<String>, input: RuleIn, template: Option<String>) -> Response {
    let store = &state.store;
    let policy = policy_for(store, &me.account_id, &me.username).await;
    if !policy.enabled {
        return fail(StatusCode::FORBIDDEN, "disabled", "Retention rules are turned off on this server.");
    }
    let body = Body { scope: input.scope, action: input.action, exclude: input.exclude };
    let folders = folders_of(state, &me.account_id).await;
    if let Err(e) = rr::validate(&input.name, &body, &policy, &folders) {
        return fail(StatusCode::UNPROCESSABLE_ENTITY, e.code, e.detail);
    }
    let existing = match store.retention_rules(&me.account_id).await {
        Ok(r) => r,
        Err(e) => return store_fail(e),
    };
    if id.is_none() && existing.len() as i64 >= policy.max_rules {
        return fail(StatusCode::UNPROCESSABLE_ENTITY, "tooManyRules", format!("You can keep at most {} rules.", policy.max_rules));
    }
    let now = chrono::Utc::now().timestamp();
    let prev = id.as_ref().and_then(|i| existing.iter().find(|r| &r.id == i));
    if id.is_some() && prev.is_none() {
        return fail(StatusCode::NOT_FOUND, "notFound", "No such rule.");
    }
    let row = RuleRow {
        id: id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        account_id: me.account_id.clone(),
        name: input.name.trim().to_string(),
        enabled: input.enabled,
        sort_order: prev.map(|p| p.sort_order).unwrap_or(existing.iter().map(|r| r.sort_order).max().unwrap_or(0) + 1),
        template: prev.and_then(|p| p.template.clone()).or(template),
        body_json: serde_json::to_string(&body).unwrap_or_default(),
        created_at: prev.map(|p| p.created_at).unwrap_or(now),
        updated_at: now,
    };
    match store.retention_rule_put(&row).await {
        Ok(()) => no_store(Json(rule_json(&row))),
        Err(e) => store_fail(e),
    }
}

async fn create_rule(State(state): State<AppState>, headers: HeaderMap, Json(input): Json<RuleIn>) -> Response {
    match me(&state, &headers).await {
        Ok(me) => save_rule(&state, &me, None, input, None).await,
        Err(r) => r,
    }
}

async fn update_rule(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>, Json(input): Json<RuleIn>) -> Response {
    match me(&state, &headers).await {
        Ok(me) => save_rule(&state, &me, Some(id), input, None).await,
        Err(r) => r,
    }
}

async fn delete_rule(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Response {
    let me = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    match state.store.retention_rule_delete(&me.account_id, &id).await {
        Ok(true) => no_store(Json(json!({"deleted": true}))),
        Ok(false) => fail(StatusCode::NOT_FOUND, "notFound", "No such rule."),
        Err(e) => store_fail(e),
    }
}

async fn adopt_template(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Response {
    let me = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let allowed = template_ids(&state.store, &me).await;
    let Some(t) = rr::template(&id).filter(|t| allowed.iter().any(|a| a == t.id)) else {
        return fail(StatusCode::NOT_FOUND, "notFound", "No such template.");
    };
    let b = (t.body)();
    let input = RuleIn { name: t.name.to_string(), enabled: true, scope: b.scope, action: b.action, exclude: b.exclude };
    save_rule(&state, &me, None, input, Some(t.id.to_string())).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SettingsIn {
    enabled: Option<bool>,
    trash_first: Option<bool>,
    undo_days: Option<i64>,
    paused_until: Option<i64>,
    folder_defaults: Option<Vec<DefaultIn>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DefaultIn {
    #[serde(default)]
    id: Option<String>,
    name: String,
    #[serde(default = "yes")]
    enabled: bool,
    #[serde(default)]
    exclude: Exclude,
    folder: String,
    action: Action,
}

async fn put_settings(State(state): State<AppState>, headers: HeaderMap, Json(input): Json<SettingsIn>) -> Response {
    let me = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let store = &state.store;
    let policy = policy_for(store, &me.account_id, &me.username).await;
    let mut row = match store.retention_account(&me.account_id).await {
        Ok(r) => r,
        Err(e) => return store_fail(e),
    };
    if let Some(v) = input.enabled {
        row.enabled = v;
    }
    if let Some(v) = input.trash_first {
        row.trash_first = v;
    }
    if let Some(v) = input.undo_days {
        if !(0..=policy.undo_days).contains(&v) {
            return fail(StatusCode::UNPROCESSABLE_ENTITY, "invalidUndoDays", format!("The undo window is 0 to {} days.", policy.undo_days));
        }
        row.undo_days = v;
    }
    if let Some(v) = input.paused_until {
        row.paused_until = v.max(0);
    }
    if let Some(list) = input.folder_defaults {
        if list.len() > 50 {
            return fail(StatusCode::UNPROCESSABLE_ENTITY, "tooMany", "At most 50 folder defaults.");
        }
        let folders = folders_of(&state, &me.account_id).await;
        let mut rows = Vec::new();
        for d in list {
            let body = Body {
                scope: Scope { folders: vec![d.folder], subfolders: true, ..Scope::default() },
                action: d.action,
                exclude: d.exclude,
            };
            if let Err(e) = rr::validate(&d.name, &body, &policy, &folders) {
                return fail(StatusCode::UNPROCESSABLE_ENTITY, e.code, e.detail);
            }
            rows.push(DefaultRow { id: d.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()), name: d.name, enabled: d.enabled, body });
        }
        row.folder_defaults_json = serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into());
    }
    row.updated_at = chrono::Utc::now().timestamp();
    match store.retention_account_put(&row).await {
        Ok(()) => list_rules(State(state), headers).await,
        Err(e) => store_fail(e),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreviewIn {
    /// A rule being edited, evaluated alone. Absent: every rule of the account.
    #[serde(default)]
    rule: Option<RuleIn>,
}

async fn preview(State(state): State<AppState>, headers: HeaderMap, Json(input): Json<PreviewIn>) -> Response {
    let me = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let only = input.rule.map(|r| Body { scope: r.scope, action: r.action, exclude: r.exclude });
    if let Some(b) = &only {
        let policy = policy_for(&state.store, &me.account_id, &me.username).await;
        if let Err(e) = rr::validate("preview", b, &policy, &[]) {
            return fail(StatusCode::UNPROCESSABLE_ENTITY, e.code, e.detail);
        }
    }
    match run_account(&state, None, &me.account_id, &me.username, true, "preview", only).await {
        Ok(s) => no_store(Json(s.json())),
        Err(e) => fail(StatusCode::BAD_GATEWAY, "mailUnavailable", e),
    }
}

async fn run_now(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let me = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    match run_account(&state, None, &me.account_id, &me.username, false, "user", None).await {
        Ok(s) => no_store(Json(s.json())),
        Err(e) => fail(StatusCode::BAD_GATEWAY, "mailUnavailable", e),
    }
}

/// The retention that applies to this person, per category. Other code (the
/// privacy dashboard) reads it through this function; the route returns the same.
pub(crate) async fn effective_for_account(state: &AppState, account_id: &str, username: &str) -> Value {
    let store = &state.store;
    let now = chrono::Utc::now().timestamp();
    let policy = policy_for(store, account_id, username).await;
    let row = store.retention_account(account_id).await.unwrap_or_else(|_| AccountRow::fresh(account_id));
    let rules = store.retention_rules(account_id).await.unwrap_or_default();
    let (acct, p) = account_of(&row, &policy, now);
    let mut mail = Vec::new();
    for (role, days) in &p.mandatory {
        mail.push(json!({"category": format!("folder:{role}"), "days": days, "action": "delete", "source": "administrator"}));
    }
    if p.enabled && acct.enabled {
        for r in rules.iter().filter(|r| r.enabled).filter_map(rule_of) {
            let a = &r.body.action;
            let days = if a.kind == ActionKind::Keep { Value::Null } else { json!(rr::effective_days(a.kind, a.days, &p)) };
            mail.push(json!({"category": "rule", "name": r.name, "days": days, "action": a.kind, "source": "you", "paused": acct.paused}));
        }
    }
    let system = crate::retention_tiers::effective_system(store, account_id, username).await;
    json!({
        "mail": mail,
        "userRules": {
            "available": p.enabled, "enabled": acct.enabled, "paused": acct.paused,
            "rules": rules.len(), "trashFirst": acct.trash_first, "undoDays": p.undo_days,
            "allowPermanentDelete": p.allow_permanent, "lastRunAt": row.last_run_at,
            "allowStripAttachments": p.allow_strip,
        },
        "system": system,
    })
}

async fn effective(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match me(&state, &headers).await {
        Ok(me) => no_store(Json(effective_for_account(&state, &me.account_id, &me.username).await)),
        Err(r) => r,
    }
}

// ─── admin overview ────────────────────────────────────────────────────────────

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/retention/user-rules", Capability::SettingsRead, admin_overview);
}

async fn admin_overview(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SettingsRead)?;
    let since = chrono::Utc::now().timestamp() - 30 * rr::DAY;
    let s = state
        .store
        .retention_rule_stats(since)
        .await
        .map_err(|e| AdminApiError::conflict("retentionStoreUnavailable", e.to_string()))?;
    let global = Who::of("", "");
    let job = state.store.get_setting("retention.user.lastJob").await.ok().flatten().and_then(|s| serde_json::from_str::<Value>(&s).ok());
    Ok(no_store(Json(json!({
        "enabled": setting(&state.store, "retention.user.enabled", &global).await,
        "usersWithRules": s.users_with_rules,
        "rules": s.rules,
        "enabledRules": s.enabled_rules,
        "runs30d": s.runs_since,
        "matched30d": s.matched_since,
        "messagesChanged30d": s.acted_since,
        "lastJob": job,
        "keys": ["retention.user.enabled", "retention.user.minDays", "retention.user.maxDays", "retention.user.maxRules",
                 "retention.user.allowPermanentDelete", "retention.user.undoDays", "retention.user.mandatory",
                 "retention.user.protectedFolders", "retention.user.templates", "retention.user.runForRestricted",
                 "retention.user.intervalHours", "retention.user.batchMessages", "retention.user.maxPerRun", "retention.user.pauseMs",
                 "retention.user.allowStripAttachments", "retention.user.strip.minAgeDays", "retention.user.strip.minAttachmentKb",
                 "retention.user.strip.keepInlineImages", "retention.user.strip.undoDays",
                 "retention.user.calendar.enabled", "retention.user.calendar.minDays", "retention.user.calendar.maxDays",
                 "retention.user.notes.enabled", "retention.user.notes.minDays", "retention.user.notes.maxDays"],
    }))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::retention_rules::{DAY, TRASH_STAMP};
    use std::sync::Mutex;

    fn kept_msg(id: &str, from: &str) -> Msg {
        Msg {
            id: id.into(),
            mailbox_id: "inbox".into(),
            received_at: 1,
            size: 1,
            keywords: vec![],
            from: from.into(),
            subject: "s".into(),
            has_attachment: false,
        }
    }

    #[test]
    fn kept_senders_are_left_out_of_a_plan() {
        let entries: Vec<String> = ["sender:News@Example.org", "list:digest.example.org", "sender: ", "bare@example.org"]
            .map(String::from)
            .to_vec();
        assert_eq!(kept_senders(&entries), vec!["news@example.org"]);
        let decision = || Decision { outcome: Outcome::MarkRead, source: "rule:r".into() };
        let mut plan = Plan::default();
        plan.entries.push((kept_msg("a", "news@example.org"), decision()));
        plan.entries.push((kept_msg("b", "other@example.org"), decision()));
        plan.entries.push((kept_msg("c", "News@example.org"), decision()));
        assert_eq!(drop_kept(&mut plan, &kept_senders(&entries)), 2);
        assert_eq!(plan.entries.iter().map(|(m, _)| m.id.as_str()).collect::<Vec<_>>(), vec!["b"]);
        assert_eq!(drop_kept(&mut plan, &[]), 0);
    }

    /// An in-memory mailbox.
    struct Fake {
        folders: Vec<Folder>,
        msgs: Mutex<Vec<Msg>>,
        calls: Mutex<Vec<String>>,
        fail_destroy: bool,
    }

    impl Fake {
        fn new(msgs: Vec<Msg>) -> Self {
            let f = |id: &str, role: Option<&str>| Folder { id: id.into(), name: id.into(), role: role.map(Into::into), parent_id: None };
            Self {
                folders: vec![f("inbox", Some("inbox")), f("trash", Some("trash")), f("archive", Some("archive")), f("junk", Some("junk")), f("drafts", Some("drafts"))],
                msgs: Mutex::new(msgs),
                calls: Mutex::default(),
                fail_destroy: false,
            }
        }
        fn in_folder(&self, f: &str) -> usize {
            self.msgs.lock().unwrap().iter().filter(|m| m.mailbox_id == f).count()
        }
    }

    #[async_trait]
    impl MailPort for Fake {
        async fn folders(&self) -> Result<Vec<Folder>, String> {
            Ok(self.folders.clone())
        }
        async fn list(&self, folder: &str, before: i64, position: usize, limit: usize) -> Result<Vec<Msg>, String> {
            let mut v: Vec<Msg> = self.msgs.lock().unwrap().iter().filter(|m| m.mailbox_id == folder && m.received_at < before).cloned().collect();
            v.sort_by_key(|m| m.received_at);
            Ok(v.into_iter().skip(position).take(limit).collect())
        }
        async fn move_to(&self, ids: &[String], to: &str, stamp: Option<&str>) -> Result<usize, String> {
            self.calls.lock().unwrap().push(format!("move {}", ids.len()));
            let mut n = 0;
            for m in self.msgs.lock().unwrap().iter_mut().filter(|m| ids.contains(&m.id)) {
                m.mailbox_id = to.to_string();
                if let Some(s) = stamp {
                    m.keywords.push(s.to_string());
                }
                n += 1;
            }
            Ok(n)
        }
        async fn stamp(&self, ids: &[String], k: &str) -> Result<usize, String> {
            let mut n = 0;
            for m in self.msgs.lock().unwrap().iter_mut().filter(|m| ids.contains(&m.id)) {
                if !m.keywords.iter().any(|x| x == k) {
                    m.keywords.push(k.to_string());
                }
                n += 1;
            }
            Ok(n)
        }
        async fn mark_read(&self, ids: &[String]) -> Result<usize, String> {
            self.stamp(ids, "$seen").await
        }
        async fn strip(&self, ids: &[String], p: &StripParams, dry: bool) -> Result<StripReport, String> {
            self.calls.lock().unwrap().push(format!("strip {} dry={dry}", ids.len()));
            let mut report = StripReport::default();
            let mut add = Vec::new();
            for m in self.msgs.lock().unwrap().iter().filter(|m| ids.contains(&m.id)) {
                if m.has_attachment && m.size as u64 >= p.min_bytes && !m.has_keyword("signed") {
                    // The copy loses 90% of its size and keeps flags and date.
                    let mut copy = m.clone();
                    copy.id = format!("{}-s", m.id);
                    copy.size = m.size / 10;
                    copy.has_attachment = false;
                    copy.keywords.push("mw-stripped".into());
                    report.replaced.push((m.id.clone(), if dry { String::new() } else { copy.id.clone() }, (m.size - copy.size) as u64));
                    add.push(copy);
                } else {
                    report.checked.push(m.id.clone());
                }
            }
            if !dry {
                self.msgs.lock().unwrap().extend(add);
            }
            Ok(report)
        }
        async fn destroy(&self, ids: &[String]) -> Result<usize, String> {
            self.calls.lock().unwrap().push(format!("destroy {}", ids.len()));
            if self.fail_destroy {
                return Err("refused".into());
            }
            let mut v = self.msgs.lock().unwrap();
            let before = v.len();
            v.retain(|m| !ids.contains(&m.id));
            Ok(before - v.len())
        }
    }

    const NOW: i64 = 1_800_000_000;

    fn msg(id: &str, mailbox: &str, age: i64, kw: &[&str]) -> Msg {
        Msg {
            id: id.into(),
            mailbox_id: mailbox.into(),
            received_at: NOW - age * DAY,
            size: 1024,
            keywords: kw.iter().map(|s| s.to_string()).collect(),
            from: "a@b.example".into(),
            subject: "s".into(),
            has_attachment: false,
        }
    }

    fn rules() -> Vec<Rule> {
        let r = |id: &str, scope: Scope, kind, days| Rule {
            id: id.into(),
            name: id.into(),
            enabled: true,
            body: Body { scope, action: Action { kind, days, target: None }, exclude: Exclude::default() },
        };
        vec![
            r("news", Scope { labels: vec!["newsletter".into()], ..Scope::default() }, ActionKind::Delete, 30),
            r("old", Scope { folders: vec!["role:inbox".into()], subfolders: false, read: rr::ReadState::Read, ..Scope::default() }, ActionKind::Archive, 180),
        ]
    }

    async fn run(port: &Fake, p: &Policy, dry: bool) -> (Plan, Executed) {
        let rules = rules();
        let acct = Account::default();
        let inputs = Inputs { now: NOW, rules: &rules, defaults: &[], policy: p, acct: &acct };
        let folders = port.folders().await.unwrap();
        let plan = build_plan(port, &folders, &inputs).await;
        let done = if dry { Executed::default() } else { execute(port, &plan, 5000, 2, Duration::ZERO).await };
        (plan, done)
    }

    fn mailbox() -> Fake {
        let mut v = vec![];
        for i in 0..5 {
            v.push(msg(&format!("n{i}"), "inbox", 40 + i, &["newsletter", "$seen"]));
        }
        v.push(msg("young", "inbox", 3, &["newsletter", "$seen"]));
        v.push(msg("flag", "inbox", 90, &["newsletter", "$flagged"]));
        v.push(msg("keep", "inbox", 90, &["newsletter", "keep"]));
        v.push(msg("read200", "inbox", 200, &["$seen"]));
        v.push(msg("unread200", "inbox", 200, &[]));
        v.push(msg("draft", "drafts", 900, &["$seen", "newsletter"]));
        Fake::new(v)
    }

    #[tokio::test]
    async fn dry_run_equals_the_real_run_and_changes_nothing() {
        let port = mailbox();
        let p = Policy::default();
        let (dry_plan, _) = run(&port, &p, true).await;
        assert_eq!(port.in_folder("trash"), 0, "a dry run moves nothing");
        let would = tally(&dry_plan, 5000);
        assert_eq!((would.trashed, would.moved), (5, 1));
        let (_, done) = run(&port, &p, false).await;
        assert_eq!(done.counts, would, "the run did what the preview said");
        assert_eq!((port.in_folder("trash"), port.in_folder("archive")), (5, 1));
        assert_eq!(done.errors, 0);
        // Batches of 2: three move calls for trash plus one for archive.
        assert!(port.calls.lock().unwrap().len() >= 4);
    }

    #[tokio::test]
    async fn a_second_run_finds_nothing() {
        let port = mailbox();
        let p = Policy::default();
        run(&port, &p, false).await;
        let (plan, done) = run(&port, &p, false).await;
        assert!(plan.entries.is_empty(), "{:?}", plan.entries.iter().map(|(m, d)| (&m.id, &d.source)).collect::<Vec<_>>());
        assert_eq!(done.counts.total(), 0);
    }

    #[tokio::test]
    async fn excluded_and_protected_mail_is_never_touched() {
        let port = mailbox();
        run(&port, &Policy::default(), false).await;
        let msgs = port.msgs.lock().unwrap();
        for id in ["flag", "keep", "young", "unread200", "draft"] {
            let m = msgs.iter().find(|m| m.id == id).unwrap();
            assert!(m.mailbox_id == "inbox" || m.mailbox_id == "drafts", "{id} stayed");
        }
    }

    #[tokio::test]
    async fn trash_then_purge_after_the_undo_window() {
        let port = mailbox();
        let p = Policy { undo_days: 7, ..Policy::default() };
        run(&port, &p, false).await;
        assert_eq!(port.in_folder("trash"), 5);
        // Inside the window nothing is purged.
        let (plan, _) = run(&port, &p, false).await;
        assert!(plan.entries.is_empty());
        // Eight days later the stamped messages are purged, and only they.
        let later = NOW + 8 * DAY;
        let rules = rules();
        let acct = Account::default();
        let inputs = Inputs { now: later, rules: &rules, defaults: &[], policy: &p, acct: &acct };
        let folders = port.folders().await.unwrap();
        let plan = build_plan(&port, &folders, &inputs).await;
        let purged = plan.entries.iter().filter(|(_, d)| d.outcome == Outcome::Destroy).count();
        assert_eq!(purged, 5);
        assert!(plan.entries.iter().filter(|(m, _)| m.mailbox_id == "trash").all(|(m, _)| m.keywords.iter().any(|k| k.starts_with(TRASH_STAMP))));
        let done = execute(&port, &plan, 5000, 200, Duration::ZERO).await;
        assert_eq!(done.counts.purged, 5);
        assert_eq!(port.in_folder("trash"), 0);
    }

    #[tokio::test]
    async fn the_per_run_cap_defers_the_rest_and_errors_are_counted() {
        let port = mailbox();
        let p = Policy::default();
        let rules = rules();
        let acct = Account::default();
        let inputs = Inputs { now: NOW, rules: &rules, defaults: &[], policy: &p, acct: &acct };
        let folders = port.folders().await.unwrap();
        let plan = build_plan(&port, &folders, &inputs).await;
        let total = plan.entries.len();
        let done = execute(&port, &plan, 2, 200, Duration::ZERO).await;
        assert_eq!(done.counts.total(), 2);
        let (plan2, _) = run(&port, &p, true).await;
        assert_eq!(plan2.entries.len(), total - 2, "what was left is planned again");

        let mut failing = Fake::new(vec![msg("a", "trash", 100, &[&format!("{TRASH_STAMP}{}", NOW.div_euclid(DAY) - 30)])]);
        failing.fail_destroy = true;
        let (_, done) = run(&failing, &p, false).await;
        assert_eq!((done.counts.purged, done.errors), (0, 1));
        assert_eq!(failing.in_folder("trash"), 1, "a refused delete leaves the message");
    }

    #[tokio::test]
    async fn mandatory_rules_purge_junk_without_any_user_rule() {
        let port = Fake::new(vec![msg("j1", "junk", 40, &[]), msg("j2", "junk", 5, &[]), msg("i1", "inbox", 400, &["$seen"])]);
        let p = Policy { mandatory: vec![("junk".into(), 30)], ..Policy::default() };
        let acct = Account::default();
        let inputs = Inputs { now: NOW, rules: &[], defaults: &[], policy: &p, acct: &acct };
        let folders = port.folders().await.unwrap();
        let plan = build_plan(&port, &folders, &inputs).await;
        let done = execute(&port, &plan, 100, 50, Duration::ZERO).await;
        assert_eq!(done.counts.purged, 1);
        assert_eq!((port.in_folder("junk"), port.in_folder("inbox")), (1, 1));
    }


    fn big(id: &str, age: i64, size: i64, kw: &[&str]) -> Msg {
        Msg { size, has_attachment: true, ..msg(id, "inbox", age, kw) }
    }

    fn strip_rules() -> Vec<Rule> {
        vec![Rule {
            id: "strip".into(),
            name: "strip".into(),
            enabled: true,
            body: Body { scope: Scope::default(), action: Action { kind: ActionKind::StripAttachments, days: 120, target: None }, exclude: Exclude::default() },
        }]
    }

    async fn strip_run(port: &Fake, p: &Policy, rules: &[Rule], dry: bool) -> (Plan, Executed) {
        let acct = Account::default();
        let inputs = Inputs { now: NOW, rules, defaults: &[], policy: p, acct: &acct };
        let folders = port.folders().await.unwrap();
        let plan = build_plan(port, &folders, &inputs).await;
        let done = if dry { Executed::default() } else { execute(port, &plan, 5000, 50, Duration::ZERO).await };
        (plan, done)
    }

    fn strip_mailbox() -> Fake {
        Fake::new(vec![
            big("old-big", 200, 2_000_000, &["$seen", "$flagged-not"]),
            big("old-big2", 300, 5_000_000, &["$seen"]),
            big("old-signed", 200, 150_000, &["$seen", "signed"]),
            big("young-big", 30, 3_000_000, &["$seen"]),
            big("done", 400, 3_000_000, &["$seen", "mw-stripped"]),
            big("checked", 400, 3_000_000, &["$seen", "mw-strip-checked"]),
            msg("plain", "inbox", 400, &["$seen"]),
        ])
    }

    #[tokio::test]
    async fn strip_is_inert_while_the_administrator_has_it_off() {
        let port = strip_mailbox();
        let (plan, done) = strip_run(&port, &Policy::default(), &strip_rules(), false).await;
        assert!(plan.entries.is_empty());
        assert_eq!(done.counts.total(), 0);
        assert!(port.calls.lock().unwrap().is_empty(), "no mail was opened");
    }

    #[tokio::test]
    async fn strip_replaces_big_old_attachments_and_holds_the_originals_in_trash() {
        let port = strip_mailbox();
        let p = Policy { allow_strip: true, strip_min_attachment_kb: 100, undo_days: 7, ..Policy::default() };
        let (plan, done) = strip_run(&port, &p, &strip_rules(), false).await;
        assert_eq!(plan.entries.len(), 3, "old-big, old-big2 and old-signed are planned; young, done, checked and plain are not");
        assert_eq!(done.errors, 0);
        assert_eq!(done.counts.stripped, 2);
        assert_eq!(done.counts.strip_skipped, 1, "the signed one was looked at and left alone");
        assert_eq!(done.counts.reclaimed_bytes, (2_000_000 - 200_000) + (5_000_000 - 500_000));
        let msgs = port.msgs.lock().unwrap();
        // Copies in the inbox keep their flags and the marker; originals wait in Trash.
        let copy = msgs.iter().find(|m| m.id == "old-big-s").unwrap();
        assert_eq!(copy.mailbox_id, "inbox");
        assert_eq!(copy.received_at, NOW - 200 * DAY);
        assert!(copy.keywords.iter().any(|k| k == "$seen") && copy.keywords.iter().any(|k| k == "mw-stripped"));
        let orig = msgs.iter().find(|m| m.id == "old-big").unwrap();
        assert_eq!(orig.mailbox_id, "trash");
        assert!(orig.keywords.iter().any(|k| k.starts_with(TRASH_STAMP)));
        assert!(msgs.iter().find(|m| m.id == "old-signed").unwrap().keywords.iter().any(|k| k == "mw-strip-checked"));
        drop(msgs);
        // A second run finds nothing: the copies carry the marker, the small one is checked.
        let (again, done2) = strip_run(&port, &p, &strip_rules(), false).await;
        assert!(again.entries.is_empty(), "{:?}", again.entries.iter().map(|(m, _)| &m.id).collect::<Vec<_>>());
        assert_eq!(done2.counts.total(), 0);
    }

    #[tokio::test]
    async fn strip_with_no_undo_window_deletes_the_original_and_the_window_is_its_own() {
        let port = strip_mailbox();
        let p = Policy { allow_strip: true, strip_undo_days: 0, ..Policy::default() };
        let (_, done) = strip_run(&port, &p, &strip_rules(), false).await;
        assert_eq!(done.counts.stripped, 2);
        assert_eq!(port.in_folder("trash"), 0);
        assert!(port.msgs.lock().unwrap().iter().all(|m| m.id != "old-big"));

        // With a 30 day strip window and a 7 day Trash window, the purge date is 30 days out.
        let port = strip_mailbox();
        let p = Policy { allow_strip: true, strip_undo_days: 30, undo_days: 7, ..Policy::default() };
        strip_run(&port, &p, &strip_rules(), false).await;
        let stamp = port.msgs.lock().unwrap().iter().find(|m| m.id == "old-big").unwrap().trashed_day().unwrap();
        assert_eq!(stamp + p.undo_days, NOW.div_euclid(DAY) + 30);
    }

    #[tokio::test]
    async fn strip_dry_run_counts_messages_and_bytes_without_changing_anything() {
        let port = strip_mailbox();
        let p = Policy { allow_strip: true, ..Policy::default() };
        let (plan, _) = strip_run(&port, &p, &strip_rules(), true).await;
        let mut s = Summary::default();
        s.would = tally(&plan, 5000);
        verify_strips(&port, &plan, 5000, &mut s).await;
        assert_eq!(s.would.stripped, 2, "the signed message does not qualify once opened");
        assert_eq!(s.would.reclaimed_bytes, 1_800_000 + 4_500_000);
        assert!(!s.strip_estimate);
        assert_eq!(port.msgs.lock().unwrap().len(), 7, "nothing was created");
        assert_eq!(port.in_folder("trash"), 0);
    }

    #[tokio::test]
    async fn policy_reads_the_strip_settings_and_defaults_to_off() {
        let dir = std::env::temp_dir().join(format!("mw-strip-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let s = Store::open(dir.join("s.sqlite").to_str().unwrap(), mw_store::ServerKey::from_bytes(&[5u8; 32]).unwrap()).await.unwrap();
        let off = policy_for(&s, "a", "a@example.test").await;
        assert!(!off.allow_strip, "disabled by default");
        assert_eq!((off.strip_min_age_days, off.strip_min_attachment_kb, off.strip_keep_inline, off.strip_undo_days), (90, 100, true, -1));
        assert_eq!(off.strip_undo(), off.undo_days, "-1 follows the retention undo window");
        for (key, value) in [("retention.user.allowStripAttachments", "true"), ("retention.user.strip.minAgeDays", "120"), ("retention.user.strip.undoDays", "3")] {
            s.put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                key: key.into(),
                global_value: Some(value.into()),
                allow_user_override: false,
                global_locked: true,
                updated_by: "test".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        }
        let on = policy_for(&s, "a", "a@example.test").await;
        assert!(on.allow_strip);
        assert_eq!((on.strip_min_age_days, on.strip_undo()), (120, 3));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn strip_days_are_raised_to_the_servers_floor() {
        let p = Policy { allow_strip: true, strip_min_age_days: 90, ..Policy::default() };
        assert_eq!(rr::effective_days(ActionKind::StripAttachments, 10, &p), 90);
        assert_eq!(rr::effective_days(ActionKind::StripAttachments, 400, &p), 400);
    }

    #[test]
    fn account_undo_window_never_exceeds_the_servers() {
        let p = Policy { undo_days: 14, allow_permanent: true, ..Policy::default() };
        let mut row = AccountRow::fresh("a");
        assert_eq!(account_of(&row, &p, 0).1.undo_days, 14, "unset uses the server value");
        row.undo_days = 3;
        assert_eq!(account_of(&row, &p, 0).1.undo_days, 3);
        row.undo_days = 90;
        assert_eq!(account_of(&row, &p, 0).1.undo_days, 14);
        row.trash_first = false;
        assert!(!account_of(&row, &p, 0).0.trash_first);
        let strict = Policy { allow_permanent: false, ..p };
        assert!(account_of(&row, &strict, 0).0.trash_first, "the server can force the undo window");
    }
}
