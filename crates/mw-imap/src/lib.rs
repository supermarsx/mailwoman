#![forbid(unsafe_code)]
//! `mw-imap` — hardened IMAP4rev2 (RFC 9051) client with RFC 3501 fallback and
//! the V1-critical extensions (SPECIAL-USE, LIST-STATUS, IDLE, CONDSTORE,
//! QRESYNC, MOVE, UIDPLUS, ESEARCH, ENABLE, ID, COMPRESS, SASL), built over
//! `tokio` + `rustls` with `imap-proto` as the response parser (plan §1.1).
//!
//! Scaffolder note (e0): [`ImapBackend`] is a compiling stub implementing the
//! frozen [`AccountBackend`] seam with `todo!()` bodies. The transport, command
//! layer, fallback ladder and `cargo-fuzz` target are filled in by e2.

use async_trait::async_trait;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, Flag, MailboxDelta, MessageRef, MoveOutcome,
    RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};

/// IMAP implementation of the account-backend seam.
#[derive(Debug, Default)]
pub struct ImapBackend {
    // e2: tokio+rustls connection, negotiated CAPABILITY set, selected mailbox.
}

#[async_trait]
impl AccountBackend for ImapBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        todo!("e2: CAPABILITY/ENABLE probe -> BackendCaps")
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        todo!("e2: LIST (SPECIAL-USE) + STATUS/LIST-STATUS")
    }

    async fn sync_mailbox(
        &self,
        _mbox: &RawMailboxRef,
        _cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        todo!("e2: SELECT (QRESYNC) / UID FETCH (CONDSTORE) / UID-window ladder")
    }

    async fn fetch_raw(&self, _refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        todo!("e2: UID FETCH BODY.PEEK[]")
    }

    async fn store_flags(
        &self,
        _refs: &[MessageRef],
        _add: &[Flag],
        _remove: &[Flag],
    ) -> Result<()> {
        todo!("e2: UID STORE +/-FLAGS")
    }

    async fn move_messages(
        &self,
        _refs: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> Result<MoveOutcome> {
        todo!("e2: UID MOVE (UIDPLUS) or COPY+STORE\\Deleted+EXPUNGE")
    }

    async fn append(
        &self,
        _mbox: &RawMailboxRef,
        _raw: &[u8],
        _flags: &[Flag],
    ) -> Result<MessageRef> {
        todo!("e2: APPEND (UIDPLUS)")
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        todo!("e2: IDLE loop feeding ChangeSink")
    }
}
