//! High-level IMAP session: authentication, mailbox enumeration, selection,
//! and the message commands (`UID FETCH`/`STORE`/`MOVE`/`APPEND`).
//!
//! A [`Session`] wraps one [`Connection`] plus the negotiated capability set and
//! the currently-selected mailbox. It converts `imap-proto` responses into the
//! frozen backend types ([`RawMailbox`], [`RawMessage`], [`MoveOutcome`], …).

use imap_proto::{
    AttributeValue, MailboxDatum, NameAttribute, Response, ResponseCode, SectionPath,
    StatusAttribute, UidSetMember,
};
use mw_engine::backend::{
    AclEntry, BackendCaps, Flag, MailboxRole, MessageRef, MetadataEntry, MoveOutcome, RawMailbox,
    RawMailboxRef, RawMessage,
};

use crate::caps::CapabilitySet;
use crate::connection::{Connection, Tagged};
use crate::error::{ImapError, ImapResult};
use crate::transport::TlsMode;

/// Credentials + mechanism selection for login.
#[derive(Clone)]
pub enum Credentials {
    /// Username + password; the session picks SASL PLAIN/LOGIN or the `LOGIN`
    /// command based on advertised capabilities.
    Password { username: String, password: String },
    /// OAuth2 bearer token (Gmail / Outlook) via SASL XOAUTH2.
    XOAuth2 { username: String, token: String },
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never render secrets.
        match self {
            Credentials::Password { username, .. } => f
                .debug_struct("Password")
                .field("username", username)
                .finish_non_exhaustive(),
            Credentials::XOAuth2 { username, .. } => f
                .debug_struct("XOAuth2")
                .field("username", username)
                .finish_non_exhaustive(),
        }
    }
}

/// How to SELECT a mailbox for a given sync strategy.
#[derive(Debug, Clone, Copy)]
pub enum SelectMode {
    /// Plain `SELECT` (UID-window / initial load).
    Plain,
    /// `SELECT (CONDSTORE)` to receive MODSEQ data.
    Condstore,
    /// `SELECT (QRESYNC (uidvalidity highestmodseq))` for a fast resync.
    Qresync {
        uidvalidity: u32,
        highestmodseq: u64,
    },
}

/// Everything harvested from a SELECT/EXAMINE response.
#[derive(Debug, Default)]
pub struct SelectResult {
    pub uidvalidity: u32,
    pub uidnext: u32,
    pub highestmodseq: u64,
    pub exists: u32,
    pub read_only: bool,
    /// VANISHED (EARLIER) UIDs reported during a QRESYNC select.
    pub vanished: Vec<u32>,
    /// FETCH items pushed during a QRESYNC select (changed since modseq).
    pub fetched: Vec<FetchItem>,
}

/// One parsed FETCH result, in backend terms.
#[derive(Debug, Default, Clone)]
pub struct FetchItem {
    pub uid: Option<u32>,
    pub flags: Vec<Flag>,
    pub internaldate: Option<String>,
    pub size: Option<u32>,
    pub body: Option<Vec<u8>>,
}

/// A live, authenticated (or pre-auth) IMAP session.
pub struct Session {
    conn: Connection,
    caps: CapabilitySet,
    selected: Option<String>,
    qresync_enabled: bool,
    host: String,
    port: u16,
}

impl Session {
    /// Connect, read the greeting, harvest any greeting-time capabilities, and
    /// (for STARTTLS) upgrade before authentication.
    pub async fn connect(host: &str, port: u16, mode: TlsMode) -> ImapResult<Self> {
        let (conn, greeting) = Connection::connect(host, port, mode).await?;
        let mut caps = CapabilitySet::default();
        if let Response::Data {
            code: Some(ResponseCode::Capabilities(c)),
            ..
        } = &greeting
        {
            caps.extend_from(c);
        }
        let mut session = Session {
            conn,
            caps,
            selected: None,
            qresync_enabled: false,
            host: host.to_string(),
            port,
        };
        if mode == TlsMode::StartTls && !session.conn.is_encrypted() {
            session.ensure_capabilities().await?;
            if !session.caps.has("STARTTLS") {
                return Err(ImapError::Unsupported("STARTTLS not advertised".into()));
            }
            session.conn.starttls().await?;
            // Capabilities MUST be re-fetched after the TLS upgrade.
            session.caps = CapabilitySet::default();
            session.probe_capabilities().await?;
        }
        Ok(session)
    }

    /// The negotiated capability flags for the frozen seam.
    pub fn backend_caps(&self) -> BackendCaps {
        self.caps.to_backend_caps()
    }

    /// Force a `CAPABILITY` round-trip, replacing the cached set.
    pub async fn probe_capabilities(&mut self) -> ImapResult<()> {
        let tagged = self.conn.execute("CAPABILITY").await?.ok()?;
        self.caps = CapabilitySet::default();
        for resp in &tagged.untagged {
            if let Response::Capabilities(c) = resp {
                self.caps.extend_from(c);
            }
        }
        if let Some(ResponseCode::Capabilities(c)) = &tagged.code {
            self.caps.extend_from(c);
        }
        Ok(())
    }

    async fn ensure_capabilities(&mut self) -> ImapResult<()> {
        if self.caps.is_empty() {
            self.probe_capabilities().await?;
        }
        Ok(())
    }

    /// Authenticate using the strongest mechanism the server + credentials allow.
    pub async fn login(&mut self, creds: &Credentials) -> ImapResult<()> {
        self.ensure_capabilities().await?;
        let tagged = match creds {
            Credentials::XOAuth2 { username, token } => {
                if self.caps.has_auth("XOAUTH2") {
                    let frame = crate::sasl::xoauth2(username, token);
                    self.conn.authenticate("XOAUTH2", &[frame]).await?
                } else if self.caps.has_auth("OAUTHBEARER") {
                    let frame = crate::sasl::oauthbearer(username, token, &self.host, self.port);
                    self.conn.authenticate("OAUTHBEARER", &[frame]).await?
                } else {
                    return Err(ImapError::Unsupported(
                        "neither AUTH=XOAUTH2 nor AUTH=OAUTHBEARER advertised".into(),
                    ));
                }
            }
            Credentials::Password { username, password } => {
                // Preference: SCRAM-SHA-256-PLUS (channel-bound) → SCRAM-SHA-256
                // → PLAIN → LOGIN → the LOGIN command. SCRAM keeps the password
                // off the wire; `-PLUS` additionally binds it to the TLS channel.
                let channel_binding = self.conn.channel_binding();
                if self.caps.has_auth("SCRAM-SHA-256-PLUS") && channel_binding.is_some() {
                    let mut client =
                        crate::sasl::ScramSha256::new(username, password, channel_binding);
                    self.conn
                        .authenticate_sasl("SCRAM-SHA-256-PLUS", &mut client)
                        .await?
                } else if self.caps.has_auth("SCRAM-SHA-256") {
                    let mut client = crate::sasl::ScramSha256::new(username, password, None);
                    self.conn
                        .authenticate_sasl("SCRAM-SHA-256", &mut client)
                        .await?
                } else if self.caps.has_auth("PLAIN") {
                    let frame = crate::sasl::plain(username, password);
                    self.conn.authenticate("PLAIN", &[frame]).await?
                } else if self.caps.has_auth("LOGIN") {
                    let frames = crate::sasl::login(username, password);
                    self.conn.authenticate("LOGIN", &frames).await?
                } else {
                    // Fall back to the plain LOGIN command.
                    let cmd = format!("LOGIN {} {}", quote(username), quote(password));
                    self.conn.execute(&cmd).await?
                }
            }
        };
        let tagged = tagged.ok().map_err(|e| match e {
            // A NO/BAD at the auth step is an auth failure, not a protocol fault.
            ImapError::No(m) | ImapError::Bad(m) => ImapError::Auth(m),
            other => other,
        })?;
        // Many servers return the post-auth capability list in the tagged OK.
        if let Some(ResponseCode::Capabilities(c)) = &tagged.code {
            self.caps.extend_from(c);
        } else {
            // Otherwise re-probe: pre-auth capabilities can differ from post-auth.
            self.probe_capabilities().await?;
        }
        Ok(())
    }

    /// Send `ID` (RFC 2971) advertising the client name; best-effort.
    pub async fn send_id(&mut self) -> ImapResult<()> {
        if !self.caps.has("ID") {
            return Ok(());
        }
        self.conn
            .execute("ID (\"name\" \"mailwoman\" \"version\" \"1\")")
            .await?
            .ok()?;
        Ok(())
    }

