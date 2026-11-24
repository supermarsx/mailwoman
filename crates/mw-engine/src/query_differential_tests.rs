//! t50hp-e31 (design t50hp-e29 part 11): the `Email/query` differential test.
//!
//! Every answer `Email/query` gives is compared with a brute-force oracle
//! computed from the current SQL rows (folder, flags, `message_meta`) and from
//! the content each message was generated with (words, sender, attachment,
//! calendar). Nothing in the oracle reads the search index or the projection
//! tables, so it is independent of both query paths.
//!
//! Corpus: two accounts with the same folder names, the same message bodies
//! and the same flags, pins and follow-ups. Every query runs as account A
//! (and a subset as account B); an id or a count of the other account in an
//! answer fails the test.
//!
//! Matrix: every filter field alone, every pair of a base set, text-operator
//! forms (OR / NOT / `-`), each with several windows (position 0 or 7, an
//! anchor with offset -1, limit 0, 1, 50 or none, calculateTotal on and off);
//! the singles and text forms also under every sort property, ascending and
//! descending, and two compound sorts. Between rounds, random flag, label, pin,
//! follow-up, move, destroy and ingest operations change both accounts.
//!
//! Comparison: ids, position, total (present exactly when asked) and
//! anchorNotFound. Where primary sort keys differ the order must be equal;
//! within an equal primary key a non-compound sort is compared as a set,
//! because the index breaks ties by document address (which a rewrite moves)
//! and the SQL folder list by uid. A compound sort breaks ties by stable id in
//! both and is compared exactly.
//!
//! Modelled pre-existing behaviour (not a difference between the paths): a user
//! keyword whose name has a derived prefix (`$list:`, `$supersedes:`,
//! `$mw-mention:`, `$priority-high/low`) stays in the index once set, because
//! `index_publication::patch_current` keeps every derived-prefix value of the
//! stored document; only a rebuild from source drops it. The oracle therefore
//! treats such a keyword as present from the first time it was set until the
//! message is destroyed. Decision P2 (union of index and SQL) keeps this.
//!
//! Accepted differences (declared up front, design part 11), none of which this
//! corpus can reach, so none is tolerated by the comparison:
//! 1. an SQL-only route reports an exact total above `mw_search::MAX_HITS`,
//!    where the index path withholds it (`mailwomanCannotCalculateTotal`);
//! 2. Focus takes its top 20 within the SQL `scanned` set (Focus is not an
//!    `Email/query` and is not exercised here).
use super::*;
use crate::account::MailSubmitter;
use crate::backend::*;
use async_trait::async_trait;
use mw_search::{Clause, Expr, TextField};
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, NewAccount, SavedSearchRow, ServerKey, Store,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// Accepted differences between the query paths (see the module note). The
/// comparison below tolerates neither; they are listed so a reviewer can see
/// that the test was written knowing them.
pub(crate) const ACCEPTED_DIFFERENCES: [&str; 2] = [
    "SQL-only route: exact total above MAX_HITS where the index withholds it",
    "Focus: top 20 within the SQL scanned set",
];

#[derive(Default)]
struct Remote {
    next_uid: AtomicU32,
}
#[async_trait]
impl AccountBackend for Remote {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![])
    }
    async fn sync_mailbox(&self, _: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: vec![],
            flag_changes: vec![],
            removed: vec![],
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, refs: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        let uids = refs
            .iter()
            .map(|_| self.next_uid.fetch_add(1, Ordering::SeqCst))
            .collect();
        Ok(MoveOutcome::Uidplus {
            uidvalidity: 1,
            uids,
        })
    }
    async fn expunge_messages(&self, _: &[MessageRef]) -> Result<()> {
        Ok(())
    }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("fixture".into()))
    }
}
struct NoSend;
#[async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        panic!("not sending")
    }
}

