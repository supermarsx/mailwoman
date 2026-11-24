//! Fake upstream for the bulk-operation benchmark (t60). It keeps per-folder
//! UID state, answers the engine's real sync/fetch/flag/move/expunge/append
//! calls, and counts every upstream command so a record can say what a bulk
//! operation asked of the mail server. Corpus messages are regenerated from
//! their seed instead of being held in memory.
use super::corpus;
use async_trait::async_trait;
use mw_engine::backend::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
enum Source {
    Corpus(u32),
    Raw(Arc<Vec<u8>>),
}
#[derive(Clone)]
struct Message {
    source: Source,
    flags: Vec<Flag>,
    /// Already known to the engine (seeded, moved or appended by it). A sync
    /// reports only arrivals the engine has not been told about.
    announced: bool,
}
struct Folder {
    role: MailboxRole,
    next: u32,
    messages: BTreeMap<u32, Message>,
}

#[derive(Default)]
pub struct Commands {
    pub list: AtomicU64,
    pub sync: AtomicU64,
    pub fetch: AtomicU64,
    pub fetched_messages: AtomicU64,
    pub store: AtomicU64,
    pub stored_messages: AtomicU64,
    pub r#move: AtomicU64,
    pub moved_messages: AtomicU64,
    pub expunge: AtomicU64,
    pub expunged_messages: AtomicU64,
    pub append: AtomicU64,
    pub create_mailbox: AtomicU64,
    pub delete_mailbox: AtomicU64,
}
impl Commands {
    pub fn snapshot(&self) -> Value {
        let get = |value: &AtomicU64| value.load(Ordering::Relaxed);
        json!({"list":get(&self.list),"sync":get(&self.sync),"fetch":get(&self.fetch),"fetchedMessages":get(&self.fetched_messages),
            "store":get(&self.store),"storedMessages":get(&self.stored_messages),"move":get(&self.r#move),"movedMessages":get(&self.moved_messages),
            "expunge":get(&self.expunge),"expungedMessages":get(&self.expunged_messages),"append":get(&self.append),
            "createMailbox":get(&self.create_mailbox),"deleteMailbox":get(&self.delete_mailbox)})
    }
}
/// `after - before` for two [`Commands::snapshot`] values.
pub fn delta(before: &Value, after: &Value) -> Value {
    let mut out = serde_json::Map::new();
    for (key, value) in after.as_object().into_iter().flatten() {
        let old = before[key].as_u64().unwrap_or(0);
        out.insert(key.clone(), json!(value.as_u64().unwrap_or(0).saturating_sub(old)));
    }
    Value::Object(out)
}

pub struct BulkBackend {
    tag: String,
    to: String,
    folders: Mutex<BTreeMap<String, Folder>>,
    pub commands: Commands,
}

pub const FOLDERS: [(&str, MailboxRole); 6] = [
    ("INBOX", MailboxRole::Inbox),
    ("Archive", MailboxRole::Archive),
    ("Trash", MailboxRole::Trash),
    ("Junk", MailboxRole::Junk),
    ("Bulk", MailboxRole::None),
    ("Import", MailboxRole::None),
];

impl BulkBackend {
    /// `seeded` maps a folder to its corpus seeds (UID = position + 1).
    /// `announced` marks them as already in the engine's cache (store seed).
    pub fn new(tag: &str, to: &str, seeded: &[(&str, std::ops::RangeInclusive<u32>)], announced: bool) -> Self {
        let mut folders = BTreeMap::new();
        for (name, role) in FOLDERS {
            folders.insert(name.to_string(), Folder { role, next: 1, messages: BTreeMap::new() });
        }
        for (name, seeds) in seeded {
            let folder = folders.get_mut(*name).expect("known fixture folder");
            for seed in seeds.clone() {
                let uid = folder.next;
                folder.next += 1;
                folder.messages.insert(uid, Message { source: Source::Corpus(seed), flags: vec![], announced });
            }
        }
        Self { tag: tag.into(), to: to.into(), folders: Mutex::new(folders), commands: Commands::default() }
    }
    fn raw_of(&self, message: &Message) -> Vec<u8> {
        match &message.source {
            Source::Corpus(seed) => corpus::raw(&self.tag, *seed, &self.to),
            Source::Raw(raw) => raw.as_ref().clone(),
        }
    }
    fn coordinates(reference: &MessageRef) -> Result<(String, u32)> {
        match reference {
            MessageRef::Imap { mailbox, uidvalidity: 1, uid } if mailbox.uidvalidity == 1 && *uid > 0 => Ok((mailbox.name.clone(), *uid)),
            _ => Err(EngineError::Protocol("invalid fixture coordinates".into())),
        }
    }
    /// A new message that the engine has not seen (it reports on next sync).
    pub fn deliver(&self, folder: &str, seed: u32) -> u32 {
        let mut state = self.folders.lock().unwrap();
        let folder = state.get_mut(folder).expect("known folder");
        let uid = folder.next;
        folder.next += 1;
        folder.messages.insert(uid, Message { source: Source::Corpus(seed), flags: vec![], announced: false });
        uid
    }
    /// `(count, unread, flagged)` of one folder, or `None` once deleted.
    pub fn folder_counts(&self, folder: &str) -> Option<(usize, usize, usize)> {
        let state = self.folders.lock().unwrap();
        state.get(folder).map(|folder| {
            let unread = folder.messages.values().filter(|m| !m.flags.contains(&Flag::Seen)).count();
            let flagged = folder.messages.values().filter(|m| m.flags.contains(&Flag::Flagged)).count();
            (folder.messages.len(), unread, flagged)
        })
    }
    /// The Message-ID at each UID of a folder (for location checks).
    pub fn folder_message_ids(&self, folder: &str) -> BTreeMap<u32, String> {
        let state = self.folders.lock().unwrap();
        state
            .get(folder)
            .map(|folder| {
                folder
                    .messages
                    .iter()
                    .map(|(uid, message)| {
                        let id = match &message.source {
                            Source::Corpus(seed) => corpus::message_id(&self.tag, *seed),
                            Source::Raw(raw) => message_id_of(raw),
                        };
                        (*uid, id)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn message_id_of(raw: &[u8]) -> String {
    let text = String::from_utf8_lossy(raw);
    text.lines()
        .take_while(|line| !line.is_empty())
        .find_map(|line| line.strip_prefix("Message-ID: ").map(|value| value.trim().to_string()))
        .unwrap_or_default()
}

fn mailbox_ref(name: &str) -> RawMailboxRef {
    RawMailboxRef { name: name.into(), uidvalidity: 1 }
}

#[async_trait]
impl AccountBackend for BulkBackend {
    fn supports_cached_body_refetch(&self) -> bool {
        true
    }
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps { uidplus: true, r#move: true, special_use: true, ..Default::default() })
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        self.commands.list.fetch_add(1, Ordering::Relaxed);
        let state = self.folders.lock().unwrap();
        Ok(state
            .iter()
            .map(|(name, folder)| RawMailbox {
                mailbox_ref: mailbox_ref(name),
                role: folder.role,
                parent: None,
                uidnext: folder.next,
                highestmodseq: 0,
                total: folder.messages.len() as u32,
                unread: folder.messages.values().filter(|m| !m.flags.contains(&Flag::Seen)).count() as u32,
            })
            .collect())
    }
    async fn sync_mailbox(&self, mailbox: &RawMailboxRef, cursor: &SyncCursor) -> Result<MailboxDelta> {
        self.commands.sync.fetch_add(1, Ordering::Relaxed);
        if mailbox.uidvalidity != 1 && mailbox.uidvalidity != 0 {
            return Err(EngineError::Protocol("invalid fixture mailbox".into()));
        }
        let from = match cursor {
            SyncCursor::UidWindow { uidnext, .. } => *uidnext,
            _ => 1,
        };
        let mut state = self.folders.lock().unwrap();
        let folder = state.get_mut(&mailbox.name).ok_or_else(|| EngineError::MailboxNotFound(mailbox.name.clone()))?;
        let mut added = Vec::new();
        for (uid, message) in folder.messages.range_mut(from.max(1)..) {
            if !message.announced {
                message.announced = true;
                added.push(MessageRef::Imap { mailbox: mailbox_ref(&mailbox.name), uidvalidity: 1, uid: *uid });
            }
        }
        Ok(MailboxDelta {
            added,
            flag_changes: vec![],
            removed: vec![],
            next_cursor: SyncCursor::UidWindow { uidvalidity: 1, uidnext: folder.next },
        })
    }
    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        self.commands.fetch.fetch_add(1, Ordering::Relaxed);
        self.commands.fetched_messages.fetch_add(refs.len() as u64, Ordering::Relaxed);
        let found = {
            let state = self.folders.lock().unwrap();
            refs.iter()
                .map(|reference| {
                    let (folder, uid) = Self::coordinates(reference)?;
                    state
                        .get(&folder)
                        .and_then(|folder| folder.messages.get(&uid))
                        .cloned()
                        .ok_or_else(|| EngineError::Protocol("fixture message no longer exists".into()))
                        .map(|message| (reference.clone(), message))
                })
                .collect::<Result<Vec<_>>>()?
        };
        Ok(found
            .into_iter()
            .map(|(reference, message)| {
                let seed = match message.source { Source::Corpus(seed) => Some(seed), Source::Raw(_) => None };
                RawMessage {
                    message_ref: reference,
                    raw: self.raw_of(&message),
                    flags: message.flags.clone(),
                    internaldate: Some(seed.map(corpus::received_at).unwrap_or_else(|| corpus::received_at(0))),
                }
            })
            .collect())
    }
    async fn store_flags(&self, refs: &[MessageRef], add: &[Flag], remove: &[Flag]) -> Result<()> {
        self.commands.store.fetch_add(1, Ordering::Relaxed);
        self.commands.stored_messages.fetch_add(refs.len() as u64, Ordering::Relaxed);
        let coordinates = refs.iter().map(Self::coordinates).collect::<Result<Vec<_>>>()?;
        let mut state = self.folders.lock().unwrap();
        if coordinates.iter().any(|(folder, uid)| !state.get(folder).is_some_and(|f| f.messages.contains_key(uid))) {
            return Err(EngineError::Protocol("fixture flag target does not exist".into()));
        }
        for (folder, uid) in coordinates {
            let message = state.get_mut(&folder).unwrap().messages.get_mut(&uid).unwrap();
            message.flags.retain(|flag| !remove.contains(flag));
            for flag in add {
                if !message.flags.contains(flag) {
                    message.flags.push(flag.clone());
                }
            }
        }
        Ok(())
    }
    async fn move_messages(&self, refs: &[MessageRef], to: &RawMailboxRef) -> Result<MoveOutcome> {
        self.commands.r#move.fetch_add(1, Ordering::Relaxed);
        self.commands.moved_messages.fetch_add(refs.len() as u64, Ordering::Relaxed);
        let coordinates = refs.iter().map(Self::coordinates).collect::<Result<Vec<_>>>()?;
        let mut state = self.folders.lock().unwrap();
        if !state.contains_key(&to.name)
            || coordinates.iter().collect::<std::collections::HashSet<_>>().len() != coordinates.len()
            || coordinates.iter().any(|(folder, uid)| *folder == to.name || !state.get(folder).is_some_and(|f| f.messages.contains_key(uid)))
        {
            return Err(EngineError::Protocol("invalid fixture move".into()));
        }
        let mut uids = Vec::with_capacity(refs.len());
        for (folder, uid) in coordinates {
            let mut message = state.get_mut(&folder).unwrap().messages.remove(&uid).unwrap();
            message.announced = true;
            let destination = state.get_mut(&to.name).unwrap();
            let next = destination.next;
            destination.next += 1;
            destination.messages.insert(next, message);
            uids.push(next);
        }
        Ok(MoveOutcome::Uidplus { uidvalidity: 1, uids })
    }
    async fn append(&self, mailbox: &RawMailboxRef, raw: &[u8], flags: &[Flag]) -> Result<MessageRef> {
        self.commands.append.fetch_add(1, Ordering::Relaxed);
        let mut state = self.folders.lock().unwrap();
        let folder = state.get_mut(&mailbox.name).ok_or_else(|| EngineError::MailboxNotFound(mailbox.name.clone()))?;
        let uid = folder.next;
        folder.next += 1;
        folder.messages.insert(uid, Message { source: Source::Raw(Arc::new(raw.to_vec())), flags: flags.to_vec(), announced: true });
        Ok(MessageRef::Imap { mailbox: mailbox_ref(&mailbox.name), uidvalidity: 1, uid })
    }
    fn supports_dated_import(&self) -> bool {
        true
    }
    async fn append_with_date(&self, mailbox: &RawMailboxRef, raw: &[u8], flags: &[Flag], _received_at: &str) -> Result<MessageRef> {
        self.append(mailbox, raw, flags).await
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("bulk fixture has no IDLE; arrivals are delivered by explicit refresh".into()))
    }
    fn manages_mailboxes(&self) -> bool {
        true
    }
    async fn hierarchy_delimiter(&self) -> Result<Option<char>> {
        Ok(Some('/'))
    }
    async fn create_mailbox(&self, name: &str) -> Result<()> {
        self.commands.create_mailbox.fetch_add(1, Ordering::Relaxed);
        self.folders.lock().unwrap().entry(name.into()).or_insert(Folder { role: MailboxRole::None, next: 1, messages: BTreeMap::new() });
        Ok(())
    }
    async fn delete_mailbox(&self, mailbox: &RawMailboxRef) -> Result<()> {
        self.commands.delete_mailbox.fetch_add(1, Ordering::Relaxed);
        match self.folders.lock().unwrap().remove(&mailbox.name) {
            Some(_) => Ok(()),
            None => Err(EngineError::MailboxNotFound(mailbox.name.clone())),
        }
    }
    async fn subscribed_mailboxes(&self) -> Result<Vec<String>> {
        Ok(self.folders.lock().unwrap().keys().cloned().collect())
    }
    async fn expunge_messages(&self, refs: &[MessageRef]) -> Result<()> {
        self.commands.expunge.fetch_add(1, Ordering::Relaxed);
        self.commands.expunged_messages.fetch_add(refs.len() as u64, Ordering::Relaxed);
        let coordinates = refs.iter().map(Self::coordinates).collect::<Result<Vec<_>>>()?;
        let mut state = self.folders.lock().unwrap();
        if coordinates.iter().any(|(folder, uid)| !state.get(folder).is_some_and(|f| f.messages.contains_key(uid))) {
            return Err(EngineError::Protocol("fixture expunge target does not exist".into()));
        }
        for (folder, uid) in coordinates {
            state.get_mut(&folder).unwrap().messages.remove(&uid);
        }
        Ok(())
    }
}

#[tokio::test]
async fn bulk_backend_reports_only_unannounced_arrivals_and_counts_commands() {
    let backend = BulkBackend::new("b", "u@example.invalid", &[("INBOX", 1..=3)], false);
    let inbox = mailbox_ref("INBOX");
    let first = backend.sync_mailbox(&inbox, &SyncCursor::UidWindow { uidvalidity: 1, uidnext: 1 }).await.unwrap();
    assert_eq!(first.added.len(), 3);
    let again = backend.sync_mailbox(&inbox, &first.next_cursor).await.unwrap();
    assert!(again.added.is_empty());
    let raws = backend.fetch_raw(&first.added).await.unwrap();
    assert_eq!(raws[0].raw, corpus::raw("b", 1, "u@example.invalid"));
    let moved = backend.move_messages(&first.added[..2], &mailbox_ref("Archive")).await.unwrap();
    assert_eq!(moved, MoveOutcome::Uidplus { uidvalidity: 1, uids: vec![1, 2] });
    assert!(backend.sync_mailbox(&mailbox_ref("Archive"), &SyncCursor::UidWindow { uidvalidity: 1, uidnext: 1 }).await.unwrap().added.is_empty());
    backend.store_flags(&first.added[2..], &[Flag::Seen], &[]).await.unwrap();
    assert_eq!(backend.folder_counts("INBOX"), Some((1, 0, 0)));
    backend.expunge_messages(&first.added[2..]).await.unwrap();
    assert_eq!(backend.folder_counts("INBOX"), Some((0, 0, 0)));
    assert!(backend.expunge_messages(&first.added[2..]).await.is_err());
    let uid = backend.deliver("INBOX", 9);
    assert_eq!(uid, 4);
    assert_eq!(backend.sync_mailbox(&inbox, &first.next_cursor).await.unwrap().added.len(), 1);
    let counts = backend.commands.snapshot();
    assert_eq!((counts["move"].as_u64(), counts["movedMessages"].as_u64(), counts["expunge"].as_u64()), (Some(1), Some(2), Some(2)));
    assert_eq!(backend.folder_message_ids("Archive")[&1], corpus::message_id("b", 1));
}