    /// `ENABLE` the sync extensions the server supports (QRESYNC implies
    /// CONDSTORE). No-op when `ENABLE` is unavailable.
    pub async fn enable_sync_extensions(&mut self) -> ImapResult<()> {
        if !self.caps.has("ENABLE") {
            return Ok(());
        }
        let mut exts = Vec::new();
        if self.caps.has("QRESYNC") {
            exts.push("QRESYNC");
        } else if self.caps.has("CONDSTORE") {
            exts.push("CONDSTORE");
        }
        if self.caps.has("IMAP4REV2") {
            exts.push("IMAP4rev2");
        }
        if exts.is_empty() {
            return Ok(());
        }
        let tagged = self
            .conn
            .execute(&format!("ENABLE {}", exts.join(" ")))
            .await?
            .ok()?;
        for resp in &tagged.untagged {
            if let Response::Capabilities(c) = resp {
                // ENABLED echoes the extensions now active.
                if c.iter().any(|cap| matches!(cap, imap_proto::Capability::Atom(a) if a.eq_ignore_ascii_case("QRESYNC")))
                {
                    self.qresync_enabled = true;
                }
            }
        }
        if self.caps.has("QRESYNC") {
            self.qresync_enabled = true;
        }
        Ok(())
    }

    async fn ensure_qresync_enabled(&mut self) -> ImapResult<()> {
        if !self.qresync_enabled && self.caps.has("QRESYNC") && self.caps.has("ENABLE") {
            self.conn.execute("ENABLE QRESYNC").await?.ok()?;
            self.qresync_enabled = true;
        }
        Ok(())
    }

    // --- Mailbox enumeration -------------------------------------------------

    /// Enumerate mailboxes with special-use roles and status counts.
    pub async fn list_mailboxes(&mut self) -> ImapResult<Vec<RawMailbox>> {
        let want_status = self.caps.list_status_supported();
        let status_items = self.status_item_list();

        let cmd = if want_status {
            format!("LIST \"\" \"*\" RETURN (STATUS ({status_items}))")
        } else {
            "LIST \"\" \"*\"".to_string()
        };
        let tagged = self.conn.execute(&cmd).await?.ok()?;

        let mut entries: Vec<ListEntry> = Vec::new();
        let mut status: std::collections::HashMap<String, StatusCounts> = Default::default();
        for resp in tagged.untagged {
            match resp {
                Response::MailboxData(MailboxDatum::List {
                    name_attributes,
                    delimiter,
                    name,
                }) => {
                    entries.push(ListEntry {
                        name: name.into_owned(),
                        delimiter: delimiter.map(|c| c.into_owned()),
                        role: role_from_attrs(&name_attributes),
                        selectable: !name_attributes
                            .iter()
                            .any(|a| matches!(a, NameAttribute::NoSelect)),
                    });
                }
                Response::MailboxData(MailboxDatum::Status {
                    mailbox,
                    status: attrs,
                }) => {
                    status.insert(mailbox.into_owned(), StatusCounts::from_attrs(&attrs));
                }
                _ => {}
            }
        }

        // For servers without LIST-STATUS, query STATUS per selectable mailbox.
        if !want_status {
            for entry in &entries {
                if !entry.selectable {
                    continue;
                }
                let cmd = format!("STATUS {} ({status_items})", quote(&entry.name));
                let tagged = self.conn.execute(&cmd).await?.ok()?;
                for resp in tagged.untagged {
                    if let Response::MailboxData(MailboxDatum::Status {
                        mailbox,
                        status: attrs,
                    }) = resp
                    {
                        status.insert(mailbox.into_owned(), StatusCounts::from_attrs(&attrs));
                    }
                }
            }
        }

        Ok(entries
            .into_iter()
            .map(|e| {
                let counts = status.get(&e.name).copied().unwrap_or_default();
                let role = if e.name.eq_ignore_ascii_case("INBOX") {
                    MailboxRole::Inbox
                } else {
                    e.role
                };
                let parent = parent_of(&e.name, e.delimiter.as_deref());
                RawMailbox {
                    mailbox_ref: RawMailboxRef {
                        name: e.name,
                        uidvalidity: counts.uidvalidity,
                    },
                    role,
                    parent,
                    uidnext: counts.uidnext,
                    highestmodseq: counts.highestmodseq,
                    total: counts.total,
                    unread: counts.unread,
                }
            })
            .collect())
    }

    fn status_item_list(&self) -> String {
        let mut items = String::from("MESSAGES UNSEEN UIDNEXT UIDVALIDITY");
        if self.caps.has("CONDSTORE") {
            items.push_str(" HIGHESTMODSEQ");
        }
        items
    }

    // --- Selection -----------------------------------------------------------

    /// SELECT a mailbox in the requested mode, parsing all status data.
    pub async fn select(&mut self, name: &str, mode: SelectMode) -> ImapResult<SelectResult> {
        let cmd = match mode {
            SelectMode::Plain => format!("SELECT {}", quote(name)),
            SelectMode::Condstore => format!("SELECT {} (CONDSTORE)", quote(name)),
            SelectMode::Qresync {
                uidvalidity,
                highestmodseq,
            } => {
                format!(
                    "SELECT {} (QRESYNC ({uidvalidity} {highestmodseq}))",
                    quote(name)
                )
            }
        };
        let tagged = self
            .conn
            .execute(&cmd)
            .await
            .map_err(|e| self.map_select_err(name, e))?;
        let tagged = tagged.ok().map_err(|e| self.map_select_err(name, e))?;

        let mut result = SelectResult::default();
        apply_select_code(&mut result, tagged.code.as_ref());
        for resp in tagged.untagged {
            match resp {
                Response::MailboxData(MailboxDatum::Exists(n)) => result.exists = n,
                Response::Data {
                    code: Some(code), ..
                } => apply_select_code(&mut result, Some(&code)),
                Response::Vanished { uids, .. } => expand_uid_ranges(&uids, &mut result.vanished),
                Response::Fetch(_, attrs) => result.fetched.push(parse_fetch(attrs)),
                _ => {}
            }
        }
        self.selected = Some(name.to_string());
        Ok(result)
    }

    fn map_select_err(&self, name: &str, e: ImapError) -> ImapError {
        match e {
            ImapError::No(m) | ImapError::Bad(m) => {
                ImapError::MailboxNotFound(format!("{name}: {m}"))
            }
            other => other,
        }
    }

    /// Ensure `name` is the selected mailbox (plain SELECT if not already).
    async fn ensure_selected(&mut self, name: &str) -> ImapResult<()> {
        if self.selected.as_deref() == Some(name) {
            return Ok(());
        }
        self.select(name, SelectMode::Plain).await?;
        Ok(())
    }

    // --- Fetch / search ------------------------------------------------------

    /// `UID FETCH <set> (FLAGS)(CHANGEDSINCE modseq)` — CONDSTORE flag deltas.
    pub async fn uid_fetch_changed(&mut self, modseq: u64) -> ImapResult<Vec<FetchItem>> {
        let cmd = format!("UID FETCH 1:* (UID FLAGS) (CHANGEDSINCE {modseq})");
        let tagged = self.conn.execute(&cmd).await?.ok()?;
        Ok(collect_fetches(tagged))
    }

    /// `UID SEARCH UID a:b` — bounded new-message discovery (avoids the `n:*`
    /// wrap-around gotcha by never using an open upper bound).
    pub async fn uid_search_range(&mut self, low: u32, high: u32) -> ImapResult<Vec<u32>> {
        if low > high {
            return Ok(Vec::new());
        }
        let cmd = format!("UID SEARCH UID {low}:{high}");
        self.uid_search(&cmd).await
    }

    /// `UID SEARCH ALL` — every UID in the selected mailbox.
    pub async fn uid_search_all(&mut self) -> ImapResult<Vec<u32>> {
        self.uid_search("UID SEARCH ALL").await
    }

    async fn uid_search(&mut self, cmd: &str) -> ImapResult<Vec<u32>> {
        let tagged = self.conn.execute(cmd).await?.ok()?;
        let mut uids = Vec::new();
        for resp in tagged.untagged {
            if let Response::MailboxData(MailboxDatum::Search(mut s)) = resp {
                uids.append(&mut s);
            }
        }
        Ok(uids)
    }