/// SplitMix64: deterministic, seedable, no dependency.
#[derive(Clone)]
pub(crate) struct Rng(u64);
impl Rng {
    pub(crate) fn new(seed: u64) -> Self {
        Rng(seed)
    }
    pub(crate) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub(crate) fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    pub(crate) fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

const WORDS: [&str; 6] = ["alpha", "bravo", "charlie", "delta", "echo", "foxtrot"];
/// 2026-07-01T00:00:00Z.
const BASE: i64 = 1_782_864_000;
pub(crate) const FOLDERS: [&str; 3] = ["INBOX", "Archive", "Work"];
const LABELS: [&str; 4] = ["work", "home", "$list:x", "mw-followup-completed"];
const FOLLOW_UP_AT: &str = "2026-09-01T00:00:00Z";

/// What message `n` was generated with. Bodies are identical across accounts.
#[derive(Clone, Debug)]
struct Content {
    from: String,
    to: String,
    cc: Option<String>,
    subject: String,
    body: String,
    filename: Option<String>,
    calendar_end: Option<i64>,
    sent_at: i64,
    /// From Email/get after ingest (a content property, not the index).
    has_attachment: bool,
}

fn rfc3339(secs: i64) -> String {
    chrono::DateTime::from_timestamp(secs, 0)
        .expect("in range")
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn content(n: u32) -> Content {
    let mut r = Rng::new(u64::from(n) * 7919 + 13);
    let word = |r: &mut Rng| WORDS[r.below(WORDS.len() as u64) as usize];
    let subject = format!("{} {}", word(&mut r), word(&mut r));
    let mut body = format!("{} {} {}", word(&mut r), word(&mut r), word(&mut r));
    for _ in 0..(n % 7) * 60 {
        body.push_str(" pad");
    }
    Content {
        from: format!(
            "s{}@{}",
            n % 5,
            if n % 3 == 0 {
                "other.invalid"
            } else {
                "fixture.invalid"
            }
        ),
        to: format!("o{}@fixture.invalid", n % 2),
        cc: (n % 4 == 1).then(|| format!("audit{}@fixture.invalid", n % 3)),
        subject,
        body,
        filename: (n % 5 == 2).then(|| format!("report{}.pdf", n % 3)),
        calendar_end: (n % 9 == 4).then(|| BASE + i64::from(n) * 86_400),
        sent_at: BASE - 3_600 * i64::from((n * 37) % 50),
        has_attachment: false,
    }
}

fn raw_message(n: u32, tag: &str, c: &Content) -> String {
    let mut head = format!("From: {}\r\nTo: {}\r\n", c.from, c.to);
    if let Some(cc) = &c.cc {
        head.push_str(&format!("Cc: {cc}\r\n"));
    }
    let date = chrono::DateTime::from_timestamp(c.sent_at, 0)
        .expect("date")
        .to_rfc2822();
    head.push_str(&format!(
        "Subject: {}\r\nDate: {date}\r\nMessage-ID: <d{n}.{tag}@fixture.invalid>\r\nMIME-Version: 1.0\r\n",
        c.subject
    ));
    if c.filename.is_none() && c.calendar_end.is_none() {
        return format!("{head}Content-Type: text/plain\r\n\r\n{}\r\n", c.body);
    }
    let b = format!("b{n}");
    let mut out = format!(
        "{head}Content-Type: multipart/mixed; boundary=\"{b}\"\r\n\r\n--{b}\r\nContent-Type: text/plain\r\n\r\n{}\r\n",
        c.body
    );
    if let Some(f) = &c.filename {
        out.push_str(&format!(
            "--{b}\r\nContent-Type: application/pdf; name=\"{f}\"\r\nContent-Disposition: attachment; filename=\"{f}\"\r\nContent-Transfer-Encoding: base64\r\n\r\nJVBERi0xLjQK\r\n"
        ));
    }
    if let Some(end) = c.calendar_end {
        let ics = |s: i64| {
            chrono::DateTime::from_timestamp(s, 0)
                .expect("date")
                .format("%Y%m%dT%H%M%SZ")
                .to_string()
        };
        out.push_str(&format!(
            "--{b}\r\nContent-Type: text/calendar; method=REQUEST\r\n\r\nBEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//e31//EN\r\nMETHOD:REQUEST\r\nBEGIN:VEVENT\r\nUID:e31-{n}@fixture.invalid\r\nDTSTAMP:20260701T000000Z\r\nDTSTART:{}\r\nDTEND:{}\r\nSUMMARY:meeting\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
            ics(end - 3_600),
            ics(end)
        ));
    }
    out.push_str(&format!("--{b}--\r\n"));
    out
}

/// Initial flags of message `n` (the same for both accounts).
fn initial_flags(n: u32) -> (Vec<Flag>, bool, bool) {
    let mut r = Rng::new(u64::from(n) * 31 + 7);
    let mut flags = Vec::new();
    if r.chance(70) {
        flags.push(Flag::Seen);
    }
    if r.chance(15) {
        flags.push(Flag::Flagged);
    }
    if r.chance(10) {
        flags.push(Flag::Answered);
    }
    for (label, pct) in LABELS.iter().zip([30, 20, 10, 8]) {
        if r.chance(pct) {
            flags.push(Flag::Keyword((*label).to_owned()));
        }
    }
    let pinned = r.chance(12);
    let follow_up = r.chance(12);
    (flags, pinned, follow_up)
}

pub(crate) struct Account {
    pub(crate) id: String,
    /// Folder name -> id, in [`FOLDERS`] order.
    pub(crate) folders: Vec<(String, String)>,
    pub(crate) saved: String,
    tag: &'static str,
}

impl Account {
    pub(crate) fn folder(&self, name: &str) -> &str {
        &self
            .folders
            .iter()
            .find(|(n, _)| n == name)
            .expect("folder")
            .1
    }
}

pub(crate) struct World {
    pub(crate) engine: Engine,
    pub(crate) a: Account,
    pub(crate) b: Account,
    content: std::sync::Mutex<BTreeMap<String, Content>>,
    /// Derived-prefix keywords each message has carried since its ingest
    /// (see the module note: the index keeps them).
    sticky: std::sync::Mutex<BTreeMap<String, BTreeSet<String>>>,
    next: AtomicU32,
}

/// patch_current's derived set (index_publication.rs).
pub(crate) fn derived_name(k: &str) -> bool {
    k.starts_with("$mw-mention:")
        || k.starts_with("$list:")
        || k.starts_with("$supersedes:")
        || k == mw_search::KW_PRIORITY_HIGH
        || k == mw_search::KW_PRIORITY_LOW
}

async fn make_account(store: &Store, tag: &'static str) -> Account {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let login = format!(
        "e31{tag}-{}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst),
        chrono::Utc::now().timestamp_millis()
    );
    let id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "fixture.invalid",
                port: 993,
                tls: "implicit",
                username: &login,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: login.clone(),
                password: "x".into(),
            },
        )
        .await
        .unwrap();
    let mut folders = Vec::new();
    for (name, role) in FOLDERS.iter().zip([Some("inbox"), Some("archive"), None]) {
        let mb = store
            .upsert_mailbox(&MailboxUpsert {
                account_id: &id,
                name,
                role,
                uidvalidity: 1,
                uidnext: 1,
                highestmodseq: 0,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        folders.push(((*name).to_owned(), mb));
    }
    // A saved-search folder with the same name and query in both accounts.
    let owner = store
        .native_personal_subject(&id)
        .await
        .unwrap()
        .expect("owner");
    let saved = format!("e31-saved-{login}");
    assert!(
        store
            .upsert_personal_saved_search(&SavedSearchRow {
                id: saved.clone(),
                user: owner,
                name: "Work label".into(),
                query_json: r#"{"hasKeyword":"work"}"#.into(),
                as_folder: true,
            })
            .await
            .unwrap()
    );
    Account {
        id,
        folders,
        saved,
        tag,
    }
}

impl World {
    pub(crate) async fn new(store: Store, messages: u32) -> World {
        let a = make_account(&store, "a").await;
        let b = make_account(&store, "b").await;
        let engine = Engine::new(store);
        for acct in [&a, &b] {
            engine.register_backend(
                acct.id.clone(),
                AccountRuntime::new(
                    Arc::new(Remote {
                        next_uid: AtomicU32::new(100_000),
                    }),
                    Arc::new(NoSend),
                    "o0@fixture.invalid",
                ),
            );
        }
        let w = World {
            engine,
            a,
            b,
            content: Default::default(),
            sticky: Default::default(),
            next: AtomicU32::new(1),
        };
        for _ in 0..messages {
            w.ingest_pair().await;
        }
        w
    }

    pub(crate) fn account(&self, which: char) -> &Account {
        if which == 'a' { &self.a } else { &self.b }
    }

    pub(crate) async fn send(&self, account: &str, calls: Value) -> Value {
        self.engine
            .handle_jmap(account, &json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":calls}))
            .await
    }

    /// One generated message into both accounts: same folder, body, flags,
    /// pin and follow-up.
    async fn ingest_pair(&self) {
        let n = self.next.fetch_add(1, Ordering::SeqCst);
        let folder = FOLDERS[match n % 6 {
            0..=2 => 0,
            3 | 4 => 1,
            _ => 2,
        }];
        for acct in [&self.a, &self.b] {
            self.ingest_into(acct, n, folder).await;
        }
    }

    async fn ingest_into(&self, acct: &Account, n: u32, folder: &str) -> String {
        let c = content(n);
        let (flags, pinned, follow_up) = initial_flags(n);
        let derived: BTreeSet<String> = flags
            .iter()
            .filter_map(|f| match f {
                Flag::Keyword(k) if derived_name(k) => Some(k.clone()),
                _ => None,
            })
            .collect();
        let message = RawMessage {
            message_ref: MessageRef::Imap {
                mailbox: RawMailboxRef {
                    name: folder.to_owned(),
                    uidvalidity: 1,
                },
                uidvalidity: 1,
                uid: n,
            },
            raw: raw_message(n, acct.tag, &c).into_bytes(),
            flags,
            // Two messages per hour: equal-second ties on the date sort.
            internaldate: Some(rfc3339(BASE + i64::from(n / 2) * 3_600)),
        };
        let id = self
            .engine
            .ingest(&acct.id, acct.folder(folder), &message)
            .await
            .unwrap();
        if pinned || follow_up {
            let mut patch = serde_json::Map::new();
            if pinned {
                patch.insert("pinned".into(), json!(true));
            }
            if follow_up {
                patch.insert("followUpAt".into(), json!(FOLLOW_UP_AT));
            }
            self.update(&acct.id, &id, Value::Object(patch)).await;
        }
        let got = self
            .send(&acct.id, json!([["Email/get", {"accountId": acct.id, "ids": [id], "properties": ["hasAttachment"]}, "g"]]))
            .await;
        let has_attachment = got["methodResponses"][0][1]["list"][0]["hasAttachment"]
            .as_bool()
            .expect("hasAttachment");
        self.content.lock().unwrap().insert(
            id.clone(),
            Content {
                has_attachment,
                ..c
            },
        );
        self.sticky.lock().unwrap().insert(id.clone(), derived);
        id
    }

    pub(crate) async fn update(&self, account: &str, id: &str, patch: Value) {
        let mut update = serde_json::Map::new();
        update.insert(id.to_owned(), patch);
        let r = self
            .send(
                account,
                json!([["Email/set", {"accountId": account, "update": update}, "u"]]),
            )
            .await;
        assert!(
            r["methodResponses"][0][1]["updated"].get(id).is_some(),
            "update {id}: {r}"
        );
    }

    /// The current rows of an account, from SQL.
    pub(crate) async fn rows(&self, acct: &Account) -> Vec<Row> {
        let store = self.engine.store();
        let mut out = Vec::new();
        let content = self.content.lock().unwrap().clone();
        let sticky = self.sticky.lock().unwrap().clone();
        for (_, mb) in &acct.folders {
            let ids = store.list_message_ids(mb, i64::MAX, 0).await.unwrap();
            let messages = store.get_messages(&ids).await.unwrap();
            let metas = store.get_message_metas(&ids).await.unwrap();
            for (m, meta) in messages.into_iter().zip(metas) {
                let m = m.expect("listed message");
                assert_eq!(m.account_id, acct.id);
                let meta = meta.unwrap_or_default();
                let mut keywords: BTreeSet<String> = crate::mapping::flags_to_keywords(
                    &crate::mapping::flags_from_json(&m.flags_json),
                )
                .into_keys()
                .collect();
                if meta.follow_up_at.is_some() {
                    keywords.insert(mw_search::KW_FOLLOWUP_ACTIVE.to_owned());
                }
                keywords.extend(sticky.get(&m.stable_id).into_iter().flatten().cloned());
                let date = m
                    .internaldate
                    .as_deref()
                    .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
                    .map(|d| d.timestamp())
                    .unwrap_or(0);
                out.push(Row {
                    c: content
                        .get(&m.stable_id)
                        .cloned()
                        .expect("generated content"),
                    id: m.stable_id,
                    mailbox: m.mailbox_id,
                    keywords,
                    pinned: meta.pinned,
                    date,
                    size: m.size,
                });
            }
        }
        out
    }

    /// One random operation on `acct`: flag, label, pin, follow-up, move,
    /// destroy or ingest.
    pub(crate) async fn mutate(&self, acct: &Account, rng: &mut Rng) -> String {
        let rows = self.rows(acct).await;
        let op = rng.below(9);
        if rows.is_empty() || op == 8 {
            let n = self.next.fetch_add(1, Ordering::SeqCst);
            let folder = FOLDERS[rng.below(3) as usize];
            let id = self.ingest_into(acct, n, folder).await;
            return format!("ingest {id} into {folder}");
        }
        let row = &rows[rng.below(rows.len() as u64) as usize];
        let id = row.id.clone();
        let toggle = |k: &str| {
            let on = !row.keywords.contains(k) || derived_name(k) && rng_flip(&id);
            json!({ format!("keywords/{k}"): if on { json!(true) } else { Value::Null } })
        };
        let (what, patch) = match op {
            0 => ("$seen", toggle("$seen")),
            1 => ("$flagged", toggle("$flagged")),
            2 => {
                let label = LABELS[rng.below(LABELS.len() as u64) as usize];
                (label, toggle(label))
            }
            3 => ("pinned", json!({"pinned": !row.pinned})),
            4 => {
                let on = !row.keywords.contains(mw_search::KW_FOLLOWUP_ACTIVE);
                (
                    "followUpAt",
                    json!({"followUpAt": if on { json!(FOLLOW_UP_AT) } else { Value::Null }}),
                )
            }
            5 | 6 => {
                let others: Vec<&(String, String)> = acct
                    .folders
                    .iter()
                    .filter(|(_, mb)| *mb != row.mailbox)
                    .collect();
                let target = others[rng.below(others.len() as u64) as usize];
                let r = self
                    .send(&acct.id, json!([["Email/set", {"accountId": acct.id, "update": {id.clone(): {"mailboxIds": {target.1.clone(): true}}}}, "m"]]))
                    .await;
                assert!(
                    r["methodResponses"][0][1]["updated"].get(&id).is_some(),
                    "move: {r}"
                );
                return format!("move {id} to {}", target.0);
            }
            _ => {
                let r = self
                    .send(&acct.id, json!([["Email/set", {"accountId": acct.id, "destroy": [id.clone()]}, "d"]]))
                    .await;
                assert!(
                    r["methodResponses"][0][1]["destroyed"]
                        .as_array()
                        .is_some_and(|d| !d.is_empty()),
                    "destroy: {r}"
                );
                self.content.lock().unwrap().remove(&id);
                self.sticky.lock().unwrap().remove(&id);
                return format!("destroy {id}");
            }
        };
        self.update(&acct.id, &id, patch.clone()).await;
        if derived_name(what)
            && patch
                .get(format!("keywords/{what}"))
                .is_some_and(|v| v.as_bool() == Some(true))
        {
            self.sticky
                .lock()
                .unwrap()
                .entry(id.clone())
                .or_default()
                .insert(what.to_owned());
        }
        format!("{what} {patch} on {id}")
    }
}

/// For a sticky derived-prefix keyword the oracle row always shows it, so a
/// plain toggle would only ever remove it from SQL; flip a coin from the id so
/// both directions are exercised.
fn rng_flip(id: &str) -> bool {
    id.bytes().fold(0u8, |a, b| a.wrapping_add(b)) % 2 == 0
}

/// A message as the oracle sees it.
#[derive(Clone, Debug)]
pub(crate) struct Row {
    pub(crate) id: String,
    pub(crate) mailbox: String,
    /// JMAP keywords from flags_json, plus `$mw-followup-active` when a
    /// follow-up is set (the index's derived marker).
    pub(crate) keywords: BTreeSet<String>,
    pub(crate) pinned: bool,
    pub(crate) date: i64,
    pub(crate) size: u64,
    c: Content,
}

/// mw-search's tokenizer (split on non-alphanumeric, lowercase, <= 40 bytes).
fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty() && t.len() <= 40)
        .map(str::to_lowercase)
        .collect()
}

