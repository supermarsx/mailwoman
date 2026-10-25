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
use std::collections::BTreeSet;

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
            SyncCursor::ImapSnapshot {
                uidvalidity,
                uidnext,
                known_uids,
                highestmodseq,
            } => {
                let condstore = self.caps().has("CONDSTORE") || self.caps().has("QRESYNC");
                let sel = self
                    .select(
                        &mbox.name,
                        if condstore {
                            SelectMode::Condstore
                        } else {
                            SelectMode::Plain
                        },
                    )
                    .await?;
                if sel.uidvalidity != *uidvalidity
                    || (self.caps().has("QRESYNC") && sel.highestmodseq > 0)
                {
                    return self.full_resync(&mbox.name, &sel).await;
                }
                self.snapshot_delta(
                    &mbox.name,
                    &sel,
                    Some(known_uids),
                    *uidnext,
                    highestmodseq.filter(|_| condstore && sel.highestmodseq > 0),
                )
                .await
            }
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
            // A bridge/plugin cursor (Graph deltaLink / Gmail historyId / EWS
            // SyncState) is opaque to the standards IMAP backend — it can only
            // reach here by misrouting, so error rather than silently full-resync.
            SyncCursor::Plugin { .. } => Err(crate::error::ImapError::Unsupported(
                "plugin/bridge cursor passed to the IMAP backend".into(),
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
        if sel.uidvalidity != uidvalidity || sel.highestmodseq == 0 {
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
        _modseq: u64,
    ) -> ImapResult<MailboxDelta> {
        let sel = self.select(name, SelectMode::Condstore).await?;
        if sel.uidvalidity != uidvalidity {
            return self.full_resync(name, &sel).await;
        }
        // Legacy CONDSTORE cursors have no UID membership. Refresh flags once
        // and let the engine reconcile old cache members before persisting it.
        self.snapshot_delta(name, &sel, None, u32::MAX, None).await
    }

    async fn sync_uid_window(
        &mut self,
        name: &str,
        uidvalidity: u32,
        uidnext: u32,
    ) -> ImapResult<MailboxDelta> {
        // Request mod-sequences on initial selection so the promoted cursor
        // records the server's actual checkpoint, rather than zero.
        let mode = if self.caps().has("CONDSTORE") || self.caps().has("QRESYNC") {
            SelectMode::Condstore
        } else {
            SelectMode::Plain
        };
        let sel = self.select(name, mode).await?;
        if uidvalidity == 0
            || sel.uidvalidity != uidvalidity
            || (self.caps().has("QRESYNC") && sel.highestmodseq > 0)
        {
            return self.full_resync(name, &sel).await;
        }
        self.snapshot_delta(name, &sel, None, uidnext, None).await
    }

    /// UIDVALIDITY changed (or first-ever sync of a reset mailbox): every
    /// current UID is treated as newly added; the engine re-derives stable ids.
    async fn full_resync(&mut self, name: &str, sel: &SelectResult) -> ImapResult<MailboxDelta> {
        if sel.highestmodseq == 0 || !self.caps().has("QRESYNC") {
            return self.snapshot_delta(name, sel, None, 1, None).await;
        }
        let all = self.uid_search_all().await?;
        Ok(MailboxDelta {
            added: refs(name, sel.uidvalidity, &all),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: self.strongest_cursor(sel),
        })
    }

    async fn snapshot_delta(
        &mut self,
        name: &str,
        sel: &SelectResult,
        previous: Option<&BTreeSet<u32>>,
        legacy_uidnext: u32,
        since: Option<u64>,
    ) -> ImapResult<MailboxDelta> {
        let current: BTreeSet<u32> = self.uid_search_all().await?.into_iter().collect();
        let fetched = if current.is_empty() {
            Vec::new()
        } else if let Some(since) = since {
            self.uid_fetch_changed(since).await?
        } else {
            self.uid_fetch_all_flags().await?
        };
        let added: BTreeSet<u32> = current
            .iter()
            .copied()
            .filter(|uid| previous.map_or(*uid >= legacy_uidnext, |old| !old.contains(uid)))
            .collect();
        let removed: Vec<u32> = previous
            .map(|old| old.difference(&current).copied().collect())
            .unwrap_or_default();
        let changed: Vec<_> = fetched
            .into_iter()
            .filter(|item| {
                item.uid
                    .is_some_and(|uid| current.contains(&uid) && !added.contains(&uid))
            })
            .collect();
        Ok(MailboxDelta {
            added: refs(
                name,
                sel.uidvalidity,
                &added.into_iter().collect::<Vec<_>>(),
            ),
            flag_changes: fetch_flag_changes(name, sel.uidvalidity, &changed),
            removed: refs(name, sel.uidvalidity, &removed),
            next_cursor: SyncCursor::ImapSnapshot {
                uidvalidity: sel.uidvalidity,
                uidnext: sel.uidnext,
                known_uids: current,
                highestmodseq: (self.caps().has("CONDSTORE") && sel.highestmodseq > 0)
                    .then_some(sel.highestmodseq),
            },
        })
    }

    /// Pick the strongest cursor this server supports for a freshly-synced state.
    fn strongest_cursor(&self, sel: &SelectResult) -> SyncCursor {
        // NOMODSEQ (or an absent HIGHESTMODSEQ) means this mailbox cannot
        // support a mod-sequence cursor even when the server advertises it.
        if sel.highestmodseq > 0 && (self.caps().has("QRESYNC") || self.caps().has("CONDSTORE")) {
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