    // --- SORT / THREAD (RFC 5256) --------------------------------------------

    /// `UID SORT (<criteria>) UTF-8 <search>` (RFC 5256): server-side ordering.
    ///
    /// Returns the matching UIDs in the requested order. `search` is an IMAP
    /// search-key string (`ALL` when empty). Errors if the server does not
    /// advertise `SORT`.
    pub async fn uid_sort(
        &mut self,
        criteria: &[SortCriterion],
        search: &str,
    ) -> ImapResult<Vec<u32>> {
        if !self.caps.has("SORT") {
            return Err(ImapError::Unsupported(
                "SORT (RFC 5256) not advertised".into(),
            ));
        }
        let keys = criteria
            .iter()
            .map(|c| c.render())
            .collect::<Vec<_>>()
            .join(" ");
        let search = if search.trim().is_empty() {
            "ALL"
        } else {
            search
        };
        let cmd = format!("UID SORT ({keys}) UTF-8 {search}");
        let tagged = self.conn.execute(&cmd).await?.ok()?;
        let mut uids = Vec::new();
        for resp in tagged.untagged {
            if let Response::MailboxData(MailboxDatum::Sort(mut s)) = resp {
                uids.append(&mut s);
            }
        }
        Ok(uids)
    }

    /// `UID THREAD <algorithm> UTF-8 <search>` (RFC 5256): server-side threading.
    ///
    /// Returns the thread forest as [`ThreadNode`] roots. `imap-proto` does not
    /// model the `* THREAD (…)` reply, so it is read and parsed at the raw-line
    /// level ([`Connection::execute_lines`]). Errors if the server does not
    /// advertise the requested `THREAD=` algorithm.
    pub async fn uid_thread(
        &mut self,
        algorithm: ThreadAlgorithm,
        search: &str,
    ) -> ImapResult<Vec<ThreadNode>> {
        if !self.caps.has(algorithm.cap()) {
            return Err(ImapError::Unsupported(format!(
                "{} not advertised",
                algorithm.cap()
            )));
        }
        let search = if search.trim().is_empty() {
            "ALL"
        } else {
            search
        };
        let cmd = format!("UID THREAD {} UTF-8 {search}", algorithm.token());
        let lines = self.conn.execute_lines(&cmd).await?;
        for line in lines {
            if let Some(rest) = line.strip_prefix("* THREAD") {
                return Ok(parse_thread_response(rest.trim()));
            }
        }
        Ok(Vec::new())
    }

    /// Fetch full raw RFC822 bytes for a set of UIDs in an already-known mailbox.
    pub async fn fetch_raw(
        &mut self,
        mailbox: &RawMailboxRef,
        uids: &[u32],
    ) -> ImapResult<Vec<RawMessage>> {
        if uids.is_empty() {
            return Ok(Vec::new());
        }
        self.ensure_selected(&mailbox.name).await?;
        let set = format_uid_set(uids);
        let cmd = format!("UID FETCH {set} (UID FLAGS INTERNALDATE RFC822.SIZE BODY.PEEK[])");
        let tagged = self.conn.execute(&cmd).await?.ok()?;
        let uidvalidity = mailbox.uidvalidity;
        let mut out = Vec::new();
        for item in collect_fetches(tagged) {
            let Some(uid) = item.uid else { continue };
            let Some(raw) = item.body else { continue };
            out.push(RawMessage {
                message_ref: MessageRef::Imap {
                    mailbox: mailbox.clone(),
                    uidvalidity,
                    uid,
                },
                raw,
                flags: item.flags,
                internaldate: item.internaldate,
            });
        }
        Ok(out)
    }

    // --- Store / move / append ----------------------------------------------

    /// Apply flag additions/removals to a set of UIDs in one mailbox.
    pub async fn store_flags(
        &mut self,
        mailbox: &RawMailboxRef,
        uids: &[u32],
        add: &[Flag],
        remove: &[Flag],
    ) -> ImapResult<()> {
        if uids.is_empty() {
            return Ok(());
        }
        self.ensure_selected(&mailbox.name).await?;
        let set = format_uid_set(uids);
        if !add.is_empty() {
            let flags = add.iter().map(flag_to_imap).collect::<Vec<_>>().join(" ");
            self.conn
                .execute(&format!("UID STORE {set} +FLAGS.SILENT ({flags})"))
                .await?
                .ok()?;
        }
        if !remove.is_empty() {
            let flags = remove
                .iter()
                .map(flag_to_imap)
                .collect::<Vec<_>>()
                .join(" ");
            self.conn
                .execute(&format!("UID STORE {set} -FLAGS.SILENT ({flags})"))
                .await?
                .ok()?;
        }
        Ok(())
    }

    /// Move UIDs from `mailbox` to `dest`, using UID MOVE or COPY+EXPUNGE.
    pub async fn move_messages(
        &mut self,
        mailbox: &RawMailboxRef,
        uids: &[u32],
        dest: &str,
    ) -> ImapResult<MoveOutcome> {
        if uids.is_empty() {
            return Ok(MoveOutcome::RederiveByMessageId);
        }
        self.ensure_selected(&mailbox.name).await?;
        let set = format_uid_set(uids);
        if self.caps.has("MOVE") {
            let tagged = self
                .conn
                .execute(&format!("UID MOVE {set} {}", quote(dest)))
                .await?
                .ok()?;
            Ok(copyuid_outcome(&tagged))
        } else {
            let copy = self
                .conn
                .execute(&format!("UID COPY {set} {}", quote(dest)))
                .await?
                .ok()?;
            let outcome = copyuid_outcome(&copy);
            self.conn
                .execute(&format!("UID STORE {set} +FLAGS.SILENT (\\Deleted)"))
                .await?
                .ok()?;
            if self.caps.has("UIDPLUS") {
                self.conn
                    .execute(&format!("UID EXPUNGE {set}"))
                    .await?
                    .ok()?;
            } else {
                self.conn.execute("EXPUNGE").await?.ok()?;
            }
            Ok(outcome)
        }
    }

    /// APPEND a message; returns a ref carrying the APPENDUID coordinates when
    /// UIDPLUS is present, else a ref with `uid == 0` for the engine to re-derive.
    pub async fn append(
        &mut self,
        mailbox: &str,
        raw: &[u8],
        flags: &[Flag],
    ) -> ImapResult<MessageRef> {
        let flag_str = if flags.is_empty() {
            String::new()
        } else {
            format!(
                " ({})",
                flags.iter().map(flag_to_imap).collect::<Vec<_>>().join(" ")
            )
        };
        let head = format!("APPEND {}{flag_str}", quote(mailbox));
        let tagged = self.conn.execute_with_literal(&head, raw).await?;
        // A new message invalidates the selected-mailbox message counts.
        self.selected = None;
        if let Some(ResponseCode::AppendUid(uidvalidity, uidset)) = &tagged.code {
            let uid = uidset.first().map(uidset_member_first).unwrap_or(0);
            return Ok(MessageRef::Imap {
                mailbox: RawMailboxRef {
                    name: mailbox.to_string(),
                    uidvalidity: *uidvalidity,
                },
                uidvalidity: *uidvalidity,
                uid,
            });
        }
        Ok(MessageRef::Imap {
            mailbox: RawMailboxRef {
                name: mailbox.to_string(),
                uidvalidity: 0,
            },
            uidvalidity: 0,
            uid: 0,
        })
    }

    /// Graceful LOGOUT.
    pub async fn logout(&mut self) -> ImapResult<()> {
        self.conn.execute("LOGOUT").await?;
        Ok(())
    }

    // --- ACL (RFC 4314) + METADATA (RFC 5464) --------------------------------
    //
    // `mw-imap` is the CLIENT: these commands are sent to the upstream IMAP
    // server, which is the real ACL enforcement point. Each is guarded behind
    // the advertised capability (`ACL` / `METADATA`|`METADATA-SERVER`) and returns
    // `Unsupported` otherwise. `imap-proto` models none of the `ACL`/`LISTRIGHTS`/
    // `MYRIGHTS`/`METADATA` untagged replies, so — as with `THREAD` — they are
    // read and parsed at the raw-line level via [`Connection::execute_lines`].

    fn require_acl(&self) -> ImapResult<()> {
        if self.caps.has("ACL") {
            Ok(())
        } else {
            Err(ImapError::Unsupported(
                "ACL (RFC 4314) not advertised".into(),
            ))
        }
    }

