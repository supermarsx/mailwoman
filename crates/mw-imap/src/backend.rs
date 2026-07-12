//! [`ImapBackend`] — the [`AccountBackend`] implementation over a live session.
//!
//! The struct owns one authenticated [`Session`] behind a mutex for the
//! request/response methods, plus the [`ImapConfig`] it dialled with so `watch`
//! can open an independent connection for its IDLE loop (IDLE monopolises a
//! connection, so it must not share the command connection).

use std::time::Duration;

use async_trait::async_trait;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeEvent, ChangeSink, EngineError, Flag, MailboxDelta,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor,
    WatchHandle,
};
use tokio::sync::{Mutex, watch};

use crate::session::{Credentials, SelectMode, Session};
use crate::transport::TlsMode;

/// Connection + auth parameters for one IMAP account.
#[derive(Debug, Clone)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub tls: TlsMode,
    pub credentials: Credentials,
    /// Mailbox the `watch` IDLE loop observes (defaults to `INBOX`).
    pub watch_mailbox: String,
}

impl ImapConfig {
    /// Build a config with sensible defaults (implicit TLS on 993, watch INBOX).
    pub fn new(host: impl Into<String>, credentials: Credentials) -> Self {
        ImapConfig {
            host: host.into(),
            port: 993,
            tls: TlsMode::Implicit,
            credentials,
            watch_mailbox: "INBOX".to_string(),
        }
    }

    /// Override the port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Override the TLS mode.
    pub fn tls(mut self, tls: TlsMode) -> Self {
        self.tls = tls;
        self
    }
}

/// IMAP implementation of the account-backend seam.
pub struct ImapBackend {
    config: ImapConfig,
    session: Mutex<Session>,
    caps: BackendCaps,
}

impl ImapBackend {
    /// Dial, authenticate, send `ID`, and enable the sync extensions.
    pub async fn connect(config: ImapConfig) -> Result<Self> {
        let session = connect_session(&config).await?;
        let caps = session.backend_caps();
        Ok(ImapBackend {
            config,
            session: Mutex::new(session),
            caps,
        })
    }

    /// The capabilities negotiated at connect time.
    pub fn caps(&self) -> &BackendCaps {
        &self.caps
    }
}

async fn connect_session(config: &ImapConfig) -> Result<Session> {
    let mut session = Session::connect(&config.host, config.port, config.tls).await?;
    session.login(&config.credentials).await?;
    // ID + ENABLE are best-effort niceties; never fail the connect over them.
    let _ = session.send_id().await;
    session.enable_sync_extensions().await?;
    Ok(session)
}

#[async_trait]
impl AccountBackend for ImapBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(self.caps.clone())
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        let mut session = self.session.lock().await;
        Ok(session.list_mailboxes().await?)
    }

    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        let mut session = self.session.lock().await;
        Ok(session.sync_mailbox(mbox, cursor).await?)
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        let groups = group_imap_refs(refs)?;
        let mut session = self.session.lock().await;
        let mut out = Vec::new();
        for (mailbox, uids) in groups {
            out.extend(session.fetch_raw(&mailbox, &uids).await?);
        }
        Ok(out)
    }

    async fn store_flags(&self, refs: &[MessageRef], add: &[Flag], remove: &[Flag]) -> Result<()> {
        let groups = group_imap_refs(refs)?;
        let mut session = self.session.lock().await;
        for (mailbox, uids) in groups {
            session.store_flags(&mailbox, &uids, add, remove).await?;
        }
        Ok(())
    }

    async fn move_messages(&self, refs: &[MessageRef], to: &RawMailboxRef) -> Result<MoveOutcome> {
        let groups = group_imap_refs(refs)?;
        let mut session = self.session.lock().await;
        let mut merged: Option<MoveOutcome> = None;
        for (mailbox, uids) in groups {
            let outcome = session.move_messages(&mailbox, &uids, &to.name).await?;
            merged = Some(merge_outcomes(merged, outcome));
        }
        Ok(merged.unwrap_or(MoveOutcome::RederiveByMessageId))
    }

    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], flags: &[Flag]) -> Result<MessageRef> {
        let mut session = self.session.lock().await;
        Ok(session.append(&mbox.name, raw, flags).await?)
    }

    async fn watch(&self, sink: ChangeSink) -> Result<WatchHandle> {
        let (stop_tx, stop_rx) = watch::channel(false);
        let config = self.config.clone();
        tokio::spawn(async move {
            watch_loop(config, sink, stop_rx).await;
        });
        Ok(WatchHandle::new(stop_tx))
    }
}

