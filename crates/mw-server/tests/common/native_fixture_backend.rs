//! Deterministic per-account transport for the separate representative profile.
//! Production engine, policy, state and mutation paths remain unchanged.
use async_trait::async_trait;
use mw_engine::backend::*;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

// t60: process-wide upstream command counters, read by the fixture diagnostics.
static COMMANDS: [AtomicU64; 10] = [const { AtomicU64::new(0) }; 10];
const FETCH: usize = 0;
const FETCHED: usize = 1;
const STORE: usize = 2;
const STORED: usize = 3;
const MOVE: usize = 4;
const MOVED: usize = 5;
const EXPUNGE: usize = 6;
const EXPUNGED: usize = 7;
const APPEND: usize = 8;
const LIST: usize = 9;
fn count(command: usize, messages: Option<(usize, usize)>) {
    COMMANDS[command].fetch_add(1, Ordering::Relaxed);
    if let Some((slot, n)) = messages {
        COMMANDS[slot].fetch_add(n as u64, Ordering::Relaxed);
    }
}
/// Cumulative fake-upstream commands of every representative transport.
pub fn upstream_commands() -> serde_json::Value {
    let get = |i: usize| COMMANDS[i].load(Ordering::Relaxed);
    serde_json::json!({"fetch":get(FETCH),"fetchedMessages":get(FETCHED),"store":get(STORE),"storedMessages":get(STORED),
        "move":get(MOVE),"movedMessages":get(MOVED),"expunge":get(EXPUNGE),"expungedMessages":get(EXPUNGED),"append":get(APPEND),"list":get(LIST),
        "commands":get(FETCH)+get(STORE)+get(MOVE)+get(EXPUNGE)+get(APPEND)+get(LIST),"scope":"process-wide, every fixture account"})
}