impl Row {
    fn field_values(&self, field: TextField) -> Vec<String> {
        let c = &self.c;
        match field {
            TextField::From => vec![c.from.clone()],
            TextField::To => vec![c.to.clone()],
            TextField::Cc => vec![c.cc.clone().unwrap_or_default()],
            TextField::Subject => vec![c.subject.clone()],
            TextField::Body => vec![c.body.clone()],
            TextField::Filename => c.filename.iter().cloned().collect(),
            TextField::All => [
                TextField::From,
                TextField::To,
                TextField::Cc,
                TextField::Subject,
                TextField::Body,
                TextField::Filename,
            ]
            .into_iter()
            .flat_map(|f| self.field_values(f))
            .collect(),
        }
    }

    fn text(&self, field: TextField, value: &str, phrase: bool) -> bool {
        let wanted = tokenize(value);
        if wanted.is_empty() {
            return false;
        }
        let one = |f: TextField| {
            let values: Vec<Vec<String>> =
                self.field_values(f).iter().map(|v| tokenize(v)).collect();
            if phrase && wanted.len() > 1 {
                values
                    .iter()
                    .any(|toks| toks.windows(wanted.len()).any(|w| w == wanted.as_slice()))
            } else {
                let all: BTreeSet<&String> = values.iter().flatten().collect();
                wanted.iter().all(|t| all.contains(t))
            }
        };
        match field {
            TextField::All => [
                TextField::From,
                TextField::To,
                TextField::Cc,
                TextField::Subject,
                TextField::Body,
                TextField::Filename,
            ]
            .into_iter()
            .any(one),
            f => one(f),
        }
    }