    fn require_metadata(&self) -> ImapResult<()> {
        if self.caps.has("METADATA") || self.caps.has("METADATA-SERVER") {
            Ok(())
        } else {
            Err(ImapError::Unsupported(
                "METADATA (RFC 5464) not advertised".into(),
            ))
        }
    }

    /// `GETACL <mailbox>` (RFC 4314): every `identifier`→`rights` grant on the
    /// mailbox, parsed from the untagged `* ACL <mailbox> <id> <rights> …` reply.
    pub async fn get_acl(&mut self, mailbox: &str) -> ImapResult<Vec<AclEntry>> {
        self.require_acl()?;
        let cmd = format!("GETACL {}", quote(mailbox));
        let lines = self.conn.execute_lines(&cmd).await?;
        for line in lines {
            if let Some(rest) = untagged_payload(&line, "ACL") {
                return Ok(parse_acl(rest));
            }
        }
        Ok(Vec::new())
    }

    /// `SETACL <mailbox> <identifier> <rights>` (RFC 4314): set (replace) the
    /// rights `identifier` holds on the mailbox.
    pub async fn set_acl(
        &mut self,
        mailbox: &str,
        identifier: &str,
        rights: &str,
    ) -> ImapResult<()> {
        self.require_acl()?;
        let cmd = format!(
            "SETACL {} {} {}",
            quote(mailbox),
            quote(identifier),
            quote(rights)
        );
        self.conn.execute(&cmd).await?.ok()?;
        Ok(())
    }

    /// `DELETEACL <mailbox> <identifier>` (RFC 4314): remove `identifier`'s
    /// entire ACL entry on the mailbox.
    pub async fn delete_acl(&mut self, mailbox: &str, identifier: &str) -> ImapResult<()> {
        self.require_acl()?;
        let cmd = format!("DELETEACL {} {}", quote(mailbox), quote(identifier));
        self.conn.execute(&cmd).await?.ok()?;
        Ok(())
    }

    /// `LISTRIGHTS <mailbox> <identifier>` (RFC 4314): the rights tokens that may
    /// be granted to `identifier` (first token = always-granted set, the rest =
    /// individually-optional rights), parsed from the untagged `* LISTRIGHTS`
    /// reply.
    pub async fn list_rights(
        &mut self,
        mailbox: &str,
        identifier: &str,
    ) -> ImapResult<Vec<String>> {
        self.require_acl()?;
        let cmd = format!("LISTRIGHTS {} {}", quote(mailbox), quote(identifier));
        let lines = self.conn.execute_lines(&cmd).await?;
        for line in lines {
            if let Some(rest) = untagged_payload(&line, "LISTRIGHTS") {
                // Tokens after the mailbox + identifier are the rights tokens.
                return Ok(tokenize(rest).into_iter().skip(2).map(tok_value).collect());
            }
        }
        Ok(Vec::new())
    }

    /// `MYRIGHTS <mailbox>` (RFC 4314): the rights string the authenticated user
    /// holds on the mailbox, parsed from the untagged `* MYRIGHTS` reply.
    pub async fn my_rights(&mut self, mailbox: &str) -> ImapResult<String> {
        self.require_acl()?;
        let cmd = format!("MYRIGHTS {}", quote(mailbox));
        let lines = self.conn.execute_lines(&cmd).await?;
        for line in lines {
            if let Some(rest) = untagged_payload(&line, "MYRIGHTS") {
                // `* MYRIGHTS <mailbox> <rights>` — the rights is the 2nd token.
                if let Some(r) = tokenize(rest).into_iter().nth(1) {
                    return Ok(tok_value(r));
                }
            }
        }
        Ok(String::new())
    }

    /// `GETMETADATA <mailbox> (<entry> …)` (RFC 5464): fetch annotation values.
    /// A server-level lookup uses the empty mailbox name (`""`). Parsed from the
    /// untagged `* METADATA <mailbox> (<entry> <value> …)` reply; a missing entry
    /// (NIL) yields a [`MetadataEntry`] with `value == None`.
    pub async fn get_metadata(
        &mut self,
        mailbox: &str,
        entries: &[String],
    ) -> ImapResult<Vec<MetadataEntry>> {
        self.require_metadata()?;
        let entry_list = entries.join(" ");
        let cmd = format!("GETMETADATA {} ({entry_list})", quote(mailbox));
        let lines = self.conn.execute_lines(&cmd).await?;
        // RFC 5464 servers (Dovecot always) return values as IMAP synchronizing
        // literals whose octet count spans the CRLF `execute_lines` stripped
        // between lines. Reassemble the reply with those CRLFs restored so the
        // literal-aware tokenizer consumes each value octet-exactly instead of
        // leaking the `{n}` length marker.
        let blob = lines.join("\r\n");
        Ok(parse_metadata(&blob))
    }

    /// `SETMETADATA <mailbox> (<entry> <value>)` (RFC 5464): set one annotation,
    /// or remove it when `value == None` (RFC 5464 NIL). Server-level uses `""`.
    pub async fn set_metadata(
        &mut self,
        mailbox: &str,
        entry: &str,
        value: Option<&str>,
    ) -> ImapResult<()> {
        self.require_metadata()?;
        let rendered = match value {
            Some(v) => quote(v),
            None => "NIL".to_string(),
        };
        let cmd = format!("SETMETADATA {} ({entry} {rendered})", quote(mailbox));
        self.conn.execute(&cmd).await?.ok()?;
        Ok(())
    }

    // --- accessors used by the sync ladder + backend -------------------------

    pub(crate) fn caps(&self) -> &CapabilitySet {
        &self.caps
    }

    pub(crate) fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    pub(crate) async fn ensure_qresync(&mut self) -> ImapResult<()> {
        self.ensure_qresync_enabled().await
    }
}

// --- SORT / THREAD types (RFC 5256) -----------------------------------------

/// A SORT (RFC 5256) ordering key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Arrival,
    Cc,
    Date,
    From,
    Size,
    Subject,
    To,
}

impl SortKey {
    fn token(self) -> &'static str {
        match self {
            SortKey::Arrival => "ARRIVAL",
            SortKey::Cc => "CC",
            SortKey::Date => "DATE",
            SortKey::From => "FROM",
            SortKey::Size => "SIZE",
            SortKey::Subject => "SUBJECT",
            SortKey::To => "TO",
        }
    }
}

/// One SORT ordering term: a [`SortKey`] optionally in reverse (RFC 5256 allows
/// `REVERSE` per key).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortCriterion {
    pub key: SortKey,
    pub reverse: bool,
}

impl SortCriterion {
    /// Ascending order on `key`.
    pub fn asc(key: SortKey) -> Self {
        Self {
            key,
            reverse: false,
        }
    }

    /// Descending (`REVERSE`) order on `key`.
    pub fn desc(key: SortKey) -> Self {
        Self { key, reverse: true }
    }

    fn render(self) -> String {
        if self.reverse {
            format!("REVERSE {}", self.key.token())
        } else {
            self.key.token().to_string()
        }
    }
}

/// A THREAD (RFC 5256) threading algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadAlgorithm {
    /// ORDEREDSUBJECT — subject + date "poor man's threading".
    OrderedSubject,
    /// REFERENCES — References/In-Reply-To message-graph threading.
    References,
}

impl ThreadAlgorithm {
    fn token(self) -> &'static str {
        match self {
            ThreadAlgorithm::OrderedSubject => "ORDEREDSUBJECT",
            ThreadAlgorithm::References => "REFERENCES",
        }
    }

    fn cap(self) -> &'static str {
        match self {
            ThreadAlgorithm::OrderedSubject => "THREAD=ORDEREDSUBJECT",
            ThreadAlgorithm::References => "THREAD=REFERENCES",
        }
    }
}

/// A node in a THREAD (RFC 5256) tree: a message UID and its child threads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadNode {
    /// The message's UID (this is a `UID THREAD` reply).
    pub id: u32,
    /// Reply threads rooted at this message.
    pub children: Vec<ThreadNode>,
}

/// One token of a THREAD reply body: a number or a parenthesised sub-thread.
enum ThreadElem {
    Num(u32),
    List(Vec<ThreadElem>),
}

