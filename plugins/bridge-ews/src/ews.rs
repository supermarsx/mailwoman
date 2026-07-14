//! EWS mail operations: folder + item sync, MIME fetch, send, flag update, move,
//! and GAL resolution (`ResolveNames`). Each operation is a pure request **builder**
//! (SOAP XML) plus a **parser** over recorded responses — no I/O here; the wasm
//! guest runs these over the host `http-fetch` import (`crate::guest`).

use base64::Engine as _;
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;

use crate::soap;

fn local(name: QName<'_>) -> String {
    String::from_utf8_lossy(name.local_name().as_ref()).into_owned()
}

fn attr_val(e: &quick_xml::events::BytesStart<'_>, key: &str) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

// ── Folder hierarchy (⇒ engine mailboxes) ──────────────────────────────────────

/// A mail folder as enumerated by `SyncFolderHierarchy`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EwsFolder {
    pub id: String,
    pub change_key: String,
    pub display_name: String,
    /// Lowercased JMAP special-use role (`inbox`/`sent`/…/`none`).
    pub role: String,
    pub total: u32,
    pub unread: u32,
}

/// Map a folder display name to a JMAP special-use role (best-effort; Exchange does
/// not return `DistinguishedFolderId` in the hierarchy sync).
#[must_use]
pub fn role_for(display_name: &str) -> &'static str {
    match display_name.trim().to_ascii_lowercase().as_str() {
        "inbox" => "inbox",
        "sent items" | "sent" => "sent",
        "drafts" => "drafts",
        "deleted items" | "trash" => "trash",
        "junk email" | "junk" => "junk",
        "archive" => "archive",
        "outbox" => "none",
        _ => "none",
    }
}

/// Build a `SyncFolderHierarchy` request; empty `sync_state` ⇒ a full enumeration.
#[must_use]
pub fn sync_folder_hierarchy_request(sync_state: &str) -> String {
    let state = if sync_state.is_empty() {
        String::new()
    } else {
        format!("<m:SyncState>{}</m:SyncState>", soap::escape(sync_state))
    };
    soap::envelope(&format!(
        concat!(
            "<m:SyncFolderHierarchy>",
            "<m:FolderShape><t:BaseShape>Default</t:BaseShape></m:FolderShape>",
            "{state}",
            "</m:SyncFolderHierarchy>"
        ),
        state = state
    ))
}

