//! [`Pop3Backend`] — the [`AccountBackend`] implementation (plan §2.1).
//!
//! POP3 has one mailbox (`INBOX`), no server-side flags, and no folders, so
//! most of the trait collapses: sync is a UIDL-set diff, `store_flags` is a
//! no-op (the engine keeps flags locally), and `move`/`append` are
//! [`EngineError::Unsupported`]. Each call opens a fresh short POP3 session —
//! the natural unit of a locked maildrop — and `QUIT`s to commit any `DELE`s
//! required by the [`LeavePolicy`].

use std::collections::BTreeSet;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use mw_engine::backend::EngineError;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeEvent, ChangeSink, Flag, MailboxDelta, MailboxRole,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor,
    WatchHandle,
};
use tokio::sync::watch;

use crate::conn::Pop3Conn;
use crate::policy::{DeleteContext, LeavePolicy};

/// How the transport reaches the server.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TlsMode {
    /// Plaintext (test sockets / a local relay only).
    Plain,
    /// Implicit TLS from the first byte (POP3S, `:995`).
    #[default]
    Implicit,
    /// Opportunistic upgrade via `STLS` on the cleartext port (`:110`).
    StartTls,
}

/// Authentication method for the AUTHORIZATION state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Pop3Auth {
    /// Classic `USER`/`PASS` (RFC 1939 §7).
    #[default]
    UserPass,
    /// SASL `AUTH PLAIN`.
    SaslPlain,
    /// SASL `AUTH LOGIN`.
    SaslLogin,
    /// SASL `AUTH XOAUTH2` (Gmail/Outlook; `secret` is the access token).
    XOAuth2,
}

/// Everything needed to open and authenticate a POP3 session, plus the
/// leave-on-server policy the backend honours.
#[derive(Debug, Clone)]
pub struct Pop3Config {
    pub host: String,
    pub port: u16,
    pub tls: TlsMode,
    pub auth: Pop3Auth,
    pub username: String,
    /// Password, or the OAuth access token when `auth == XOAuth2`.
    pub secret: String,
    /// Retention policy governing `DELE` issuance.
    pub leave_policy: LeavePolicy,
    /// Interval between `watch` poll notifications (POP3 has no IDLE).
    pub poll_interval: Duration,
}

/// POP3 implementation of the account-backend seam.
#[derive(Debug, Clone)]
pub struct Pop3Backend {
    config: Pop3Config,
}

impl Pop3Backend {
    /// Build a backend from a full connection + policy configuration.
    pub fn new(config: Pop3Config) -> Self {
        Self { config }
    }

    /// The leave-on-server policy this backend enforces.
    pub fn leave_policy(&self) -> LeavePolicy {
        self.config.leave_policy
    }

    fn inbox_ref() -> RawMailboxRef {
        RawMailboxRef {
            name: "INBOX".to_string(),
            uidvalidity: 0,
        }
    }

    async fn open(&self) -> Result<Pop3Conn> {
        Pop3Conn::open(&self.config).await
    }
}

/// Pull the ingested-UIDL set out of a cursor, tolerating a fresh/mismatched
/// cursor by starting from empty (first sync of this mailbox).
fn seen_from_cursor(cursor: &SyncCursor) -> BTreeSet<String> {
    match cursor {
        SyncCursor::Pop3Uidl { seen } => seen.clone(),
        _ => BTreeSet::new(),
    }
}

/// Extract whole-day age from an RFC822 `Date:` header, if parseable.
fn age_days_from_headers(raw: &[u8]) -> Option<i64> {
    let text = String::from_utf8_lossy(raw);
    for line in text.lines() {
        if line.is_empty() {
            break; // end of headers
        }
        if let Some(rest) = line.get(..5)
            && rest.eq_ignore_ascii_case("date:")
        {
            let value = line[5..].trim();
            if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(value) {
                return Some((Utc::now() - dt.with_timezone(&Utc)).num_days());
            }
        }
    }
    None
}