    fn wildcard(&self, field: TextField, value: &str) -> bool {
        let pattern: Vec<char> = value.to_lowercase().chars().collect();
        fn glob(p: &[char], t: &[char]) -> bool {
            match p.split_first() {
                None => t.is_empty(),
                Some(('*', rest)) => (0..=t.len()).any(|i| glob(rest, &t[i..])),
                Some((c, rest)) => t.first() == Some(c) && glob(rest, &t[1..]),
            }
        }
        self.field_values(field)
            .iter()
            .flat_map(|v| tokenize(v))
            .any(|tok| glob(&pattern, &tok.chars().collect::<Vec<_>>()))
    }

    fn follow_up(&self) -> bool {
        (self.keywords.contains(mw_search::KW_FOLLOWUP_ACTIVE)
            || self.keywords.contains("$flagged"))
            && !self.keywords.contains(mw_search::KW_FOLLOWUP_COMPLETED)
    }

    /// The parsed text operators.
    fn eval(&self, e: &Expr) -> bool {
        match e {
            Expr::All => true,
            Expr::And(items) => items.iter().all(|i| self.eval(i)),
            Expr::Or(items) => items.iter().any(|i| self.eval(i)),
            Expr::Not(inner) => !self.eval(inner),
            Expr::Clause(c) => match c {
                Clause::Text {
                    field,
                    value,
                    phrase,
                } => self.text(*field, value, *phrase),
                Clause::Wildcard { field, value } => self.wildcard(*field, value),
                Clause::Fuzzy { .. } => panic!("the oracle does not model fuzzy terms"),
                Clause::FromDomain(d) => match mw_search::normalize_sender_domain(d) {
                    Some(d) => self.c.from.rsplit_once('@').map(|(_, x)| x) == Some(d.as_str()),
                    None => false,
                },
                Clause::Keyword(k) => self.keywords.contains(k),
                Clause::NotKeyword(k) => !self.keywords.contains(k),
                Clause::Mailbox(m) => &self.mailbox == m,
                Clause::HasAttachment(b) => self.c.has_attachment == *b,
                Clause::Pinned(b) => self.pinned == *b,
                Clause::FollowUp => self.follow_up(),
                Clause::Mention(_) => panic!("the oracle does not model mentions"),
                Clause::HasCalendar(b) => self.c.calendar_end.is_some() == *b,
                Clause::Bulk(b) => !*b,
                Clause::CalendarEndAfter(at) => self.c.calendar_end.is_some_and(|end| end >= *at),
                Clause::DateRange { after, before } => {
                    (after.is_some() || before.is_some())
                        && after.is_none_or(|a| self.date >= a)
                        && before.is_none_or(|b| self.date < b)
                }
                Clause::SizeRange { larger, smaller } => {
                    (larger.is_some() || smaller.is_some())
                        && larger.is_none_or(|l| self.size > l)
                        && smaller.is_none_or(|s| self.size < s)
                }
                Clause::IdSet(_) | Clause::Account(_) => {
                    panic!("engine-built clauses never come from the parser")
                }
            },
        }
    }
}