#[derive(Clone)]
struct Message {
    raw: Arc<Vec<u8>>,
    flags: Vec<Flag>,
}
struct Folder {
    next: u32,
    messages: BTreeMap<u32, Message>,
}
pub struct RepresentativeBackend {
    folders: Mutex<BTreeMap<String, Folder>>,
}
impl RepresentativeBackend {
    pub fn new(raws: Vec<Vec<u8>>) -> Self {
        let next = raws.len() as u32 + 1;
        let messages = raws
            .into_iter()
            .enumerate()
            .map(|(index, raw)| {
                (
                    index as u32 + 1,
                    Message {
                        raw: Arc::new(raw),
                        flags: vec![],
                    },
                )
            })
            .collect();
        Self {
            folders: Mutex::new(BTreeMap::from([
                ("INBOX".into(), Folder { next, messages }),
                (
                    "Archive".into(),
                    Folder {
                        next: 1,
                        messages: BTreeMap::new(),
                    },
                ),
            ])),
        }
    }
    /// t60: the same transport with each message's seeded flags (`flags[i]`
    /// belongs to UID i + 1), so upstream flag state matches the Store seed.
    pub fn with_flags(raws: Vec<Vec<u8>>, flags: Vec<Vec<Flag>>) -> Self {
        let backend = Self::new(raws);
        {
            let mut state = backend.folders.lock().unwrap();
            let inbox = state.get_mut("INBOX").expect("seeded inbox");
            for ((_, message), seeded) in inbox.messages.iter_mut().zip(flags) {
                message.flags = seeded;
            }
        }
        backend
    }
    /// t60: add empty folders (the goal mix's Trash and Drafts).
    pub fn with_folders(self, names: &[&str]) -> Self {
        {
            let mut state = self.folders.lock().unwrap();
            for name in names {
                state.entry((*name).to_string()).or_insert(Folder { next: 1, messages: BTreeMap::new() });
            }
        }
        self
    }
    fn coordinates(reference: &MessageRef) -> Result<(&str, u32)> {
        match reference {
            MessageRef::Imap {
                mailbox,
                uidvalidity: 1,
                uid,
            } if mailbox.uidvalidity == 1 && *uid > 0 => Ok((&mailbox.name, *uid)),
            _ => Err(EngineError::Protocol("invalid fixture coordinates".into())),
        }
    }
}
#[async_trait]
impl AccountBackend for RepresentativeBackend {
    fn supports_cached_body_refetch(&self) -> bool {
        true
    }
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            r#move: true,
            ..Default::default()
        })
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        count(LIST, None);
        let state = self.folders.lock().unwrap();
        Ok(state
            .iter()
            .map(|(name, folder)| RawMailbox {
                mailbox_ref: RawMailboxRef {
                    name: name.clone(),
                    uidvalidity: 1,
                },
                role: match name.as_str() {
                    "INBOX" => MailboxRole::Inbox,
                    "Trash" => MailboxRole::Trash,
                    "Drafts" => MailboxRole::Drafts,
                    _ => MailboxRole::Archive,
                },
                parent: None,
                uidnext: folder.next,
                highestmodseq: 0,
                total: folder.messages.len() as u32,
                unread: folder
                    .messages
                    .values()
                    .filter(|message| !message.flags.contains(&Flag::Seen))
                    .count() as u32,
            })
            .collect())
    }
    async fn sync_mailbox(
        &self,
        mailbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        if mailbox.uidvalidity != 1 || !self.folders.lock().unwrap().contains_key(&mailbox.name) {
            return Err(EngineError::Protocol("invalid fixture mailbox".into()));
        }
        // All arrivals are preseeded, while flags and moves travel through the
        // engine's own mutation paths. This transport adds no background mail.
        Ok(MailboxDelta {
            added: vec![],
            flag_changes: vec![],
            removed: vec![],
            next_cursor: cursor.clone(),
        })
    }
    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        count(FETCH, Some((FETCHED, refs.len())));
        let state = self.folders.lock().unwrap();
        refs.iter()
            .map(|reference| {
                let (folder, uid) = Self::coordinates(reference)?;
                let message = state
                    .get(folder)
                    .and_then(|folder| folder.messages.get(&uid))
                    .ok_or_else(|| {
                        EngineError::Protocol("fixture message no longer exists".into())
                    })?;
                Ok(RawMessage {
                    message_ref: reference.clone(),
                    raw: message.raw.as_ref().clone(),
                    flags: message.flags.clone(),
                    internaldate: Some("2026-11-01T00:00:00Z".into()),
                })
            })
            .collect()
    }
    async fn store_flags(&self, refs: &[MessageRef], add: &[Flag], remove: &[Flag]) -> Result<()> {
        count(STORE, Some((STORED, refs.len())));
        let coordinates = refs
            .iter()
            .map(Self::coordinates)
            .collect::<Result<Vec<_>>>()?;
        let mut state = self.folders.lock().unwrap();
        if coordinates.iter().any(|(folder, uid)| {
            !state
                .get(*folder)
                .is_some_and(|folder| folder.messages.contains_key(uid))
        }) {
            return Err(EngineError::Protocol(
                "fixture flag target does not exist".into(),
            ));
        }
        for (folder, uid) in coordinates {
            let message = state
                .get_mut(folder)
                .unwrap()
                .messages
                .get_mut(&uid)
                .unwrap();
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
        count(MOVE, Some((MOVED, refs.len())));
        let coordinates = refs
            .iter()
            .map(Self::coordinates)
            .collect::<Result<Vec<_>>>()?;
        let mut state = self.folders.lock().unwrap();
        if to.uidvalidity != 1
            || !state.contains_key(&to.name)
            || coordinates
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != coordinates.len()
            || coordinates.iter().any(|(folder, uid)| {
                *folder == to.name
                    || !state
                        .get(*folder)
                        .is_some_and(|folder| folder.messages.contains_key(uid))
            })
        {
            return Err(EngineError::Protocol("invalid fixture move".into()));
        }
        state[&to.name]
            .next
            .checked_add(refs.len() as u32)
            .ok_or_else(|| EngineError::Protocol("fixture UID exhausted".into()))?;
        let mut uids = Vec::with_capacity(refs.len());
        for (folder, uid) in coordinates {
            let message = state
                .get_mut(folder)
                .unwrap()
                .messages
                .remove(&uid)
                .unwrap();
            let destination = state.get_mut(&to.name).unwrap();
            let next = destination.next;
            destination.next = next
                .checked_add(1)
                .ok_or_else(|| EngineError::Protocol("fixture UID exhausted".into()))?;
            destination.messages.insert(next, message);
            uids.push(next);
        }
        Ok(MoveOutcome::Uidplus {
            uidvalidity: 1,
            uids,
        })
    }
    async fn append(&self, mailbox: &RawMailboxRef, raw: &[u8], flags: &[Flag]) -> Result<MessageRef> {
        // t60: only the goal mix's Drafts folder accepts appends (draft save);
        // everything else keeps the representative profile's refusal.
        let mut state = self.folders.lock().unwrap();
        match state.get_mut(&mailbox.name) {
            Some(folder) if mailbox.name == "Drafts" => {
                count(APPEND, None);
                let uid = folder.next;
                folder.next += 1;
                folder.messages.insert(uid, Message { raw: Arc::new(raw.to_vec()), flags: flags.to_vec() });
                Ok(MessageRef::Imap { mailbox: RawMailboxRef { name: mailbox.name.clone(), uidvalidity: 1 }, uidvalidity: 1, uid })
            }
            _ => Err(EngineError::Unsupported("representative profile does not benchmark append/send".into())),
        }
    }
    async fn expunge_messages(&self, refs: &[MessageRef]) -> Result<()> {
        // t60: permanent removal only from the goal mix's Drafts and Trash.
        let coordinates = refs.iter().map(Self::coordinates).collect::<Result<Vec<_>>>()?;
        if coordinates.iter().any(|(folder, _)| !matches!(*folder, "Drafts" | "Trash")) {
            return Err(EngineError::Unsupported("representative profile expunges only Drafts and Trash".into()));
        }
        count(EXPUNGE, Some((EXPUNGED, refs.len())));
        let mut state = self.folders.lock().unwrap();
        for (folder, uid) in coordinates {
            if let Some(folder) = state.get_mut(folder) {
                folder.messages.remove(&uid);
            }
        }
        Ok(())
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported(
            "representative profile has no upstream arrivals".into(),
        ))
    }
}