#[async_trait]
impl AccountBackend for Pop3Backend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        let mut conn = self.open().await?;
        let capa = conn.capa().await?;
        conn.quit().await.ok();
        Ok(BackendCaps {
            sasl_plain: capa.sasl_plain(),
            sasl_login: capa.sasl_login(),
            sasl_xoauth2: capa.sasl_xoauth2(),
            // POP3 has no IMAP extensions, folders, flags, or push.
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        let mut conn = self.open().await?;
        let (count, _octets) = conn.stat().await?;
        conn.quit().await.ok();
        let total = u32::try_from(count).unwrap_or(u32::MAX);
        Ok(vec![RawMailbox {
            mailbox_ref: Self::inbox_ref(),
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: 0,
            highestmodseq: 0,
            total,
            // POP3 exposes no seen-state; the engine tracks read locally.
            unread: total,
        }])
    }

    async fn sync_mailbox(
        &self,
        _mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        let seen = seen_from_cursor(cursor);
        let policy = self.config.leave_policy;

        let mut conn = self.open().await?;
        let present = conn.uidl_all().await?; // (msg-number, uidl)

        let present_uidls: BTreeSet<String> = present.iter().map(|(_, u)| u.clone()).collect();

        // New = on the server but not yet ingested.
        let mut added = Vec::new();
        for (_, uidl) in &present {
            if !seen.contains(uidl) {
                added.push(MessageRef::Pop3 { uidl: uidl.clone() });
            }
        }

        // Removed = previously ingested but gone from the maildrop.
        let mut removed = Vec::new();
        for uidl in &seen {
            if !present_uidls.contains(uidl) {
                removed.push(MessageRef::Pop3 { uidl: uidl.clone() });
            }
        }

        // Age-based reaping (delete-after-N-days): only touch already-ingested
        // messages, and only when their `Date:` header proves them old enough.
        let mut reaped: BTreeSet<String> = BTreeSet::new();
        if policy.needs_age() {
            for (num, uidl) in &present {
                if !seen.contains(uidl) {
                    continue; // never reap on first contact
                }
                let headers = conn.top(*num, 0).await?;
                let ctx = DeleteContext {
                    just_retrieved: false,
                    previously_seen: true,
                    age_days: age_days_from_headers(&headers),
                };
                if policy.should_delete(ctx) {
                    conn.dele(*num).await?;
                    reaped.insert(uidl.clone());
                    removed.push(MessageRef::Pop3 { uidl: uidl.clone() });
                }
            }
        }

        conn.quit().await?; // commits any DELE from reaping

        // Next cursor = everything currently present minus what we just reaped.
        let next_seen: BTreeSet<String> = present_uidls.difference(&reaped).cloned().collect();

        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed,
            next_cursor: SyncCursor::Pop3Uidl { seen: next_seen },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        if refs.is_empty() {
            return Ok(Vec::new());
        }
        let delete_on_retrieval = self.config.leave_policy.should_delete(DeleteContext {
            just_retrieved: true,
            previously_seen: false,
            age_days: None,
        });

        let mut conn = self.open().await?;
        let present = conn.uidl_all().await?;
        let by_uidl: std::collections::HashMap<&str, u32> =
            present.iter().map(|(n, u)| (u.as_str(), *n)).collect();

        let mut out = Vec::with_capacity(refs.len());
        for r in refs {
            let uidl = match r {
                MessageRef::Pop3 { uidl } => uidl,
                MessageRef::Imap { .. } => {
                    return Err(EngineError::Unsupported(
                        "POP3 backend received an IMAP message ref".into(),
                    ));
                }
            };
            let Some(&num) = by_uidl.get(uidl.as_str()) else {
                // Gone from the maildrop since the caller learned of it; skip.
                continue;
            };
            let raw = conn.retr(num).await?;
            if delete_on_retrieval {
                conn.dele(num).await?;
            }
            out.push(RawMessage {
                message_ref: r.clone(),
                raw,
                flags: Vec::new(),
                internaldate: None,
            });
        }
        conn.quit().await?; // commits DELEs when delete-on-retrieval
        Ok(out)
    }

    async fn store_flags(
        &self,
        _refs: &[MessageRef],
        _add: &[Flag],
        _remove: &[Flag],
    ) -> Result<()> {
        // POP3 has no server-side flags; the engine keeps keywords locally.
        Ok(())
    }

    async fn move_messages(
        &self,
        _refs: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported(
            "POP3 has no folders; moves are engine-local".into(),
        ))
    }

    async fn append(
        &self,
        _mbox: &RawMailboxRef,
        _raw: &[u8],
        _flags: &[Flag],
    ) -> Result<MessageRef> {
        Err(EngineError::Unsupported(
            "POP3 cannot append; the engine files Sent locally".into(),
        ))
    }

    async fn watch(&self, sink: ChangeSink) -> Result<WatchHandle> {
        let (stop_tx, mut stop_rx) = watch::channel(false);
        let interval = self.config.poll_interval;
        let mailbox = Self::inbox_ref();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        if sink
                            .emit(ChangeEvent::MailboxChanged { mailbox: mailbox.clone() })
                            .is_err()
                        {
                            break; // engine dropped the receiver
                        }
                    }
                    changed = stop_rx.changed() => {
                        if changed.is_err() || *stop_rx.borrow() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(WatchHandle::new(stop_tx))
    }
}