/// Combine per-mailbox move outcomes: any re-derive taints the whole result;
/// otherwise concatenate the UIDPLUS destination UIDs.
fn merge_outcomes(acc: Option<MoveOutcome>, next: MoveOutcome) -> MoveOutcome {
    match (acc, next) {
        (None, o) => o,
        (Some(MoveOutcome::RederiveByMessageId), _) | (_, MoveOutcome::RederiveByMessageId) => {
            MoveOutcome::RederiveByMessageId
        }
        (
            Some(MoveOutcome::Uidplus {
                uidvalidity,
                mut uids,
            }),
            MoveOutcome::Uidplus { uids: mut more, .. },
        ) => {
            uids.append(&mut more);
            MoveOutcome::Uidplus { uidvalidity, uids }
        }
    }
}

/// Group message refs by their source mailbox, rejecting POP3 refs.
fn group_imap_refs(refs: &[MessageRef]) -> Result<Vec<(RawMailboxRef, Vec<u32>)>> {
    let mut groups: Vec<(RawMailboxRef, Vec<u32>)> = Vec::new();
    for r in refs {
        match r {
            MessageRef::Imap { mailbox, uid, .. } => {
                if let Some(g) = groups.iter_mut().find(|(m, _)| m == mailbox) {
                    g.1.push(*uid);
                } else {
                    groups.push((mailbox.clone(), vec![*uid]));
                }
            }
            MessageRef::Pop3 { .. } => {
                return Err(EngineError::Unsupported(
                    "POP3 message ref on IMAP backend".into(),
                ));
            }
        }
    }
    Ok(groups)
}

/// The IDLE watch loop: on any mailbox activity emit `MailboxChanged`; renew the
/// IDLE before the 30-minute server limit; stop cleanly when signalled.
async fn watch_loop(config: ImapConfig, sink: ChangeSink, mut stop: watch::Receiver<bool>) {
    let mailbox = config.watch_mailbox.clone();
    let mut session = match connect_session(&config).await {
        Ok(s) => s,
        Err(_) => {
            let _ = sink.emit(ChangeEvent::Disconnected);
            return;
        }
    };
    if !session.caps().has("IDLE") {
        // No IDLE: signal once so the engine falls back to timed polling.
        let _ = sink.emit(ChangeEvent::MailboxChanged {
            mailbox: RawMailboxRef {
                name: mailbox,
                uidvalidity: 0,
            },
        });
        return;
    }
    if session.select(&mailbox, SelectMode::Plain).await.is_err() {
        let _ = sink.emit(ChangeEvent::Disconnected);
        return;
    }

    loop {
        let tag = match session.conn_mut().idle_start().await {
            Ok(t) => t,
            Err(_) => {
                let _ = sink.emit(ChangeEvent::Disconnected);
                return;
            }
        };
        let renew = tokio::time::sleep(Duration::from_secs(29 * 60));
        tokio::pin!(renew);
        let mut activity = false;
        let mut disconnected = false;

        loop {
            tokio::select! {
                _ = stop.changed() => {
                    let _ = session.conn_mut().idle_done(&tag).await;
                    let _ = session.logout().await;
                    return;
                }
                _ = &mut renew => break,
                res = session.conn_mut().idle_next() => match res {
                    Ok(resp) => {
                        if is_activity(&resp) {
                            activity = true;
                            break;
                        }
                    }
                    Err(_) => {
                        disconnected = true;
                        break;
                    }
                }
            }
        }

        if disconnected {
            let _ = sink.emit(ChangeEvent::Disconnected);
            return;
        }
        if session.conn_mut().idle_done(&tag).await.is_err() {
            let _ = sink.emit(ChangeEvent::Disconnected);
            return;
        }
        if activity {
            let event = ChangeEvent::MailboxChanged {
                mailbox: RawMailboxRef {
                    name: mailbox.clone(),
                    uidvalidity: 0,
                },
            };
            if sink.emit(event).is_err() {
                return;
            }
        }
    }
}

fn is_activity(resp: &crate::connection::OwnedResponse) -> bool {
    use imap_proto::{MailboxDatum, Response};
    matches!(
        resp,
        Response::Expunge(_)
            | Response::Vanished { .. }
            | Response::Fetch(_, _)
            | Response::MailboxData(MailboxDatum::Exists(_))
            | Response::MailboxData(MailboxDatum::Recent(_))
    )
}