/// Parse a `* THREAD` reply body (e.g. `(2)(3 6 (4 23)(44 7 96))`) into roots.
fn parse_thread_response(body: &str) -> Vec<ThreadNode> {
    let bytes = body.as_bytes();
    let mut pos = 0;
    let elems = parse_thread_elems(bytes, &mut pos);
    thread_members(&elems)
}

/// Recursive-descent over the THREAD grammar: a sequence of numbers and
/// parenthesised sub-lists, stopping at the matching `)` or end of input.
fn parse_thread_elems(bytes: &[u8], pos: &mut usize) -> Vec<ThreadElem> {
    let mut elems = Vec::new();
    while *pos < bytes.len() {
        match bytes[*pos] {
            b' ' => *pos += 1,
            b'(' => {
                *pos += 1;
                elems.push(ThreadElem::List(parse_thread_elems(bytes, pos)));
            }
            b')' => {
                *pos += 1;
                break;
            }
            b'0'..=b'9' => {
                let start = *pos;
                while *pos < bytes.len() && bytes[*pos].is_ascii_digit() {
                    *pos += 1;
                }
                let n = std::str::from_utf8(&bytes[start..*pos])
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                elems.push(ThreadElem::Num(n));
            }
            _ => *pos += 1,
        }
    }
    elems
}

/// Fold an element sequence into thread roots. Leading numbers form a
/// parent→child chain; trailing sub-lists attach as children of the last
/// number. With no leading number, each sub-list is an independent sibling root
/// (the top-level `(A)(B)(C)` case).
fn thread_members(elems: &[ThreadElem]) -> Vec<ThreadNode> {
    let mut nums = Vec::new();
    let mut i = 0;
    while let Some(ThreadElem::Num(n)) = elems.get(i) {
        nums.push(*n);
        i += 1;
    }
    let mut branch_children = Vec::new();
    for elem in &elems[i..] {
        if let ThreadElem::List(inner) = elem {
            branch_children.extend(thread_members(inner));
        }
    }
    if nums.is_empty() {
        return branch_children;
    }
    let mut node = ThreadNode {
        id: *nums.last().unwrap(),
        children: branch_children,
    };
    for &n in nums[..nums.len() - 1].iter().rev() {
        node = ThreadNode {
            id: n,
            children: vec![node],
        };
    }
    vec![node]
}

// --- free helpers -----------------------------------------------------------

struct ListEntry {
    name: String,
    delimiter: Option<String>,
    role: MailboxRole,
    selectable: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct StatusCounts {
    total: u32,
    unread: u32,
    uidnext: u32,
    uidvalidity: u32,
    highestmodseq: u64,
}

impl StatusCounts {
    fn from_attrs(attrs: &[StatusAttribute]) -> Self {
        let mut c = StatusCounts::default();
        for a in attrs {
            match a {
                StatusAttribute::Messages(n) => c.total = *n,
                StatusAttribute::Unseen(n) => c.unread = *n,
                StatusAttribute::UidNext(n) => c.uidnext = *n,
                StatusAttribute::UidValidity(n) => c.uidvalidity = *n,
                StatusAttribute::HighestModSeq(n) => c.highestmodseq = *n,
                _ => {}
            }
        }
        c
    }
}

impl CapabilitySet {
    fn list_status_supported(&self) -> bool {
        self.has("LIST-STATUS")
    }
}

fn role_from_attrs(attrs: &[NameAttribute<'_>]) -> MailboxRole {
    for a in attrs {
        let role = match a {
            NameAttribute::All => MailboxRole::All,
            NameAttribute::Archive => MailboxRole::Archive,
            NameAttribute::Drafts => MailboxRole::Drafts,
            NameAttribute::Flagged => MailboxRole::Flagged,
            NameAttribute::Junk => MailboxRole::Junk,
            NameAttribute::Sent => MailboxRole::Sent,
            NameAttribute::Trash => MailboxRole::Trash,
            _ => continue,
        };
        return role;
    }
    MailboxRole::None
}

fn parent_of(name: &str, delimiter: Option<&str>) -> Option<String> {
    let delim = delimiter?;
    if delim.is_empty() {
        return None;
    }
    let idx = name.rfind(delim)?;
    Some(name[..idx].to_string())
}

fn apply_select_code(result: &mut SelectResult, code: Option<&ResponseCode<'_>>) {
    match code {
        Some(ResponseCode::UidValidity(v)) => result.uidvalidity = *v,
        Some(ResponseCode::UidNext(v)) => result.uidnext = *v,
        Some(ResponseCode::HighestModSeq(v)) => result.highestmodseq = *v,
        Some(ResponseCode::ReadOnly) => result.read_only = true,
        Some(ResponseCode::ReadWrite) => result.read_only = false,
        _ => {}
    }
}

fn expand_uid_ranges(ranges: &[std::ops::RangeInclusive<u32>], out: &mut Vec<u32>) {
    for r in ranges {
        // Guard against absurd ranges; real VANISHED sets are bounded.
        let (start, end) = (*r.start(), *r.end());
        if end.saturating_sub(start) > 1_000_000 {
            continue;
        }
        out.extend(start..=end);
    }
}

fn parse_fetch(attrs: Vec<AttributeValue<'static>>) -> FetchItem {
    let mut item = FetchItem::default();
    for attr in attrs {
        match attr {
            AttributeValue::Uid(u) => item.uid = Some(u),
            AttributeValue::Flags(f) => item.flags = f.iter().map(|s| imap_to_flag(s)).collect(),
            AttributeValue::InternalDate(d) => item.internaldate = Some(d.into_owned()),
            AttributeValue::Rfc822Size(s) => item.size = Some(s),
            AttributeValue::BodySection {
                section: None | Some(SectionPath::Full(_)),
                data,
                ..
            } => {
                if let Some(d) = data {
                    item.body = Some(d.into_owned());
                }
            }
            AttributeValue::Rfc822(Some(d)) => item.body = Some(d.into_owned()),
            _ => {}
        }
    }
    item
}

fn collect_fetches(tagged: Tagged) -> Vec<FetchItem> {
    tagged
        .untagged
        .into_iter()
        .filter_map(|r| match r {
            Response::Fetch(_, attrs) => Some(parse_fetch(attrs)),
            _ => None,
        })
        .collect()
}

fn copyuid_outcome(tagged: &Tagged) -> MoveOutcome {
    // COPYUID may appear in the tagged completion or an untagged OK.
    if let Some(ResponseCode::CopyUid(uidvalidity, _src, dst)) = &tagged.code {
        return MoveOutcome::Uidplus {
            uidvalidity: *uidvalidity,
            uids: expand_uidset(dst),
        };
    }
    for resp in &tagged.untagged {
        if let Response::Data {
            code: Some(ResponseCode::CopyUid(uidvalidity, _src, dst)),
            ..
        } = resp
        {
            return MoveOutcome::Uidplus {
                uidvalidity: *uidvalidity,
                uids: expand_uidset(dst),
            };
        }
    }
    MoveOutcome::RederiveByMessageId
}

fn expand_uidset(members: &[UidSetMember]) -> Vec<u32> {
    let mut out = Vec::new();
    for m in members {
        match m {
            UidSetMember::Uid(u) => out.push(*u),
            UidSetMember::UidRange(r) => {
                let (s, e) = (*r.start(), *r.end());
                if e.saturating_sub(s) <= 1_000_000 {
                    out.extend(s..=e);
                }
            }
        }
    }
    out
}

fn uidset_member_first(m: &UidSetMember) -> u32 {
    match m {
        UidSetMember::Uid(u) => *u,
        UidSetMember::UidRange(r) => *r.start(),
    }
}

/// Collapse a UID list into a compact IMAP sequence set (`1,3:5,9`).
pub(crate) fn format_uid_set(uids: &[u32]) -> String {
    let mut sorted: Vec<u32> = uids.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let start = sorted[i];
        let mut j = i;
        while j + 1 < sorted.len() && sorted[j + 1] == sorted[j] + 1 {
            j += 1;
        }
        if j == i {
            parts.push(start.to_string());
        } else {
            parts.push(format!("{start}:{}", sorted[j]));
        }
        i = j + 1;
    }
    parts.join(",")
}

fn flag_to_imap(f: &Flag) -> String {
    match f {
        Flag::Seen => "\\Seen".into(),
        Flag::Answered => "\\Answered".into(),
        Flag::Flagged => "\\Flagged".into(),
        Flag::Deleted => "\\Deleted".into(),
        Flag::Draft => "\\Draft".into(),
        Flag::Recent => "\\Recent".into(),
        Flag::Keyword(k) => k.clone(),
    }
}

fn imap_to_flag(s: &str) -> Flag {
    let t = s.strip_prefix('\\').unwrap_or(s);
    match t.to_ascii_lowercase().as_str() {
        "seen" => Flag::Seen,
        "answered" => Flag::Answered,
        "flagged" => Flag::Flagged,
        "deleted" => Flag::Deleted,
        "draft" => Flag::Draft,
        "recent" => Flag::Recent,
        _ => Flag::Keyword(s.to_string()),
    }
}

// --- ACL / METADATA response parsing (RFC 4314 / RFC 5464) ------------------

/// If `line` is an untagged `* <KEYWORD> …` response, return the payload after
/// the keyword (trimmed). The keyword must be followed by whitespace (or end of
/// line) so `MYRIGHTS` never matches for `METADATA` and vice-versa.
fn untagged_payload<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = line.strip_prefix("* ")?.strip_prefix(keyword)?;
    match rest.chars().next() {
        Some(c) if c.is_whitespace() => Some(rest.trim_start()),
        None => Some(""),
        _ => None,
    }
}