#[tokio::test]
async fn representative_transport_mutations_preserve_actual_content_and_reject_foreign_coordinates()
{
    let backend = RepresentativeBackend::new(vec![b"original".to_vec()]);
    let inbox = RawMailboxRef {
        name: "INBOX".into(),
        uidvalidity: 1,
    };
    let archive = RawMailboxRef {
        name: "Archive".into(),
        uidvalidity: 1,
    };
    let source = MessageRef::Imap {
        mailbox: inbox,
        uidvalidity: 1,
        uid: 1,
    };
    backend
        .store_flags(std::slice::from_ref(&source), &[Flag::Seen], &[])
        .await
        .unwrap();
    assert_eq!(
        backend
            .fetch_raw(std::slice::from_ref(&source))
            .await
            .unwrap()[0]
            .flags,
        vec![Flag::Seen]
    );
    let moved = backend
        .move_messages(std::slice::from_ref(&source), &archive)
        .await
        .unwrap();
    let MoveOutcome::Uidplus { uids, .. } = moved else {
        panic!("missing UID proof")
    };
    let destination = MessageRef::Imap {
        mailbox: archive,
        uidvalidity: 1,
        uid: uids[0],
    };
    assert_eq!(
        backend.fetch_raw(&[destination]).await.unwrap()[0].raw,
        b"original"
    );
    assert!(backend.fetch_raw(&[source]).await.is_err());
    assert!(
        backend
            .store_flags(
                &[MessageRef::Pop3 {
                    uidl: "foreign".into()
                }],
                &[],
                &[]
            )
            .await
            .is_err()
    );
}