fn date_bound(s: &str) -> Option<i64> {
    if let Ok(d) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(d.timestamp());
    }
    if let Ok(secs) = s.parse::<i64>() {
        return Some(secs);
    }
    let d = chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()?;
    Some(d.and_hms_opt(0, 0, 0)?.and_utc().timestamp())
}

/// The oracle's reading of a parsed filter, for one row of the querying account.
fn filter_matches(f: &EmailFilter, r: &Row, owned: &[String]) -> bool {
    if let Some(mb) = &f.in_mailbox
        && &r.mailbox != mb
    {
        return false;
    }
    if let Some(list) = &f.in_mailboxes
        && !list.iter().any(|m| owned.contains(m) && *m == r.mailbox)
    {
        return false;
    }
    if let Some(list) = &f.in_mailbox_other_than
        && list.contains(&r.mailbox)
    {
        return false;
    }
    if let Some(t) = &f.text {
        let ok = match mw_search::parse_query(t) {
            Ok(q) => r.eval(&q.expr),
            Err(_) => r.text(TextField::All, t, false),
        };
        if !ok {
            return false;
        }
    }
    for (value, field) in [
        (&f.from, TextField::From),
        (&f.to, TextField::To),
        (&f.cc, TextField::Cc),
        (&f.subject, TextField::Subject),
        (&f.body, TextField::Body),
        (&f.filename, TextField::Filename),
    ] {
        if let Some(v) = value
            && !r.text(field, v, false)
        {
            return false;
        }
    }
    if f.has_attachment.is_some_and(|b| r.c.has_attachment != b)
        || f.has_calendar
            .is_some_and(|b| r.c.calendar_end.is_some() != b)
    {
        return false;
    }
    if let Some(at) = f.calendar_end_after.as_deref().and_then(date_bound)
        && !r.c.calendar_end.is_some_and(|end| end >= at)
    {
        return false;
    }
    if f.has_keyword
        .as_ref()
        .is_some_and(|k| !r.keywords.contains(k))
        || f.not_keyword
            .as_ref()
            .is_some_and(|k| r.keywords.contains(k))
    {
        return false;
    }
    if f.after.is_some() || f.before.is_some() {
        let after = f.after.as_deref().and_then(date_bound);
        let before = f.before.as_deref().and_then(date_bound);
        if !r.eval(&Expr::Clause(Clause::DateRange { after, before })) {
            return false;
        }
    }
    if (f.min_size.is_some() || f.max_size.is_some())
        && !r.eval(&Expr::Clause(Clause::SizeRange {
            larger: f.min_size,
            smaller: f.max_size,
        }))
    {
        return false;
    }
    true
}

/// A sort key value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    N(i64),
    T(String),
}

fn sort_key(r: &Row, s: &mw_search::Sort) -> Key {
    use mw_search::SortField as F;
    match s.field {
        F::ReceivedAt => Key::N(r.date),
        F::SentAt => Key::N(r.c.sent_at),
        F::Size => Key::N(r.size as i64),
        F::From => Key::T(r.c.from.to_lowercase()),
        F::Subject => Key::T(r.c.subject.to_lowercase()),
        F::GroupReceived => {
            use chrono::Datelike;
            Key::N(i64::from(
                chrono::DateTime::from_timestamp(r.date, 0)
                    .expect("date")
                    .date_naive()
                    .num_days_from_ce(),
            ))
        }
        other => panic!("the oracle does not model sort {other:?}"),
    }
}

/// The JMAP sort of a matrix entry.
#[derive(Clone, Debug)]
pub(crate) struct SortSpec {
    pub(crate) json: Value,
}

impl SortSpec {
    fn sorts(&self) -> (mw_search::Sort, Vec<mw_search::Sort>) {
        let list: Vec<Comparator> = serde_json::from_value(self.json.clone()).unwrap_or_default();
        let primary = search_index::sort_from_comparator(list.first());
        let then = list
            .iter()
            .skip(1)
            .map(|c| search_index::sort_from_comparator(Some(c)))
            .collect();
        (primary, then)
    }
}

pub(crate) fn sorts() -> Vec<SortSpec> {
    let one = |p: &str, asc: bool| SortSpec {
        json: json!([{"property": p, "isAscending": asc}]),
    };
    vec![
        SortSpec { json: Value::Null },
        one("receivedAt", true),
        one("receivedAt", false),
        one("size", true),
        one("size", false),
        one("sentAt", false),
        one("sentAt", true),
        one("from", true),
        one("from", false),
        one("subject", true),
        one("subject", false),
        SortSpec {
            json: json!([{"property": "receivedAt", "isAscending": false}, {"property": "subject", "isAscending": true}]),
        },
        SortSpec {
            json: json!([{"property": "mailwomanGroupReceived", "isAscending": false}, {"property": "size", "isAscending": true}]),
        },
    ]
}

