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
    BackendCaps, Flag, MailboxRole, MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage,
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
                if !self.caps.has_auth("XOAUTH2") {
                    return Err(ImapError::Unsupported("AUTH=XOAUTH2 not advertised".into()));
                }
                let frame = crate::sasl::xoauth2(username, token);
                self.conn.authenticate("XOAUTH2", &[frame]).await?
            }
            Credentials::Password { username, password } => {
                if self.caps.has_auth("PLAIN") {
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