/// Parse a `SyncFolderHierarchy` response into `(folders, sync_state)`.
pub fn parse_folder_hierarchy(xml: &str) -> Result<(Vec<EwsFolder>, String), String> {
    let sync_state = soap::first_text(xml, "SyncState").unwrap_or_default();
    let mut folders = Vec::new();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut cur: Option<EwsFolder> = None;
    let mut text_field: Option<String> = None;
    let mut pending = String::new();

    let is_folder = |n: &str| {
        matches!(
            n,
            "Folder" | "CalendarFolder" | "ContactsFolder" | "TasksFolder" | "SearchFolder"
        )
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let ln = local(e.name());
                if is_folder(&ln) {
                    cur = Some(EwsFolder::default());
                } else if cur.is_some() {
                    match ln.as_str() {
                        "FolderId" => {
                            if let Some(f) = cur.as_mut() {
                                f.id = attr_val(&e, "Id").unwrap_or_default();
                                f.change_key = attr_val(&e, "ChangeKey").unwrap_or_default();
                            }
                        }
                        "DisplayName" | "TotalCount" | "UnreadCount" => {
                            text_field = Some(ln);
                            pending.clear();
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                let ln = local(e.name());
                if ln == "FolderId"
                    && let Some(f) = cur.as_mut()
                {
                    f.id = attr_val(&e, "Id").unwrap_or_default();
                    f.change_key = attr_val(&e, "ChangeKey").unwrap_or_default();
                }
            }
            Ok(Event::Text(t)) if text_field.is_some() => {
                pending.push_str(&t.decode().unwrap_or_default());
            }
            Ok(Event::End(e)) => {
                let ln = local(e.name());
                if let Some(field) = text_field.take() {
                    if ln == field {
                        if let Some(f) = cur.as_mut() {
                            match field.as_str() {
                                "DisplayName" => f.display_name = pending.trim().to_string(),
                                "TotalCount" => f.total = pending.trim().parse().unwrap_or(0),
                                "UnreadCount" => f.unread = pending.trim().parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                    } else {
                        text_field = Some(field); // not our closer; keep waiting
                    }
                }
                if is_folder(&ln)
                    && let Some(mut f) = cur.take()
                {
                    f.role = role_for(&f.display_name).to_string();
                    folders.push(f);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("folder-hierarchy parse: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok((folders, sync_state))
}

// ── Item sync (⇒ engine MailboxDelta) ──────────────────────────────────────────

/// An EWS item coordinate.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EwsItem {
    pub id: String,
    pub change_key: String,
}

/// The result of a `SyncFolderItems` call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemsDelta {
    pub added: Vec<EwsItem>,
    pub removed: Vec<EwsItem>,
    pub sync_state: String,
    pub includes_last: bool,
}

/// Build a `SyncFolderItems` request for `folder_id`; empty `sync_state` ⇒ initial.
#[must_use]
pub fn sync_folder_items_request(folder_id: &str, sync_state: &str, max: u32) -> String {
    let state = if sync_state.is_empty() {
        String::new()
    } else {
        format!("<m:SyncState>{}</m:SyncState>", soap::escape(sync_state))
    };
    soap::envelope(&format!(
        concat!(
            "<m:SyncFolderItems>",
            "<m:ItemShape><t:BaseShape>IdOnly</t:BaseShape></m:ItemShape>",
            "<m:SyncFolderId><t:FolderId Id=\"{fid}\"/></m:SyncFolderId>",
            "{state}",
            "<m:MaxChangesReturned>{max}</m:MaxChangesReturned>",
            "</m:SyncFolderItems>"
        ),
        fid = soap::escape(folder_id),
        state = state,
        max = max
    ))
}

/// Parse a `SyncFolderItems` response, distinguishing `Create` (added) from
/// `Delete` (removed) changes and capturing the new `SyncState`.
pub fn parse_folder_items(xml: &str) -> Result<ItemsDelta, String> {
    let mut out = ItemsDelta {
        sync_state: soap::first_text(xml, "SyncState").unwrap_or_default(),
        includes_last: soap::first_text(xml, "IncludesLastItemInRange").as_deref() == Some("true"),
        ..Default::default()
    };
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    // 0 = none, 1 = inside <Create>, 2 = inside <Delete>/<ReadFlagChange>
    let mut mode = 0u8;
    let push_item = |e: &quick_xml::events::BytesStart<'_>, mode: u8, out: &mut ItemsDelta| {
        let item = EwsItem {
            id: attr_val(e, "Id").unwrap_or_default(),
            change_key: attr_val(e, "ChangeKey").unwrap_or_default(),
        };
        match mode {
            1 => out.added.push(item),
            2 => out.removed.push(item),
            _ => {}
        }
    };
    loop {
        match reader.read_event_into(&mut buf) {
            // `ItemId` is normally an empty element, but tolerate a start tag too.
            Ok(Event::Start(e)) => match local(e.name()).as_str() {
                "Create" => mode = 1,
                "Delete" => mode = 2,
                "ItemId" => push_item(&e, mode, &mut out),
                _ => {}
            },
            Ok(Event::Empty(e)) if local(e.name()) == "ItemId" => {
                push_item(&e, mode, &mut out);
            }
            Ok(Event::End(e)) => match local(e.name()).as_str() {
                "Create" | "Delete" => mode = 0,
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("folder-items parse: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

// ── Fetch raw MIME ──────────────────────────────────────────────────────────────

/// Build a `GetItem` request that returns full RFC822 `MimeContent` for `item_id`.
#[must_use]
pub fn get_item_mime_request(item_id: &str) -> String {
    soap::envelope(&format!(
        concat!(
            "<m:GetItem>",
            "<m:ItemShape>",
            "<t:BaseShape>IdOnly</t:BaseShape>",
            "<t:IncludeMimeContent>true</t:IncludeMimeContent>",
            "</m:ItemShape>",
            "<m:ItemIds><t:ItemId Id=\"{id}\"/></m:ItemIds>",
            "</m:GetItem>"
        ),
        id = soap::escape(item_id)
    ))
}

/// Parse the base64 `MimeContent` from a `GetItem` response into raw RFC822 bytes.
pub fn parse_item_mime(xml: &str) -> Result<Vec<u8>, String> {
    if !soap::is_success(xml) {
        return Err(soap::message_text(xml).unwrap_or_else(|| "GetItem failed".into()));
    }
    let b64 = soap::first_text(xml, "MimeContent").ok_or("GetItem response has no MimeContent")?;
    base64::engine::general_purpose::STANDARD
        .decode(b64.trim().replace(['\r', '\n', ' '], ""))
        .map_err(|e| format!("MimeContent base64: {e}"))
}

// ── Send (CreateItem, SendAndSaveCopy) ──────────────────────────────────────────

/// Build a `CreateItem` request that submits a full RFC822 message as `MimeContent`
/// (`MessageDisposition="SendAndSaveCopy"` — send + keep a Sent Items copy).
#[must_use]
pub fn create_item_send_request(raw_mime: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(raw_mime);
    soap::envelope(&format!(
        concat!(
            "<m:CreateItem MessageDisposition=\"SendAndSaveCopy\">",
            "<m:SavedItemFolderId><t:DistinguishedFolderId Id=\"sentitems\"/></m:SavedItemFolderId>",
            "<m:Items><t:Message><t:MimeContent CharacterSet=\"UTF-8\">{b64}</t:MimeContent>",
            "</t:Message></m:Items>",
            "</m:CreateItem>"
        ),
        b64 = b64
    ))
}

/// The item id assigned to a just-sent/created message (if the server echoes one).
pub fn parse_created_item_id(xml: &str) -> Result<Option<EwsItem>, String> {
    if !soap::is_success(xml) {
        return Err(soap::message_text(xml).unwrap_or_else(|| "CreateItem failed".into()));
    }
    let id = soap::first_attr(xml, "ItemId", "Id");
    Ok(id.map(|id| EwsItem {
        change_key: soap::first_attr(xml, "ItemId", "ChangeKey").unwrap_or_default(),
        id,
    }))
}

// ── Flag update (Seen) + move ──────────────────────────────────────────────────

/// Build an `UpdateItem` request setting the read (`IsRead`) state of an item.
#[must_use]
pub fn update_read_flag_request(item_id: &str, change_key: &str, is_read: bool) -> String {
    soap::envelope(&format!(
        concat!(
            "<m:UpdateItem MessageDisposition=\"SaveOnly\" ConflictResolution=\"AutoResolve\">",
            "<m:ItemChanges><t:ItemChange>",
            "<t:ItemId Id=\"{id}\" ChangeKey=\"{ck}\"/>",
            "<t:Updates><t:SetItemField>",
            "<t:FieldURI FieldURI=\"message:IsRead\"/>",
            "<t:Message><t:IsRead>{read}</t:IsRead></t:Message>",
            "</t:SetItemField></t:Updates>",
            "</t:ItemChange></m:ItemChanges>",
            "</m:UpdateItem>"
        ),
        id = soap::escape(item_id),
        ck = soap::escape(change_key),
        read = is_read
    ))
}

/// Build a `MoveItem` request moving an item to `to_folder_id`.
#[must_use]
pub fn move_item_request(item_id: &str, to_folder_id: &str) -> String {
    soap::envelope(&format!(
        concat!(
            "<m:MoveItem>",
            "<m:ToFolderId><t:FolderId Id=\"{fid}\"/></m:ToFolderId>",
            "<m:ItemIds><t:ItemId Id=\"{id}\"/></m:ItemIds>",
            "</m:MoveItem>"
        ),
        fid = soap::escape(to_folder_id),
        id = soap::escape(item_id)
    ))
}

// ── GAL (ResolveNames) ──────────────────────────────────────────────────────────

/// A resolved directory entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalEntry {
    pub display_name: String,
    pub email: String,
}

impl GalEntry {
    /// `"Display Name <email>"` (or bare email when no display name).
    #[must_use]
    pub fn formatted(&self) -> String {
        if self.display_name.is_empty() {
            self.email.clone()
        } else {
            format!("{} <{}>", self.display_name, self.email)
        }
    }
}

/// Build a `ResolveNames` request against the Active Directory GAL.
#[must_use]
pub fn resolve_names_request(query: &str) -> String {
    soap::envelope(&format!(
        concat!(
            "<m:ResolveNames ReturnFullContactData=\"false\" SearchScope=\"ActiveDirectory\">",
            "<m:UnresolvedEntry>{q}</m:UnresolvedEntry>",
            "</m:ResolveNames>"
        ),
        q = soap::escape(query)
    ))
}

/// Parse a `ResolveNames` response into GAL entries (pairs Name with the mailbox
/// `EmailAddress` inside each `<t:Mailbox>`).
pub fn parse_resolve_names(xml: &str) -> Vec<GalEntry> {
    let mut entries = Vec::new();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_mailbox = false;
    let mut name = String::new();
    let mut email = String::new();
    let mut field: Option<String> = None;
    let mut pending = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let ln = local(e.name());
                if ln == "Mailbox" {
                    in_mailbox = true;
                    name.clear();
                    email.clear();
                } else if in_mailbox && (ln == "Name" || ln == "EmailAddress") {
                    field = Some(ln);
                    pending.clear();
                }
            }
            Ok(Event::Text(t)) if field.is_some() => {
                pending.push_str(&t.decode().unwrap_or_default());
            }
            Ok(Event::End(e)) => {
                let ln = local(e.name());
                if let Some(f) = field.take() {
                    if ln == f {
                        match f.as_str() {
                            "Name" => name = pending.trim().to_string(),
                            "EmailAddress" => email = pending.trim().to_string(),
                            _ => {}
                        }
                    } else {
                        field = Some(f);
                    }
                }
                if ln == "Mailbox" {
                    in_mailbox = false;
                    if !email.is_empty() {
                        entries.push(GalEntry {
                            display_name: std::mem::take(&mut name),
                            email: std::mem::take(&mut email),
                        });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIER: &str = include_str!("../fixtures/sync_folder_hierarchy.xml");
    const ITEMS: &str = include_str!("../fixtures/sync_folder_items.xml");
    const GETITEM: &str = include_str!("../fixtures/get_item_mime.xml");
    const CREATED: &str = include_str!("../fixtures/create_item_sent.xml");
    const RESOLVE: &str = include_str!("../fixtures/resolve_names.xml");

    #[test]
    fn folder_hierarchy_parses_roles_and_counts() {
        let (folders, state) = parse_folder_hierarchy(HIER).unwrap();
        assert_eq!(state, "H-STATE-1");
        let inbox = folders.iter().find(|f| f.role == "inbox").unwrap();
        assert_eq!(inbox.display_name, "Inbox");
        assert_eq!(inbox.total, 3);
        assert_eq!(inbox.unread, 1);
        assert!(!inbox.id.is_empty());
        assert!(folders.iter().any(|f| f.role == "sent"));
    }

    #[test]
    fn folder_items_splits_create_and_delete() {
        let d = parse_folder_items(ITEMS).unwrap();
        assert_eq!(d.sync_state, "I-STATE-2");
        assert!(d.includes_last);
        assert_eq!(d.added.len(), 2);
        assert_eq!(d.removed.len(), 1);
        assert_eq!(d.added[0].id, "ITEM-AAA");
        assert_eq!(d.removed[0].id, "ITEM-OLD");
    }

    #[test]
    fn get_item_decodes_mime() {
        let raw = parse_item_mime(GETITEM).unwrap();
        let s = String::from_utf8_lossy(&raw);
        assert!(s.contains("Subject: Quarterly numbers"));
        assert!(s.contains("From: alice@corp.example"));
    }

    #[test]
    fn create_item_reports_sent_id() {
        let item = parse_created_item_id(CREATED).unwrap().unwrap();
        assert_eq!(item.id, "SENT-123");
    }

    #[test]
    fn resolve_names_builds_gal_entries() {
        let g = parse_resolve_names(RESOLVE);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].formatted(), "John Doe <john.doe@corp.example>");
        assert_eq!(g[1].email, "jane.roe@corp.example");
    }

    #[test]
    fn requests_carry_expected_operations() {
        assert!(sync_folder_hierarchy_request("").contains("SyncFolderHierarchy"));
        assert!(sync_folder_items_request("F", "S", 50).contains("SyncFolderItems"));
        assert!(get_item_mime_request("X").contains("IncludeMimeContent"));
        assert!(create_item_send_request(b"raw").contains("SendAndSaveCopy"));
        assert!(update_read_flag_request("i", "c", true).contains("message:IsRead"));
        assert!(move_item_request("i", "f").contains("MoveItem"));
        assert!(resolve_names_request("bob").contains("ResolveNames"));
    }
}