/// The window of a matrix entry.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Window {
    pub(crate) position: u64,
    pub(crate) limit: Option<u64>,
    pub(crate) total: bool,
    /// Anchor at this index of the expected result, with offset -1.
    pub(crate) anchor: Option<usize>,
}

pub(crate) const WINDOWS: [Window; 6] = [
    Window {
        position: 0,
        limit: Some(50),
        total: true,
        anchor: None,
    },
    Window {
        position: 7,
        limit: Some(1),
        total: false,
        anchor: None,
    },
    Window {
        position: 0,
        limit: Some(0),
        total: true,
        anchor: None,
    },
    Window {
        position: 0,
        limit: Some(1),
        total: true,
        anchor: None,
    },
    Window {
        position: 0,
        limit: None,
        total: true,
        anchor: None,
    },
    Window {
        position: 0,
        limit: Some(3),
        total: true,
        anchor: Some(3),
    },
];

/// Filter templates. `{INBOX}`, `{Archive}`, `{Work}` name the querying
/// account's folders, `{SAVED}` its saved-search folder, `{FOREIGN}` the
/// other account's INBOX, `{EVENTS}` the Events view.
pub(crate) fn singles() -> Vec<&'static str> {
    vec![
        r#"{}"#,
        r#"{"inMailbox":"{INBOX}"}"#,
        r#"{"inMailbox":"{Work}"}"#,
        r#"{"inMailboxes":["{INBOX}","{Work}"]}"#,
        r#"{"inMailboxes":["{FOREIGN}"]}"#,
        r#"{"inMailboxOtherThan":["{Archive}"]}"#,
        r#"{"inMailbox":"{SAVED}"}"#,
        r#"{"inMailbox":"{FOREIGN}"}"#,
        r#"{"inMailbox":"{EVENTS}"}"#,
        r#"{"text":"alpha"}"#,
        r#"{"from":"s1"}"#,
        r#"{"to":"o1"}"#,
        r#"{"cc":"audit1"}"#,
        r#"{"subject":"bravo"}"#,
        r#"{"body":"charlie"}"#,
        r#"{"filename":"report1"}"#,
        r#"{"hasKeyword":"$seen"}"#,
        r#"{"hasKeyword":"$flagged"}"#,
        r#"{"hasKeyword":"$answered"}"#,
        r#"{"hasKeyword":"work"}"#,
        r#"{"hasKeyword":"$list:x"}"#,
        r#"{"hasKeyword":"mw-followup-completed"}"#,
        r#"{"hasKeyword":"$mw-followup-active"}"#,
        r#"{"notKeyword":"$seen"}"#,
        r#"{"notKeyword":"$flagged"}"#,
        r#"{"notKeyword":"work"}"#,
        r#"{"notKeyword":"$list:x"}"#,
        r#"{"hasAttachment":true}"#,
        r#"{"hasAttachment":false}"#,
        r#"{"hasCalendar":true}"#,
        r#"{"hasCalendar":false}"#,
        r#"{"calendarEndAfter":"2026-07-20T00:00:00Z"}"#,
        r#"{"after":"2026-07-01T10:00:00Z"}"#,
        r#"{"before":"2026-07-01T14:00:00Z"}"#,
        r#"{"minSize":900}"#,
        r#"{"maxSize":700}"#,
        r#"{"semantic":true}"#,
    ]
}

/// The base set whose every pair is run.
fn pair_base() -> Vec<&'static str> {
    vec![
        r#"{"inMailbox":"{INBOX}"}"#,
        r#"{"inMailboxes":["{INBOX}","{Work}"]}"#,
        r#"{"inMailboxOtherThan":["{Archive}"]}"#,
        r#"{"text":"alpha"}"#,
        r#"{"from":"s1"}"#,
        r#"{"subject":"bravo"}"#,
        r#"{"hasKeyword":"$flagged"}"#,
        r#"{"hasKeyword":"work"}"#,
        r#"{"notKeyword":"$seen"}"#,
        r#"{"hasAttachment":true}"#,
        r#"{"hasCalendar":true}"#,
        r#"{"after":"2026-07-01T10:00:00Z"}"#,
        r#"{"minSize":900}"#,
        r#"{"maxSize":700}"#,
        r#"{"inMailbox":"{SAVED}"}"#,
    ]
}

pub(crate) fn pairs() -> Vec<String> {
    let base = pair_base();
    let mut out = Vec::new();
    for (i, x) in base.iter().enumerate() {
        for y in &base[i + 1..] {
            let xv: serde_json::Map<String, Value> = serde_json::from_str(x).unwrap();
            let yv: serde_json::Map<String, Value> = serde_json::from_str(y).unwrap();
            if xv.keys().any(|k| yv.contains_key(k)) {
                continue;
            }
            let mut merged = xv;
            merged.extend(yv);
            let merged = Value::Object(merged).to_string();
            // minSize above maxSize is refused as invalidArguments, not a query.
            if EmailFilter::parse_saved_json(&merged).is_ok() {
                out.push(merged);
            }
        }
    }
    out
}

/// Text-operator forms, including mutable conditions under OR and NOT.
pub(crate) fn text_forms() -> Vec<&'static str> {
    vec![
        "is:unread",
        "is:read",
        "is:flagged",
        "is:unflagged",
        "is:answered",
        "is:unanswered",
        "is:pinned",
        "pinned:false",
        "is:followup",
        "is:finished",
        "tag:work",
        "-tag:work",
        "tag:work -tag:home",
        "tag:$list:x",
        "list:x",
        "is:unread OR from:s3",
        "is:unread OR is:flagged",
        "(is:unread is:flagged) OR tag:home",
        "NOT in:{Archive}",
        "in:{INBOX} OR tag:work",
        "-in:{INBOX} is:unread",
        "body:alpha OR is:flagged",
        "NOT (is:unread OR tag:work)",
        "is:unread OR NOT tag:work",
        "after:1782907200 is:unread",
        "larger:900 tag:work",
        "has:attachment is:unread",
        "-is:unread -tag:work",
        "is:pinned OR is:followup",
        "alph* is:read",
        "from-domain:other.invalid -is:read",
        "has:calendar OR pinned:true",
    ]
}

