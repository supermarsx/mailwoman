//! The incremental-sync fallback ladder (plan §1.8): QRESYNC → CONDSTORE →
//! UID-window, plus UIDVALIDITY-reset full resync.
//!
//! Division of labour with `mw-engine` (plan §1.6): the backend reports exact
//! removals (VANISHED) and the server's modified set; it never sees stable ids.
//! Under QRESYNC/CONDSTORE the modified set is surfaced as `flag_changes` — the
//! engine maps any UID it has no stable id for onto a new `added` message. Under
//! the UID-window fallback the backend computes `added` itself from the
//! advertised UIDNEXT boundary.

use mw_engine::backend::{MailboxDelta, MessageRef, RawMailboxRef, SyncCursor};

use crate::error::ImapResult;
use crate::session::{FetchItem, SelectMode, SelectResult, Session};

impl Session {
    /// Incrementally sync one mailbox from a persisted cursor.
    pub async fn sync_mailbox(
        &mut self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> ImapResult<MailboxDelta> {
        match cursor {
            SyncCursor::Qresync {
                uidvalidity,
                highestmodseq,
            } if self.caps().has("QRESYNC") => {
                self.sync_qresync(&mbox.name, *uidvalidity, *highestmodseq)
                    .await
            }
            SyncCursor::Condstore {
                uidvalidity,
                modseq,
            } if self.caps().has("CONDSTORE") => {
                self.sync_condstore(&mbox.name, *uidvalidity, *modseq).await
            }
            SyncCursor::UidWindow {
                uidvalidity,
                uidnext,
            } => {
                self.sync_uid_window(&mbox.name, *uidvalidity, *uidnext)
                    .await
            }
            // Cursor asks for an extension the server no longer advertises:
            // degrade to a UID-window sync keyed off this cursor's UIDVALIDITY.
            SyncCursor::Qresync { uidvalidity, .. } | SyncCursor::Condstore { uidvalidity, .. } => {
                self.sync_uid_window(&mbox.name, *uidvalidity, 1).await
            }
            SyncCursor::Pop3Uidl { .. } => Err(crate::error::ImapError::Unsupported(
                "POP3 UIDL cursor passed to the IMAP backend".into(),
            )),
        }
    }

    async fn sync_qresync(
        &mut self,
        name: &str,
        uidvalidity: u32,
        highestmodseq: u64,
    ) -> ImapResult<MailboxDelta> {
        self.ensure_qresync().await?;
        let sel = self
            .select(
                name,
                SelectMode::Qresync {
                    uidvalidity,
                    highestmodseq,
                },
            )
            .await?;
        if sel.uidvalidity != uidvalidity {
            return self.full_resync(name, &sel).await;
        }
        let removed = refs(name, sel.uidvalidity, &sel.vanished);
        let flag_changes = fetch_flag_changes(name, sel.uidvalidity, &sel.fetched);
        Ok(MailboxDelta {
            added: Vec::new(),
            flag_changes,
            removed,
            next_cursor: SyncCursor::Qresync {
                uidvalidity: sel.uidvalidity,
                highestmodseq: sel.highestmodseq,
            },
        })
    }

    async fn sync_condstore(
        &mut self,
        name: &str,
        uidvalidity: u32,
        modseq: u64,
    ) -> ImapResult<MailboxDelta> {
        let sel = self.select(name, SelectMode::Condstore).await?;
        if sel.uidvalidity != uidvalidity {
            return self.full_resync(name, &sel).await;
        }
        let changed = self.uid_fetch_changed(modseq).await?;
        let flag_changes = fetch_flag_changes(name, sel.uidvalidity, &changed);
        let next_modseq = sel.highestmodseq.max(modseq);
        Ok(MailboxDelta {
            added: Vec::new(),
            flag_changes,
            removed: Vec::new(),
            next_cursor: SyncCursor::Condstore {
                uidvalidity: sel.uidvalidity,
                modseq: next_modseq,
            },
        })
    }

    async fn sync_uid_window(
        &mut self,
        name: &str,
        uidvalidity: u32,
        uidnext: u32,
    ) -> ImapResult<MailboxDelta> {
        let sel = self.select(name, SelectMode::Plain).await?;
        if uidvalidity != 0 && sel.uidvalidity != uidvalidity {
            return self.full_resync(name, &sel).await;
        }
        // New messages occupy [old_uidnext, new_uidnext). Bounded search avoids
        // the `n:*` wrap-around that would re-surface the last existing UID.
        let added_uids = if sel.uidnext > uidnext {
            self.uid_search_range(uidnext.max(1), sel.uidnext - 1)
                .await?
        } else {
            Vec::new()
        };
        Ok(MailboxDelta {
            added: refs(name, sel.uidvalidity, &added_uids),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: sel.uidvalidity,
                uidnext: sel.uidnext,
            },
        })
    }

    /// UIDVALIDITY changed (or first-ever sync of a reset mailbox): every
    /// current UID is treated as newly added; the engine re-derives stable ids.
    async fn full_resync(&mut self, name: &str, sel: &SelectResult) -> ImapResult<MailboxDelta> {
        let all = self.uid_search_all().await?;
        Ok(MailboxDelta {
            added: refs(name, sel.uidvalidity, &all),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: self.strongest_cursor(sel),
        })
    }

    /// Pick the strongest cursor this server supports for a freshly-synced state.
    fn strongest_cursor(&self, sel: &SelectResult) -> SyncCursor {
        if self.caps().has("QRESYNC") || self.caps().has("CONDSTORE") {
            if self.caps().has("QRESYNC") {
                SyncCursor::Qresync {
                    uidvalidity: sel.uidvalidity,
                    highestmodseq: sel.highestmodseq,
                }
            } else {
                SyncCursor::Condstore {
                    uidvalidity: sel.uidvalidity,
                    modseq: sel.highestmodseq,
                }
            }
        } else {
            SyncCursor::UidWindow {
                uidvalidity: sel.uidvalidity,
                uidnext: sel.uidnext,
            }
        }
    }
}

fn refs(name: &str, uidvalidity: u32, uids: &[u32]) -> Vec<MessageRef> {
    uids.iter()
        .map(|&uid| MessageRef::Imap {
            mailbox: RawMailboxRef {
                name: name.to_string(),
                uidvalidity,
            },
            uidvalidity,
            uid,
        })
        .collect()
}

fn fetch_flag_changes(
    name: &str,
    uidvalidity: u32,
    items: &[FetchItem],
) -> Vec<(MessageRef, Vec<mw_engine::backend::Flag>)> {
    items
        .iter()
        .filter_map(|item| {
            let uid = item.uid?;
            Some((
                MessageRef::Imap {
                    mailbox: RawMailboxRef {
                        name: name.to_string(),
                        uidvalidity,
                    },
                    uidvalidity,
                    uid,
                },
                item.flags.clone(),
            ))
        })
        .collect()
}