/// One token of an IMAP response fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Open,
    Close,
    /// A quoted-string value (quotes stripped, `\` escapes resolved).
    Str(String),
    /// A bare atom (mailbox name, entry name, rights, or `NIL`).
    Atom(String),
}

/// The string carried by a value token (both quoted strings and atoms).
fn tok_value(t: Tok) -> String {
    match t {
        Tok::Str(s) | Tok::Atom(s) => s,
        Tok::Open => "(".to_string(),
        Tok::Close => ")".to_string(),
    }
}

/// Tokenize an IMAP response fragment, honoring quoted-strings, parentheses, and
/// RFC 3501 synchronizing literals (`{n}` — optionally the non-synchronizing
/// `{n+}` — followed by CRLF and then exactly `n` octets).
///
/// Literals are read **octet-exactly**: the count spans the CRLF that terminates
/// the `{n}` marker *and* any CRLFs inside the value itself. Callers that read a
/// multi-line reply line-by-line (stripping the terminating CRLFs) must therefore
/// reassemble it with those inter-line CRLFs restored before tokenizing — see
/// [`Session::get_metadata`], which is why RFC 5464 METADATA values (Dovecot
/// always returns them as literals) round-trip instead of leaking the `{n}` marker.
fn tokenize(s: &str) -> Vec<Tok> {
    let bytes = s.as_bytes();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'(' => {
                toks.push(Tok::Open);
                i += 1;
            }
            b')' => {
                toks.push(Tok::Close);
                i += 1;
            }
            b'"' => {
                i += 1; // opening quote
                let mut val = Vec::new();
                while i < bytes.len() {
                    match bytes[i] {
                        b'\\' => {
                            i += 1;
                            if i < bytes.len() {
                                val.push(bytes[i]);
                                i += 1;
                            }
                        }
                        b'"' => {
                            i += 1;
                            break;
                        }
                        other => {
                            val.push(other);
                            i += 1;
                        }
                    }
                }
                toks.push(Tok::Str(String::from_utf8_lossy(&val).into_owned()));
            }
            b'{' => {
                // A synchronizing literal marker: read the `n` octets that follow
                // its terminating CRLF as the value. On a malformed marker, fall
                // back to treating `{` as the start of a bare atom.
                if let Some(lit) = read_literal(bytes, &mut i) {
                    toks.push(Tok::Str(lit));
                } else {
                    toks.push(Tok::Atom(read_atom(bytes, &mut i)));
                }
            }
            _ => toks.push(Tok::Atom(read_atom(bytes, &mut i))),
        }
    }
    toks
}

/// Read a bare atom starting at `*pos`, advancing past it. Stops at whitespace or
/// a parenthesis.
fn read_atom(bytes: &[u8], pos: &mut usize) -> String {
    let start = *pos;
    while *pos < bytes.len() && !matches!(bytes[*pos], b' ' | b'\t' | b'\r' | b'\n' | b'(' | b')') {
        *pos += 1;
    }
    String::from_utf8_lossy(&bytes[start..*pos]).into_owned()
}

/// At a `{` literal marker, parse `{n}` (or the non-synchronizing `{n+}`), skip
/// the CRLF that terminates the marker, and return the following `n` octets as a
/// string — advancing `*pos` past them. Returns `None` (leaving `*pos` unmoved) if
/// the bytes at `*pos` are not a well-formed literal marker, so the caller can
/// treat `{` as an ordinary atom character.
fn read_literal(bytes: &[u8], pos: &mut usize) -> Option<String> {
    debug_assert_eq!(bytes[*pos], b'{');
    let close = bytes[*pos + 1..].iter().position(|&b| b == b'}')? + *pos + 1;
    let mut digits = &bytes[*pos + 1..close];
    // Tolerate the non-synchronizing `{n+}` form (client→server, but be lenient).
    if digits.last() == Some(&b'+') {
        digits = &digits[..digits.len() - 1];
    }
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let n: usize = std::str::from_utf8(digits).ok()?.parse().ok()?;
    let mut start = close + 1;
    // The literal-length marker is terminated by CRLF (accept a bare LF too).
    if bytes.get(start) == Some(&b'\r') {
        start += 1;
    }
    if bytes.get(start) == Some(&b'\n') {
        start += 1;
    }
    let end = (start + n).min(bytes.len());
    *pos = end;
    Some(String::from_utf8_lossy(&bytes[start..end]).into_owned())
}

/// Parse a `* ACL <mailbox> <id> <rights> …` payload into grants. The leading
/// mailbox token is dropped; the remainder are `(identifier, rights)` pairs.
fn parse_acl(payload: &str) -> Vec<AclEntry> {
    let mut iter = tokenize(payload).into_iter().skip(1);
    let mut out = Vec::new();
    while let (Some(id), Some(rights)) = (iter.next(), iter.next()) {
        out.push(AclEntry {
            identifier: tok_value(id),
            rights: tok_value(rights),
        });
    }
    out
}

/// Parse a `* METADATA <mailbox> (<entry> <value> …)` reply into annotations.
///
/// Each `(entry value …)` list is a run of `entry`/`value` pairs. A value may be a
/// **quoted string**, an IMAP **synchronizing literal** (`{n}` + payload — the form
/// Dovecot and every RFC 5464 server actually returns, reassembled octet-exactly by
/// [`tokenize`]), or **NIL** (no value / a removed entry → `None`). Tokens outside a
/// paren list (the `* METADATA` keyword and the mailbox name) are skipped, so this
/// accepts either the bare payload or the full untagged line, and tolerates more
/// than one `* METADATA` response reassembled into `payload`.
fn parse_metadata(payload: &str) -> Vec<MetadataEntry> {
    let toks = tokenize(payload);
    let mut out = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        // Skip everything up to the opening paren of an (entry value …) list.
        if toks[i] != Tok::Open {
            i += 1;
            continue;
        }
        i += 1; // consume Open
        while i < toks.len() && toks[i] != Tok::Close {
            let entry = tok_value(toks[i].clone());
            i += 1;
            let value = match toks.get(i) {
                Some(Tok::Str(s)) => {
                    i += 1;
                    Some(s.clone())
                }
                Some(Tok::Atom(a)) if a.eq_ignore_ascii_case("NIL") => {
                    i += 1;
                    None
                }
                Some(Tok::Atom(a)) => {
                    let v = a.clone();
                    i += 1;
                    Some(v)
                }
                // A dangling entry with no value token (malformed / truncated).
                Some(Tok::Open) | Some(Tok::Close) | None => None,
            };
            out.push(MetadataEntry { entry, value });
        }
        if i < toks.len() {
            i += 1; // consume Close
        }
    }
    out
}

/// Quote a mailbox/argument as an IMAP quoted-string.
pub(crate) fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        if ch == '"' || ch == '\\' {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push('"');
    out
}

#[cfg(test)]
mod sort_thread_tests {
    use super::*;

    fn leaf(id: u32) -> ThreadNode {
        ThreadNode {
            id,
            children: vec![],
        }
    }