pub(crate) fn text_filter(form: &str) -> String {
    json!({ "text": form }).to_string()
}

/// Fill a template for `acct`.
pub(crate) fn fill(template: &str, w: &World, acct: &Account) -> String {
    let other = if acct.id == w.a.id { &w.b } else { &w.a };
    template
        .replace("{INBOX}", acct.folder("INBOX"))
        .replace("{Archive}", acct.folder("Archive"))
        .replace("{Work}", acct.folder("Work"))
        .replace("{SAVED}", &acct.saved)
        .replace("{FOREIGN}", other.folder("INBOX"))
        .replace("{EVENTS}", crate::query::EVENTS_VIEW_ID)
}

/// The oracle: the expected ordered result of `filter` with `sort` for `acct`,
/// as (id, primary key) pairs; for a compound sort the key is the id itself.
fn expected(
    w: &World,
    acct: &Account,
    rows: &[Row],
    filter: &str,
    sort: &SortSpec,
) -> (Vec<(String, Key)>, bool) {
    let owned: Vec<String> = acct.folders.iter().map(|(_, id)| id.clone()).collect();
    let mut f = EmailFilter::parse_saved_json(filter).expect("matrix filter parses");
    let mut extra: Option<EmailFilter> = None;
    if f.in_mailbox.as_deref() == Some(crate::query::EVENTS_VIEW_ID) {
        f.in_mailbox = None;
        f.has_calendar = Some(true);
    }
    let mut nothing = false;
    if let Some(mb) = f.in_mailbox.clone()
        && !owned.contains(&mb)
    {
        if mb == acct.saved {
            let stored = EmailFilter::parse_saved_json(r#"{"hasKeyword":"work"}"#).unwrap();
            f.in_mailbox = None;
            extra = Some(f);
            f = stored;
        } else {
            nothing = true;
        }
    }
    let _ = w;
    let (primary, then) = sort.sorts();
    let compound = !then.is_empty() || matches!(primary.field, mw_search::SortField::GroupReceived);
    let mut hits: Vec<&Row> = if nothing {
        Vec::new()
    } else {
        rows.iter()
            .filter(|r| {
                filter_matches(&f, r, &owned)
                    && extra.as_ref().is_none_or(|x| filter_matches(x, r, &owned))
            })
            .collect()
    };
    let cmp_one = |a: &Row, b: &Row, s: &mw_search::Sort| {
        let o = sort_key(a, s).cmp(&sort_key(b, s));
        if s.ascending { o } else { o.reverse() }
    };
    hits.sort_by(|a, b| {
        let mut o = cmp_one(a, b, &primary);
        for s in &then {
            o = o.then_with(|| cmp_one(a, b, s));
        }
        // Ties: compound sorts break by stable id; otherwise any order is
        // accepted within a key, so this only makes the list deterministic.
        o.then_with(|| a.id.cmp(&b.id))
    });
    let out = hits
        .into_iter()
        .map(|r| {
            (
                r.id.clone(),
                if compound {
                    Key::T(r.id.clone())
                } else {
                    sort_key(r, &primary)
                },
            )
        })
        .collect();
    (out, compound)
}

/// Run one matrix entry as `acct` and compare it with the oracle. Returns the
/// method response for callers that compare paths.
pub(crate) async fn check_one(
    w: &World,
    acct: &Account,
    rows: &[Row],
    template: &str,
    sort: &SortSpec,
    win: Window,
    context: &str,
) -> Value {
    let filter = fill(template, w, acct);
    let (want, _compound) = expected(w, acct, rows, &filter, sort);
    let mut args = json!({"accountId": acct.id, "filter": serde_json::from_str::<Value>(&filter).unwrap(), "position": win.position, "calculateTotal": win.total});
    if !sort.json.is_null() {
        args["sort"] = sort.json.clone();
    }
    if let Some(l) = win.limit {
        args["limit"] = json!(l);
    }
    // The anchor: an expected member, or (empty result) an id of the account
    // that is not in it, which must be anchorNotFound.
    let anchor = win
        .anchor
        .map(|i| match want.get(i.min(want.len().saturating_sub(1))) {
            Some((id, _)) if !want.is_empty() => (id.clone(), true),
            _ => (
                rows.first()
                    .map(|r| r.id.clone())
                    .unwrap_or_else(|| "no-such-id".into()),
                false,
            ),
        });
    if let Some((id, _)) = &anchor {
        args["anchor"] = json!(id);
        args["anchorOffset"] = json!(-1);
        args.as_object_mut().unwrap().remove("position");
    }
    let what = format!(
        "{context}: as {} filter {filter} sort {} window {win:?}",
        acct.tag, sort.json
    );
    let r = w.send(&acct.id, json!([["Email/query", args, "q"]])).await;
    let resp = r["methodResponses"][0].clone();
    // A real folder of another account answers an empty page at position 0,
    // whatever the window (query_page returns before reading it).
    let foreign_folder = template.contains(r#""inMailbox":"{FOREIGN}""#);
    if let Some((_, found)) = &anchor
        && !found
        && !foreign_folder
    {
        assert_eq!(
            resp[0],
            json!("error"),
            "{what}: expected anchorNotFound, got {resp}"
        );
        assert_eq!(resp[1]["type"], json!("anchorNotFound"), "{what}: {resp}");
        return resp;
    }
    assert_eq!(resp[0], json!("Email/query"), "{what}: {resp}");
    let body = &resp[1];
    let ids: Vec<String> = body["ids"]
        .as_array()
        .unwrap_or_else(|| panic!("{what}: no ids: {resp}"))
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();
    let position = body["position"]
        .as_u64()
        .unwrap_or_else(|| panic!("{what}: no position: {resp}")) as usize;
    // Total: exactly when asked.
    if win.total {
        assert_eq!(body["total"], json!(want.len()), "{what}: total; {resp}");
    } else {
        assert!(
            body.get("total").is_none(),
            "{what}: total not asked for; {resp}"
        );
    }
    // Position.
    let key_of = |id: &str| want.iter().find(|(i, _)| i == id).map(|(_, k)| k.clone());
    if foreign_folder {
        assert_eq!((ids_len_of(body), position), (0, 0), "{what}: {resp}");
        return resp;
    }
    if let Some((anchor_id, _)) = &anchor {
        let k = key_of(anchor_id).expect("anchor expected");
        let first = want.iter().position(|(_, kk)| *kk == k).unwrap();
        let last = want.iter().rposition(|(_, kk)| *kk == k).unwrap();
        let allowed: BTreeSet<usize> = (first..=last).map(|i| i.saturating_sub(1)).collect();
        assert!(
            allowed.contains(&position),
            "{what}: anchor position {position} not in {allowed:?}; {resp}"
        );
    } else {
        assert_eq!(position, win.position as usize, "{what}: position; {resp}");
    }
    // Ids: every id expected (so of this account and matching), distinct,
    // with the expected key sequence from `position`.
    let distinct: BTreeSet<&String> = ids.iter().collect();
    assert_eq!(distinct.len(), ids.len(), "{what}: duplicate ids; {resp}");
    let other = if acct.id == w.a.id { &w.b } else { &w.a };
    let len = win
        .limit
        .map_or(usize::MAX, |l| l as usize)
        .min(want.len().saturating_sub(position));
    assert_eq!(
        ids.len(),
        len,
        "{what}: page length; want {:?}; {resp}",
        want.iter().skip(position).take(len).collect::<Vec<_>>()
    );
    for (i, id) in ids.iter().enumerate() {
        let Some(k) = key_of(id) else {
            let own = rows.iter().any(|r| &r.id == id);
            panic!(
                "{what}: id {id} not expected (of this account: {own}, other account {}); {resp}",
                other.id
            );
        };
        assert_eq!(
            k,
            want[position + i].1,
            "{what}: key at {}; want {:?}; {resp}",
            position + i,
            want.iter().skip(position).take(len).collect::<Vec<_>>()
        );
    }
    resp
}

fn ids_len_of(body: &Value) -> usize {
    body["ids"].as_array().map_or(usize::MAX, Vec::len)
}

/// Run the matrix as `acct`. `full` = every filter x window and the sorts;
/// otherwise the singles and text forms with the first window.
pub(crate) async fn run_matrix(w: &World, which: char, full: bool, context: &str) -> usize {
    let acct = w.account(which);
    let rows = w.rows(acct).await;
    let mut n = 0;
    let mut filters: Vec<String> = singles().into_iter().map(str::to_owned).collect();
    filters.extend(text_forms().into_iter().map(text_filter));
    let sort_list = sorts();
    let default_sort = &sort_list[0];
    if full {
        let mut all = filters.clone();
        all.extend(pairs());
        for f in &all {
            for win in WINDOWS {
                check_one(w, acct, &rows, f, default_sort, win, context).await;
                n += 1;
            }
        }
        for f in &filters {
            for s in &sort_list[1..] {
                for win in [WINDOWS[0], WINDOWS[1]] {
                    check_one(w, acct, &rows, f, s, win, context).await;
                    n += 1;
                }
            }
        }
    } else {
        for f in &filters {
            check_one(w, acct, &rows, f, default_sort, WINDOWS[0], context).await;
            n += 1;
        }
    }
    n
}

/// Rounds of random operations, each followed by a comparison.
pub(crate) async fn differential(store: Store, messages: u32, rounds: u32, ops: u32) -> usize {
    let w = World::new(store, messages).await;
    // The corpus must exercise every condition: each single removes at least
    // one message and keeps at least one (design part 7 b).
    let rows = w.rows(&w.a).await;
    for template in singles()
        .into_iter()
        .map(str::to_owned)
        .chain(text_forms().into_iter().map(text_filter))
    {
        let filter = fill(&template, &w, &w.a);
        let (hits, _) = expected(&w, &w.a, &rows, &filter, &sorts()[0]);
        let unconstrained = ["{}", r#"{"semantic":true}"#].contains(&template.as_str());
        let empty_by_design = template.contains("{FOREIGN}");
        if !unconstrained && !empty_by_design {
            assert!(
                !hits.is_empty() && hits.len() < rows.len(),
                "corpus does not exercise {filter}: {} of {}",
                hits.len(),
                rows.len()
            );
        }
    }
    let mut compares = run_matrix(&w, 'a', true, "initial").await;
    compares += run_matrix(&w, 'b', false, "initial").await;
    let mut rng = Rng::new(0xE31);
    for round in 0..rounds {
        let mut log = Vec::new();
        for _ in 0..ops {
            log.push(w.mutate(&w.a, &mut rng).await);
        }
        for _ in 0..ops / 3 + 1 {
            log.push(w.mutate(&w.b, &mut rng).await);
        }
        let context = format!("round {round} after [{}]", log.join("; "));
        let last = round + 1 == rounds;
        compares += run_matrix(&w, 'a', last, &context).await;
        compares += run_matrix(&w, 'b', false, &context).await;
    }
    compares
}

#[tokio::test]
async fn email_query_equals_brute_force_oracle_sqlite() {
    let compares = differential(
        Store::open_in_memory(ServerKey::generate()).await.unwrap(),
        48,
        4,
        8,
    )
    .await;
    eprintln!(
        "t50hp-e31 differential sqlite: {compares} compares; accepted differences: {ACCEPTED_DIFFERENCES:?}"
    );
}

#[tokio::test]
async fn email_query_equals_brute_force_oracle_postgres() {
    let Some(dsn) = mw_test_gate::pg_dsn() else {
        mw_test_gate::skip(
            "DATABASE_URL_PG and MW_E14_PG_DSN unset: Email/query differential on PostgreSQL",
        );
        return;
    };
    let compares = differential(
        Store::open(&dsn, ServerKey::generate()).await.unwrap(),
        36,
        3,
        6,
    )
    .await;
    eprintln!("t50hp-e31 differential postgres: {compares} compares");
}
