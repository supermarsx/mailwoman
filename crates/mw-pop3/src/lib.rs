#![forbid(unsafe_code)]
//! `mw-pop3` — RFC 1939 client with CAPA (2449), STLS (2595) / implicit-TLS on
//! :995, UIDL pull, SASL, and leave-on-server policies (keep /
//! delete-after-N-days / delete-on-retrieval) (plan §0, SPEC §6.3).
//!
//! POP3 exposes a single INBOX and no server-side flags, so its
//! [`AccountBackend`] sync is a UIDL-set diff and flags are engine-local.
//!
//! Scaffolder note (e0): [`Pop3Backend`] is a compiling stub with `todo!()`
//! bodies; the protocol, policies and `cargo-fuzz` target are filled in by e3.

use async_trait::async_trait;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, Flag, MailboxDelta, MessageRef, MoveOutcome,
    RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};

/// Leave-on-server retention policy for retrieved messages (plan §0).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum LeavePolicy {
    /// Never delete from the server.
    #[default]
    Keep,
    /// Delete messages older than N days after retrieval.
    DeleteAfterDays(u32),
    /// Delete each message as soon as it is retrieved.
    DeleteOnRetrieval,
}

/// POP3 implementation of the account-backend seam.
#[derive(Debug, Default)]
pub struct Pop3Backend {
    // e3: tokio+rustls connection, CAPA set, retention policy, UIDL cache.
}

#[async_trait]
impl AccountBackend for Pop3Backend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        todo!("e3: CAPA probe -> BackendCaps")
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        todo!("e3: synthesize the single INBOX")
    }

    async fn sync_mailbox(
        &self,
        _mbox: &RawMailboxRef,
        _cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        todo!("e3: UIDL diff vs SyncCursor::Pop3Uidl")
    }

    async fn fetch_raw(&self, _refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        todo!("e3: RETR by message number resolved from UIDL")
    }

    async fn store_flags(
        &self,
        _refs: &[MessageRef],
        _add: &[Flag],
        _remove: &[Flag],
    ) -> Result<()> {
        todo!("e3: POP3 has no server flags — no-op / engine-local")
    }

    async fn move_messages(
        &self,
        _refs: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> Result<MoveOutcome> {
        todo!("e3: POP3 has no folders — engine-local move only")
    }

    async fn append(
        &self,
        _mbox: &RawMailboxRef,
        _raw: &[u8],
        _flags: &[Flag],
    ) -> Result<MessageRef> {
        todo!("e3: POP3 cannot append — engine handles Sent locally")
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        todo!("e3: periodic poll loop feeding ChangeSink")
    }
}