    #[test]
    fn sort_criteria_render_with_reverse() {
        let crit = [
            SortCriterion::desc(SortKey::Date),
            SortCriterion::asc(SortKey::Subject),
        ];
        let rendered = crit
            .iter()
            .map(|c| c.render())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(rendered, "REVERSE DATE SUBJECT");
    }

    #[test]
    fn thread_parses_flat_siblings() {
        // `* THREAD (1)(2)(3)` → three independent single-message threads.
        let roots = parse_thread_response("(1)(2)(3)");
        assert_eq!(roots, vec![leaf(1), leaf(2), leaf(3)]);
    }

    #[test]
    fn thread_parses_rfc5256_nested_example() {
        // The RFC 5256 §4 illustrative reply.
        let roots = parse_thread_response("(2)(3 6 (4 23)(44 7 96))");
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0], leaf(2));

        // 3 → 6, and 6 has two child branches: (4 → 23) and (44 → 7 → 96).
        let three = &roots[1];
        assert_eq!(three.id, 3);
        assert_eq!(three.children.len(), 1);
        let six = &three.children[0];
        assert_eq!(six.id, 6);
        assert_eq!(six.children.len(), 2);

        let branch_a = &six.children[0];
        assert_eq!(branch_a.id, 4);
        assert_eq!(branch_a.children, vec![leaf(23)]);

        let branch_b = &six.children[1];
        assert_eq!(branch_b.id, 44);
        assert_eq!(branch_b.children[0].id, 7);
        assert_eq!(branch_b.children[0].children, vec![leaf(96)]);
    }

    #[test]
    fn thread_parses_linear_chain() {
        // `(1 2 3)` → 1 → 2 → 3.
        let roots = parse_thread_response("(1 2 3)");
        assert_eq!(
            roots,
            vec![ThreadNode {
                id: 1,
                children: vec![ThreadNode {
                    id: 2,
                    children: vec![leaf(3)],
                }],
            }]
        );
    }

    #[test]
    fn thread_empty_reply_is_no_threads() {
        assert!(parse_thread_response("").is_empty());
    }
}

#[cfg(test)]
mod acl_metadata_tests {
    //! ACL (RFC 4314) + METADATA (RFC 5464) response parsing and command
    //! framing, driven against an in-crate scripted IMAP socket (a `tokio`
    //! `TcpListener` replaying server lines — no live server required).

    use super::*;
    use std::net::SocketAddr;
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    // --- pure parser round-trips --------------------------------------------

    #[test]
    fn getacl_reply_parses_into_grants() {
        // A quoted identifier (with a space) and `anyone` special identifier.
        let entries = parse_acl(r#""INBOX" "John Smith" lrswi anyone lr"#);
        assert_eq!(
            entries,
            vec![
                AclEntry {
                    identifier: "John Smith".into(),
                    rights: "lrswi".into(),
                },
                AclEntry {
                    identifier: "anyone".into(),
                    rights: "lr".into(),
                },
            ]
        );
    }

    #[test]
    fn myrights_reply_yields_rights_string() {
        // `* MYRIGHTS <mailbox> <rights>` — rights is the 2nd token.
        let rights = tokenize(r#""INBOX" lrswipkxtea"#)
            .into_iter()
            .nth(1)
            .map(tok_value);
        assert_eq!(rights.as_deref(), Some("lrswipkxtea"));
    }

    #[test]
    fn listrights_reply_yields_rights_tokens() {
        // `* LISTRIGHTS <mailbox> <identifier> <req> <opt> …`.
        let toks: Vec<String> = tokenize(r#""INBOX" smith la r s w i p k x t e"#)
            .into_iter()
            .skip(2)
            .map(tok_value)
            .collect();
        assert_eq!(
            toks,
            vec!["la", "r", "s", "w", "i", "p", "k", "x", "t", "e"]
        );
    }

    #[test]
    fn getmetadata_reply_parses_value_and_nil() {
        let entries =
            parse_metadata(r#""INBOX" (/shared/comment "Shared note" /private/comment NIL)"#);
        assert_eq!(
            entries,
            vec![
                MetadataEntry {
                    entry: "/shared/comment".into(),
                    value: Some("Shared note".into()),
                },
                MetadataEntry {
                    entry: "/private/comment".into(),
                    value: None,
                },
            ]
        );
    }

    #[test]
    fn server_level_metadata_uses_empty_mailbox() {
        // A server-level GETMETADATA reply carries the empty mailbox name.
        let entries = parse_metadata(r#""" (/shared/comment "site-wide")"#);
        assert_eq!(
            entries,
            vec![MetadataEntry {
                entry: "/shared/comment".into(),
                value: Some("site-wide".into()),
            }]
        );
    }

    #[test]
    fn getmetadata_reply_parses_synchronizing_literal_value() {
        // The EXACT framing Dovecot returns (RFC 5464): the value is an IMAP
        // synchronizing literal `{9}` whose 9 octets (`hello t13`) follow the CRLF
        // that terminates the marker. The parser must consume those octets across
        // the line boundary — NOT leak the `{9}` length marker as the value (the
        // t13-E10 live bug).
        let entries =
            parse_metadata("* METADATA \"INBOX\" (/private/comment {9}\r\nhello t13)\r\n");
        assert_eq!(
            entries,
            vec![MetadataEntry {
                entry: "/private/comment".into(),
                value: Some("hello t13".into()),
            }]
        );
    }

    #[test]
    fn getmetadata_reply_parses_multiple_literal_entries() {
        // Two entries, each returned as a literal; octet counts must be honored
        // independently so the second entry name is not swallowed by the first.
        let entries = parse_metadata(
            "* METADATA \"INBOX\" (/private/comment {5}\r\nhello /shared/comment {3}\r\nbye)\r\n",
        );
        assert_eq!(
            entries,
            vec![
                MetadataEntry {
                    entry: "/private/comment".into(),
                    value: Some("hello".into()),
                },
                MetadataEntry {
                    entry: "/shared/comment".into(),
                    value: Some("bye".into()),
                },
            ]
        );
    }

    #[test]
    fn getmetadata_reply_mixes_literal_quoted_and_nil() {
        // A literal value, a quoted value (the E6 regression form), and NIL
        // (removed entry) must all parse in one list.
        let entries = parse_metadata(
            "* METADATA \"INBOX\" (/private/comment {9}\r\nhello t13 /shared/comment \"quoted val\" /private/gone NIL)\r\n",
        );
        assert_eq!(
            entries,
            vec![
                MetadataEntry {
                    entry: "/private/comment".into(),
                    value: Some("hello t13".into()),
                },
                MetadataEntry {
                    entry: "/shared/comment".into(),
                    value: Some("quoted val".into()),
                },
                MetadataEntry {
                    entry: "/private/gone".into(),
                    value: None,
                },
            ]
        );
    }

    #[test]
    fn getmetadata_reply_literal_value_spanning_a_crlf() {
        // The literal octet count INCLUDES CRLFs inside the value: `line1\r\nline2`
        // is 12 octets. This proves octet-exact consumption — a char/line-oriented
        // parser would stop at the embedded CRLF and mis-slice the value (and the
        // trailing `)`), whereas the count `{12}` pulls exactly the right bytes.
        let entries =
            parse_metadata("* METADATA \"INBOX\" (/private/note {12}\r\nline1\r\nline2)\r\n");
        assert_eq!(
            entries,
            vec![MetadataEntry {
                entry: "/private/note".into(),
                value: Some("line1\r\nline2".into()),
            }]
        );
    }

    #[test]
    fn getmetadata_reply_empty_literal_value() {
        // A `{0}` literal is a present-but-empty value (distinct from NIL/None).
        let entries = parse_metadata("* METADATA \"INBOX\" (/private/comment {0}\r\n)\r\n");
        assert_eq!(
            entries,
            vec![MetadataEntry {
                entry: "/private/comment".into(),
                value: Some(String::new()),
            }]
        );
    }

    #[test]
    fn untagged_payload_distinguishes_myrights_from_metadata() {
        assert!(untagged_payload("* METADATA \"INBOX\" ()", "MYRIGHTS").is_none());
        assert!(untagged_payload("* MYRIGHTS \"INBOX\" lr", "METADATA").is_none());
        assert_eq!(
            untagged_payload("* MYRIGHTS \"INBOX\" lr", "MYRIGHTS"),
            Some("\"INBOX\" lr")
        );
    }

    // --- scripted-socket command framing ------------------------------------

    /// Spawn a mock that greets with `greeting`, then for each client command
    /// line replies with the next scripted response (`{tag}` substituted).
    /// Returns the bound address and a shared log of the client lines read.
    async fn spawn(
        greeting: &'static str,
        replies: Vec<String>,
    ) -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        tokio::spawn(async move {
            let (sock, _) = listener.accept().await.unwrap();
            let (rd, mut wr) = sock.into_split();
            let mut rd = BufReader::new(rd);
            wr.write_all(greeting.as_bytes()).await.unwrap();
            for reply in replies {
                let mut line = String::new();
                if rd.read_line(&mut line).await.unwrap() == 0 {
                    break;
                }
                let tag = line.split_whitespace().next().unwrap_or("*").to_string();
                log2.lock().unwrap().push(line.trim_end().to_string());
                wr.write_all(reply.replace("{tag}", &tag).as_bytes())
                    .await
                    .unwrap();
            }
            // Hold the socket open briefly so the client reads the last reply.
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        });
        (addr, log)
    }

    async fn connect(addr: SocketAddr) -> Session {
        Session::connect(&addr.ip().to_string(), addr.port(), TlsMode::Plaintext)
            .await
            .expect("connect")
    }

    const CAPS: &str = "* OK [CAPABILITY IMAP4rev1 ACL METADATA] ready\r\n";

    #[tokio::test]
    async fn get_acl_round_trips() {
        let reply =
            "* ACL \"INBOX\" alice lrswipkxtea bob lr\r\n{tag} OK Getacl completed\r\n".to_string();
        let (addr, log) = spawn(CAPS, vec![reply]).await;
        let mut session = connect(addr).await;

        let entries = session.get_acl("INBOX").await.unwrap();
        assert_eq!(
            entries,
            vec![
                AclEntry {
                    identifier: "alice".into(),
                    rights: "lrswipkxtea".into(),
                },
                AclEntry {
                    identifier: "bob".into(),
                    rights: "lr".into(),
                },
            ]
        );
        assert!(
            log.lock()
                .unwrap()
                .iter()
                .any(|l| l.contains("GETACL \"INBOX\""))
        );
    }

    #[tokio::test]
    async fn my_rights_round_trips() {
        let reply =
            "* MYRIGHTS \"INBOX\" lrswipkxtea\r\n{tag} OK Myrights completed\r\n".to_string();
        let (addr, _log) = spawn(CAPS, vec![reply]).await;
        let mut session = connect(addr).await;
        assert_eq!(session.my_rights("INBOX").await.unwrap(), "lrswipkxtea");
    }

    #[tokio::test]
    async fn list_rights_round_trips() {
        let reply = "* LISTRIGHTS \"INBOX\" smith la r s w i\r\n{tag} OK Listrights completed\r\n"
            .to_string();
        let (addr, _log) = spawn(CAPS, vec![reply]).await;
        let mut session = connect(addr).await;
        let rights = session.list_rights("INBOX", "smith").await.unwrap();
        assert_eq!(rights, vec!["la", "r", "s", "w", "i"]);
    }

    #[tokio::test]
    async fn get_metadata_round_trips() {
        let reply =
            "* METADATA \"INBOX\" (/shared/comment \"Team inbox\")\r\n{tag} OK Getmetadata completed\r\n"
                .to_string();
        let (addr, log) = spawn(CAPS, vec![reply]).await;
        let mut session = connect(addr).await;
        let entries = session
            .get_metadata("INBOX", &["/shared/comment".to_string()])
            .await
            .unwrap();
        assert_eq!(
            entries,
            vec![MetadataEntry {
                entry: "/shared/comment".into(),
                value: Some("Team inbox".into()),
            }]
        );
        assert!(
            log.lock()
                .unwrap()
                .iter()
                .any(|l| l.contains("GETMETADATA \"INBOX\" (/shared/comment)"))
        );
    }

    #[tokio::test]
    async fn get_metadata_literal_value_round_trips() {
        // Full client path against the exact framing Dovecot returns: the value is
        // a synchronizing literal split across a CRLF that `execute_lines` strips.
        // `get_metadata` must reassemble it and return the value, not the `{9}`
        // marker (the t13-E10 live bug).
        let reply =
            "* METADATA \"INBOX\" (/private/comment {9}\r\nhello t13)\r\n{tag} OK Getmetadata completed\r\n"
                .to_string();
        let (addr, log) = spawn(CAPS, vec![reply]).await;
        let mut session = connect(addr).await;
        let entries = session
            .get_metadata("INBOX", &["/private/comment".to_string()])
            .await
            .unwrap();
        assert_eq!(
            entries,
            vec![MetadataEntry {
                entry: "/private/comment".into(),
                value: Some("hello t13".into()),
            }]
        );
        assert!(
            log.lock()
                .unwrap()
                .iter()
                .any(|l| l.contains("GETMETADATA \"INBOX\" (/private/comment)"))
        );
    }

    #[tokio::test]
    async fn get_server_metadata_uses_empty_mailbox() {
        let reply =
            "* METADATA \"\" (/shared/comment \"site\")\r\n{tag} OK Getmetadata completed\r\n"
                .to_string();
        let (addr, log) = spawn(CAPS, vec![reply]).await;
        let mut session = connect(addr).await;
        let entries = session
            .get_metadata("", &["/shared/comment".to_string()])
            .await
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert!(
            log.lock()
                .unwrap()
                .iter()
                .any(|l| l.contains("GETMETADATA \"\" (/shared/comment)"))
        );
    }

    #[tokio::test]
    async fn set_acl_request_shape() {
        let (addr, log) = spawn(CAPS, vec!["{tag} OK Setacl completed\r\n".to_string()]).await;
        let mut session = connect(addr).await;
        session.set_acl("INBOX", "alice", "lrswi").await.unwrap();
        assert!(
            log.lock()
                .unwrap()
                .iter()
                .any(|l| l.contains(r#"SETACL "INBOX" "alice" "lrswi""#))
        );
    }

    #[tokio::test]
    async fn delete_acl_request_shape() {
        let (addr, log) = spawn(CAPS, vec!["{tag} OK Deleteacl completed\r\n".to_string()]).await;
        let mut session = connect(addr).await;
        session.delete_acl("INBOX", "alice").await.unwrap();
        assert!(
            log.lock()
                .unwrap()
                .iter()
                .any(|l| l.contains(r#"DELETEACL "INBOX" "alice""#))
        );
    }

    #[tokio::test]
    async fn set_metadata_value_and_nil_request_shapes() {
        let (addr, log) = spawn(
            CAPS,
            vec![
                "{tag} OK Setmetadata completed\r\n".to_string(),
                "{tag} OK Setmetadata completed\r\n".to_string(),
            ],
        )
        .await;
        let mut session = connect(addr).await;
        session
            .set_metadata("INBOX", "/shared/comment", Some("hi"))
            .await
            .unwrap();
        // value: None → RFC 5464 NIL (remove the entry).
        session
            .set_metadata("INBOX", "/shared/comment", None)
            .await
            .unwrap();
        let log = log.lock().unwrap();
        assert!(
            log.iter()
                .any(|l| l.contains(r#"SETMETADATA "INBOX" (/shared/comment "hi")"#))
        );
        assert!(
            log.iter()
                .any(|l| l.contains(r#"SETMETADATA "INBOX" (/shared/comment NIL)"#))
        );
    }

    #[tokio::test]
    async fn not_advertised_returns_unsupported_without_sending() {
        // Greeting advertises neither ACL nor METADATA.
        let greeting = "* OK [CAPABILITY IMAP4rev1] ready\r\n";
        let (addr, log) = spawn(greeting, vec![]).await;
        let mut session = connect(addr).await;

        assert!(matches!(
            session.get_acl("INBOX").await,
            Err(ImapError::Unsupported(_))
        ));
        assert!(matches!(
            session.my_rights("INBOX").await,
            Err(ImapError::Unsupported(_))
        ));
        assert!(matches!(
            session
                .get_metadata("INBOX", &["/shared/comment".to_string()])
                .await,
            Err(ImapError::Unsupported(_))
        ));
        assert!(matches!(
            session
                .set_metadata("INBOX", "/shared/comment", Some("x"))
                .await,
            Err(ImapError::Unsupported(_))
        ));
        // Guarded commands must never reach the wire.
        assert!(log.lock().unwrap().is_empty());
    }
}
