//! 26.20 t29-e9: the read-receipt request and inline images of a composed
//! message, what `Email/get` says about a receipt request or a received
//! receipt, and `MDN/send`.
//!
//! Every case drives the real dispatch (`Engine::handle_jmap`) and asserts on
//! what left the engine: the bytes APPENDed to the upstream Drafts folder for a
//! create, and the envelope and bytes handed to the account submitter for a
//! receipt. The submitter here records what it is handed; it can be told to
//! fail, and it notes whether `$mdnsent` was already on the message when it was
//! called.
//!
//! Run:
//!   cargo test -p mw-engine --test t29_mdn_inline --locked -- --test-threads=1

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MailboxRole,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor,
    WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_mime::{EmailAddress, MdnActionMode, MdnDisposition, MdnInput, MdnSendingMode};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, FsUploadBackend, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;
const ME: &str = "me@example.org";
const ANN: &str = "ann@example.org";

/// A 1×1 PNG.
const PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae,
    0x42, 0x60, 0x82,
];

// ── received mail the backend serves ─────────────────────────────────────────

/// One received message. `return_path` is written as given between `<` and
/// `>`; `None` leaves the header out. `extra` is raw header lines, each ending
/// in CRLF.
fn received(subject: &str, return_path: Option<&str>, to: &str, extra: &str) -> Vec<u8> {
    let mut out = String::new();
    if let Some(path) = return_path {
        out.push_str(&format!("Return-Path: <{path}>\r\n"));
    }
    out.push_str(&format!(
        "From: Ann <{ANN}>\r\nTo: {to}\r\nSubject: {subject}\r\n\
         Message-ID: <{}@example.org>\r\nDate: Mon, 05 Oct 2026 10:00:00 +0000\r\n{extra}\r\n\
         the body\r\n",
        subject.replace(' ', "-")
    ));
    out.into_bytes()
}

fn request_header(value: &str) -> String {
    format!("Disposition-Notification-To: {value}\r\n")
}

/// A disposition notification (built by `mw-mime`) that itself carries a
/// receipt request and an ordinary return path.
fn report_asking_for_a_receipt() -> Vec<u8> {
    let report = mw_mime::build_mdn(&MdnInput {
        from: EmailAddress {
            name: None,
            email: ANN.into(),
        },
        to: ME.into(),
        final_recipient: ANN.into(),
        original_recipient: None,
        original_message_id: Some("sent-earlier@mailwoman.local".into()),
        original_subject: Some("lunch".into()),
        action_mode: MdnActionMode::Manual,
        sending_mode: MdnSendingMode::Manual,
        disposition: MdnDisposition::Displayed,
        message_id: Some("report-1@example.org".into()),
    })
    .unwrap();
    let mut out = format!("Return-Path: <{ANN}>\r\n{}", request_header(ANN)).into_bytes();
    out.extend(report);
    out
}

/// The mail on the server, by folder. Subjects are unique: tests find a
/// message by its subject.
fn scripted_mail() -> HashMap<String, Vec<Vec<u8>>> {
    let ask = request_header(ANN);
    let inbox = vec![
        received("request", Some(ANN), ME, &ask),
        received("plain", Some(ANN), ME, ""),
        received(
            "other address",
            Some(ANN),
            ME,
            &request_header("<boss@example.net>"),
        ),
        received(
            "list mail",
            Some(ANN),
            ME,
            &format!("{ask}List-Id: <announce.example.org>\r\n"),
        ),
        received("no return path", None, ME, &ask),
        received("null return path", Some(""), ME, &ask),
        received("to an alias", Some(ANN), "alias@example.org", &ask),
        // Hostile request headers.
        received(
            "folded bcc",
            Some(ANN),
            ME,
            &format!("Disposition-Notification-To: {ANN}\r\n\tBcc: victim@example.org\r\n"),
        ),
        received(
            "two addresses",
            Some(ANN),
            ME,
            &request_header(&format!("{ANN}, victim@example.org")),
        ),
        received(
            "two headers",
            Some(ANN),
            ME,
            &format!("{ask}{}", request_header("victim@example.org")),
        ),
        received(
            "address in the name",
            Some(ANN),
            ME,
            &request_header(&format!("\"victim@example.org\" <{ANN}>")),
        ),
        // A subject whose encoded word decodes to a line break and a header.
        received(
            "=?utf-8?q?hostile_subject=0D=0ABcc:_victim@example.org?=",
            Some(ANN),
            ME,
            &ask,
        ),
        report_asking_for_a_receipt(),
    ];
    HashMap::from([
        ("INBOX".to_string(), inbox),
        (
            "Junk".to_string(),
            vec![received("junk request", Some(ANN), ME, &ask)],
        ),
    ])
}

/// An IMAP-shaped backend serving [`scripted_mail`]. APPENDs are accepted and
/// their bytes kept by folder.
struct FakeBackend {
    mail: HashMap<String, Vec<Vec<u8>>>,
    appended: Mutex<HashMap<String, Vec<Vec<u8>>>>,
}

impl FakeBackend {
    fn new() -> Self {
        Self {
            mail: scripted_mail(),
            appended: Mutex::default(),
        }
    }

    /// The bytes of the last message APPENDed to Drafts.
    fn last_draft(&self) -> Vec<u8> {
        self.appended.lock().unwrap()["Drafts"]
            .last()
            .cloned()
            .expect("a draft was appended")
    }
}

#[async_trait]
impl AccountBackend for FakeBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok([
            ("INBOX", MailboxRole::Inbox),
            ("Sent", MailboxRole::Sent),
            ("Drafts", MailboxRole::Drafts),
            ("Junk", MailboxRole::Junk),
        ]
        .into_iter()
        .map(|(name, role)| {
            let total = self.mail.get(name).map_or(0, Vec::len) as u32;
            RawMailbox {
                mailbox_ref: RawMailboxRef {
                    name: name.to_string(),
                    uidvalidity: UIDVALIDITY,
                },
                role,
                parent: None,
                uidnext: total + 1,
                highestmodseq: 0,
                total,
                unread: 0,
            }
        })
        .collect())
    }

    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        let seen = match cursor {
            SyncCursor::UidWindow { uidnext, .. } => *uidnext,
            _ => 1,
        };
        let total = self.mail.get(&mbox.name).map_or(0, Vec::len) as u32;
        let added = (seen.max(1)..=total)
            .map(|uid| MessageRef::Imap {
                mailbox: mbox.clone(),
                uidvalidity: UIDVALIDITY,
                uid,
            })
            .collect();
        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: total + 1,
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Ok(refs
            .iter()
            .filter_map(|r| {
                let MessageRef::Imap { mailbox, uid, .. } = r else {
                    return None;
                };
                let raw = self.mail.get(&mailbox.name)?.get(*uid as usize - 1)?;
                Some(RawMessage {
                    message_ref: r.clone(),
                    raw: raw.clone(),
                    flags: vec![Flag::Seen],
                    internaldate: Some("2026-10-05T10:00:00Z".into()),
                })
            })
            .collect())
    }

    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("move".into()))
    }

    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        let mut appended = self.appended.lock().unwrap();
        let folder = appended.entry(mbox.name.clone()).or_default();
        folder.push(raw.to_vec());
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid: 100 + folder.len() as u32,
        })
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

/// Records every message it is handed and accepts all of its recipients, or
/// refuses the connection while `down` is set.
struct RecordingSubmitter {
    handed: Mutex<Vec<Outgoing>>,
    down: AtomicBool,
    store: Store,
    /// The message whose flags are read when the submitter is called.
    watched: Mutex<Option<String>>,
    /// Whether the watched message carried `$mdnsent`, per call.
    marked_at_submit: Mutex<Vec<bool>>,
}

impl RecordingSubmitter {
    fn handed(&self) -> Vec<Outgoing> {
        self.handed.lock().unwrap().clone()
    }

    fn only(&self) -> Outgoing {
        let handed = self.handed();
        assert_eq!(handed.len(), 1, "exactly one message handed to SMTP");
        handed.into_iter().next().unwrap()
    }
}

#[async_trait]
impl MailSubmitter for RecordingSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        let watched = self.watched.lock().unwrap().clone();
        if let Some(id) = watched {
            let flags = self.store.get_message(&id).await.unwrap().flags_json;
            self.marked_at_submit
                .lock()
                .unwrap()
                .push(flags.contains("$mdnsent"));
        }
        if self.down.load(Ordering::SeqCst) {
            return Err(EngineError::Transport("connection refused".into()));
        }
        let accepted = msg.rcpt_to.clone();
        self.handed.lock().unwrap().push(msg);
        Ok(SubmissionResult {
            accepted,
            rejected: Vec::new(),
        })
    }
}

struct Harness {
    engine: Arc<Engine>,
    store: Store,
    account_id: String,
    backend: Arc<FakeBackend>,
    submitter: Arc<RecordingSubmitter>,
}

async fn new_account(store: &Store, username: &str) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: username.into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap()
}

/// An engine with one connected account whose identity is `identity`.
async fn setup_as(identity: &str) -> Harness {
    // Uploads (inline images are uploaded blobs) go to a directory of this
    // test run; it is left for the operating system's temp cleanup.
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let uploads = std::env::temp_dir().join(format!("mw-t29-e9-uploads-{unique}"));
    let store = Store::open_in_memory(ServerKey::generate())
        .await
        .unwrap()
        .with_upload_backend(Arc::new(FsUploadBackend::new(uploads)));
    let account_id = new_account(&store, identity).await;
    let engine = Arc::new(Engine::new(store.clone()));
    let backend = Arc::new(FakeBackend::new());
    let submitter = Arc::new(RecordingSubmitter {
        handed: Mutex::default(),
        down: AtomicBool::new(false),
        store: store.clone(),
        watched: Mutex::default(),
        marked_at_submit: Mutex::default(),
    });
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            backend.clone() as Arc<dyn AccountBackend>,
            submitter.clone() as Arc<dyn MailSubmitter>,
            identity,
        ),
    );
    engine.resync(&account_id).await.unwrap();
    Harness {
        engine,
        store,
        account_id,
        backend,
        submitter,
    }
}

async fn setup() -> Harness {
    setup_as(ME).await
}

async fn jmap(h: &Harness, calls: Value) -> Value {
    h.engine
        .handle_jmap(&h.account_id, &json!({ "methodCalls": calls }))
        .await
}

fn result<'a>(resp: &'a Value, call_id: &str) -> &'a Value {
    resp["methodResponses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[2] == call_id)
        .map(|r| &r[1])
        .unwrap_or(&Value::Null)
}

/// `Email/get` for one id, naming the receipt and inline-part properties.
async fn email(h: &Harness, id: &str) -> Value {
    let resp = jmap(
        h,
        json!([["Email/get", {
            "ids": [id],
            "properties": ["subject", "keywords", "mailwomanMdn", "mailwomanMdnReport", "mailwomanInlineParts"]
        }, "g"]]),
    )
    .await;
    result(&resp, "g")["list"][0].clone()
}

/// The id of the message in the folder with `role` whose subject contains
/// `subject`.
async fn find_in(h: &Harness, role: &str, subject: &str) -> String {
    let resp = jmap(h, json!([["Mailbox/get", {}, "m"]])).await;
    let mailbox = result(&resp, "m")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == role)
        .unwrap_or_else(|| panic!("a {role} mailbox: {resp}"))["id"]
        .as_str()
        .unwrap()
        .to_string();
    let resp = jmap(
        h,
        json!([
            ["Email/query", { "filter": { "inMailbox": mailbox }, "limit": 100 }, "q"],
            ["Email/get", { "#ids": { "resultOf": "q", "name": "Email/query", "path": "/ids" } }, "g"]
        ]),
    )
    .await;
    result(&resp, "g")["list"]
        .as_array()
        .unwrap_or_else(|| panic!("a list: {resp}"))
        .iter()
        .find(|e| e["subject"].as_str().is_some_and(|s| s.contains(subject)))
        .unwrap_or_else(|| panic!("a message with subject {subject:?} in {role}: {resp}"))["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn find(h: &Harness, subject: &str) -> String {
    find_in(h, "inbox", subject).await
}

/// `MDN/send`; returns the method's result.
async fn mdn_send(h: &Harness, email_id: &str, automatic: Option<bool>) -> Value {
    let mut args = json!({ "emailId": email_id });
    if let Some(automatic) = automatic {
        args["automatic"] = json!(automatic);
    }
    let resp = jmap(h, json!([["MDN/send", args, "s"]])).await;
    result(&resp, "s").clone()
}

async fn set_policy(h: &Harness, policy: &str) {
    h.store
        .set_setting(&Engine::mdn_policy_key(&h.account_id), policy)
        .await
        .unwrap();
}

/// `Email/set` create of `extra` merged over a plain message; the method result.
async fn create(h: &Harness, extra: Value) -> Value {
    let mut spec = json!({
        "from": [{ "name": "Me Myself", "email": ME }],
        "to": [{ "email": "visible@example.org" }],
        "subject": "Quarterly numbers",
        "bodyValues": { "1": { "value": "the body" } },
        "textBody": [{ "partId": "1", "type": "text/plain" }]
    });
    for (k, v) in extra.as_object().unwrap() {
        spec[k] = v.clone();
    }
    let resp = jmap(
        h,
        json!([["Email/set", { "create": { "draft": spec } }, "c"]]),
    )
    .await;
    result(&resp, "c").clone()
}

fn created_id(set: &Value) -> String {
    set["created"]["draft"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("draft created: {set}"))
        .to_string()
}

/// Asserts the create was refused as `invalidProperties` naming `property`.
fn assert_invalid(set: &Value, property: &str) {
    assert!(set["created"]["draft"].is_null(), "nothing created: {set}");
    let err = &set["notCreated"]["draft"];
    assert_eq!(err["type"], "invalidProperties", "{set}");
    assert_eq!(err["properties"], json!([property]), "{set}");
}

/// An HTML message whose body refers to each of `cids`.
fn html_with(cids: &[&str]) -> Value {
    let images: String = cids
        .iter()
        .map(|c| format!("<img src=\"cid:{c}\">"))
        .collect();
    json!({
        "bodyValues": { "h": { "value": format!("<p>chart</p>{images}") } },
        "htmlBody": [{ "partId": "h", "type": "text/html" }],
        "textBody": []
    })
}

fn with(mut base: Value, extra: Value) -> Value {
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

/// The header section of a message, unfolded into one string per field.
fn header_fields(raw: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(raw);
    let head = text
        .split_once("\r\n\r\n")
        .map_or(text.as_ref(), |(head, _)| head);
    let mut fields: Vec<String> = Vec::new();
    for line in head.split("\r\n") {
        if line.starts_with([' ', '\t']) && !fields.is_empty() {
            fields.last_mut().unwrap().push_str(line);
        } else {
            fields.push(line.to_string());
        }
    }
    fields
}

/// The values of the header fields named `name` (any case).
fn field_values(raw: &[u8], name: &str) -> Vec<String> {
    header_fields(raw)
        .iter()
        .filter_map(|f| f.split_once(':'))
        .filter(|(n, _)| n.trim().eq_ignore_ascii_case(name))
        .map(|(_, v)| v.trim().to_string())
        .collect()
}

/// The names of a message's header fields, lower-cased and sorted.
fn field_names(raw: &[u8]) -> Vec<String> {
    let mut names: Vec<String> = header_fields(raw)
        .iter()
        .filter_map(|f| {
            f.split_once(':')
                .map(|(name, _)| name.trim().to_ascii_lowercase())
        })
        .collect();
    names.sort();
    names
}

/// The header fields of a receipt this engine sends manually. Anything else in
/// a receipt's header section was put there by something other than the
/// builder.
const RECEIPT_FIELDS: [&str; 7] = [
    "content-type",
    "date",
    "from",
    "message-id",
    "mime-version",
    "subject",
    "to",
];

/// Asserts `sent` is a receipt to `ANN` alone: null reverse path, one
/// recipient, the fixed header fields, and nothing naming the victim.
fn assert_receipt_to_ann(sent: &Outgoing) {
    assert_eq!(sent.mail_from, "", "null reverse path");
    assert_eq!(sent.rcpt_to, vec![ANN.to_string()]);
    assert_eq!(field_names(&sent.raw), RECEIPT_FIELDS, "header fields");
    assert_eq!(field_values(&sent.raw, "To"), vec![format!("<{ANN}>")]);
    assert_eq!(field_values(&sent.raw, "From"), vec![format!("<{ME}>")]);
    let text = String::from_utf8_lossy(&sent.raw);
    for line in text.split("\r\n") {
        assert!(
            !line.to_ascii_lowercase().starts_with("bcc"),
            "no line of the receipt begins a Bcc field: {line:?}"
        );
    }
    let report = mw_mime::parse_mdn(&sent.raw).expect("the receipt parses as a report");
    assert_eq!(report.final_recipient.as_deref(), Some(ME));
    assert_eq!(report.disposition, MdnDisposition::Displayed);
}

// ── the receipt request of a composed message ────────────────────────────────

/// Precondition and effect: without the property the built message asks for
/// nothing; with it, it asks once, for the sender's own address.
#[tokio::test]
async fn the_receipt_request_names_the_sender_and_only_when_asked_for() {
    let h = setup().await;

    created_id(&create(&h, json!({})).await);
    let plain = h.backend.last_draft();
    assert_eq!(
        field_values(&plain, "Disposition-Notification-To"),
        Vec::<String>::new(),
        "no request without the property"
    );

    created_id(&create(&h, json!({ "mailwomanRequestReadReceipt": false })).await);
    assert!(field_values(&h.backend.last_draft(), "Disposition-Notification-To").is_empty());

    let id = created_id(&create(&h, json!({ "mailwomanRequestReadReceipt": true })).await);
    let asking = h.backend.last_draft();
    assert_eq!(
        field_values(&asking, "Disposition-Notification-To"),
        vec![format!("<{ME}>")],
        "exactly one request, for the From mailbox without its display name"
    );
    // The stored draft reads back as carrying the request.
    let got = email(&h, &id).await;
    assert_eq!(got["mailwomanMdn"]["requestedBy"], ME, "{got}");
}

/// The address is never the client's: nothing in the create can name another
/// one, and a sender that is not a mailbox cannot ask at all.
#[tokio::test]
async fn a_client_cannot_choose_the_receipt_address() {
    let h = setup().await;

    for hostile in [
        json!("victim@example.org"),
        json!({ "email": "victim@example.org" }),
        json!(["victim@example.org"]),
        json!(1),
    ] {
        let set = create(&h, json!({ "mailwomanRequestReadReceipt": hostile })).await;
        assert_invalid(&set, "mailwomanRequestReadReceipt");
    }
    assert!(
        h.backend.appended.lock().unwrap().is_empty(),
        "a refused create stores nothing"
    );

    // Properties a client might hope are written as headers are not.
    let set = create(
        &h,
        json!({
            "mailwomanRequestReadReceipt": true,
            "headers": [{ "name": "Disposition-Notification-To", "value": "victim@example.org" }],
            "header:Disposition-Notification-To": "victim@example.org",
            "mailwomanReceiptTo": "victim@example.org",
            "replyTo": [{ "email": "elsewhere@example.org" }]
        }),
    )
    .await;
    created_id(&set);
    let raw = h.backend.last_draft();
    assert_eq!(
        field_values(&raw, "Disposition-Notification-To"),
        vec![format!("<{ME}>")]
    );
    assert!(!String::from_utf8_lossy(&raw).contains("victim@example.org"));

    // A sender with no domain is a valid reverse path but cannot be asked.
    let set = create(
        &h,
        json!({ "from": [{ "email": "me" }], "mailwomanRequestReadReceipt": true }),
    )
    .await;
    assert_invalid(&set, "mailwomanRequestReadReceipt");
    // …while the same sender without the request still composes.
    created_id(&create(&h, json!({ "from": [{ "email": "me" }] })).await);
    assert!(field_values(&h.backend.last_draft(), "Disposition-Notification-To").is_empty());
}

// ── inline parts of a composed message ───────────────────────────────────────

/// One inline image: the message is `multipart/related`, the part carries the
/// Content-ID, and `Email/get` on the stored draft lists it with a blob id that
/// serves the uploaded bytes.
#[tokio::test]
async fn an_inline_image_becomes_a_related_part_the_draft_reports() {
    let h = setup().await;
    let blob = h
        .engine
        .store_upload(&h.account_id, "image/png", PNG)
        .await
        .unwrap();

    // Precondition: the same HTML message without the image is not related.
    created_id(&create(&h, html_with(&[])).await);
    assert!(
        !String::from_utf8_lossy(&h.backend.last_draft()).contains("multipart/related"),
        "no inline part, no multipart/related"
    );

    let set = create(
        &h,
        with(
            html_with(&["chart-1@inline.invalid"]),
            json!({ "attachments": [{
                "blobId": blob, "cid": "chart-1@inline.invalid", "disposition": "inline",
                "type": "image/png", "name": "chart.png"
            }] }),
        ),
    )
    .await;
    let id = created_id(&set);
    let raw = h.backend.last_draft();
    let text = String::from_utf8_lossy(&raw);
    assert!(text.contains("multipart/related"), "{text}");
    assert!(
        text.contains("Content-ID: <chart-1@inline.invalid>"),
        "{text}"
    );
    assert!(
        !text.contains("multipart/mixed"),
        "not an attachment: {text}"
    );

    let got = email(&h, &id).await;
    let parts = got["mailwomanInlineParts"].as_array().unwrap();
    assert_eq!(parts.len(), 1, "{got}");
    assert_eq!(parts[0]["cid"], "chart-1@inline.invalid");
    assert_eq!(parts[0]["type"], "image/png");
    assert_eq!(parts[0]["name"], "chart.png");
    let served = h
        .engine
        .fetch_blob(&h.account_id, parts[0]["blobId"].as_str().unwrap())
        .await
        .unwrap()
        .expect("the listed blob id resolves");
    assert_eq!(served.bytes, PNG, "the part is the uploaded file");
    assert_eq!(served.content_type, "image/png");
}

/// The properties are read from the raw message only when the request names
/// one of them.
#[tokio::test]
async fn the_raw_derived_properties_are_returned_only_when_named() {
    let h = setup().await;
    let id = find(&h, "request").await;
    let resp = jmap(&h, json!([["Email/get", { "ids": [id] }, "g"]])).await;
    let unnamed = &result(&resp, "g")["list"][0];
    assert_eq!(unnamed["subject"], "request");
    for key in ["mailwomanMdn", "mailwomanMdnReport", "mailwomanInlineParts"] {
        assert!(unnamed.get(key).is_none(), "{key} absent when not named");
    }
    let named = email(&h, &id).await;
    assert!(named["mailwomanMdn"].is_object(), "{named}");
    assert!(named["mailwomanMdnReport"].is_null(), "{named}");
    assert_eq!(named["mailwomanInlineParts"], json!([]));
}

/// The body and the inline parts must name the same content ids, the blob must
/// be this account's, and the part must be an image.
#[tokio::test]
async fn an_inline_part_that_does_not_fit_the_message_refuses_the_create() {
    let h = setup().await;
    let blob = h
        .engine
        .store_upload(&h.account_id, "image/png", PNG)
        .await
        .unwrap();
    let inline =
        |cid: &str, blob: &str| json!({ "blobId": blob, "cid": cid, "disposition": "inline" });

    // Control: this exact shape is accepted.
    created_id(
        &create(
            &h,
            with(
                html_with(&["a"]),
                json!({ "attachments": [inline("a", &blob)] }),
            ),
        )
        .await,
    );
    let stored = h.backend.appended.lock().unwrap()["Drafts"].len();

    // A reference with no part.
    assert_invalid(&create(&h, html_with(&["missing"])).await, "htmlBody");
    assert_invalid(
        &create(
            &h,
            with(
                html_with(&["a", "b"]),
                json!({ "attachments": [inline("a", &blob)] }),
            ),
        )
        .await,
        "htmlBody",
    );
    // A part with no reference, and a part with no HTML body at all.
    assert_invalid(
        &create(
            &h,
            with(
                html_with(&[]),
                json!({ "attachments": [inline("a", &blob)] }),
            ),
        )
        .await,
        "attachments",
    );
    assert_invalid(
        &create(&h, json!({ "attachments": [inline("a", &blob)] })).await,
        "attachments",
    );
    // Two parts with one cid.
    assert_invalid(
        &create(
            &h,
            with(
                html_with(&["a"]),
                json!({ "attachments": [inline("a", &blob), inline("a", &blob)] }),
            ),
        )
        .await,
        "attachments",
    );
    // A cid that would end the Content-ID header.
    assert_invalid(
        &create(
            &h,
            with(
                html_with(&["a"]),
                json!({ "attachments": [inline("a>\r\nX-Injected: <1", &blob)] }),
            ),
        )
        .await,
        "attachments",
    );
    // Not an image: declared, and as stored.
    assert_invalid(
        &create(
            &h,
            with(
                html_with(&["a"]),
                json!({ "attachments": [{
                    "blobId": blob, "cid": "a", "disposition": "inline", "type": "text/html"
                }] }),
            ),
        )
        .await,
        "attachments",
    );
    let script = h
        .engine
        .store_upload(&h.account_id, "text/html", b"<script>1</script>")
        .await
        .unwrap();
    assert_invalid(
        &create(
            &h,
            with(
                html_with(&["a"]),
                json!({ "attachments": [inline("a", &script)] }),
            ),
        )
        .await,
        "attachments",
    );
    // A blob of another account, and one that does not exist.
    let other = new_account(&h.store, "other@example.org").await;
    let foreign = h
        .engine
        .store_upload(&other, "image/png", PNG)
        .await
        .unwrap();
    for blob in [foreign.as_str(), "U0000"] {
        assert_invalid(
            &create(
                &h,
                with(
                    html_with(&["a"]),
                    json!({ "attachments": [inline("a", blob)] }),
                ),
            )
            .await,
            "attachments",
        );
    }

    assert_eq!(
        h.backend.appended.lock().unwrap()["Drafts"].len(),
        stored,
        "no refused create stored a draft"
    );
}

/// An entry that is not an inline part (no `cid`) is the ordinary attachment it
/// was before.
#[tokio::test]
async fn an_inline_disposition_without_a_cid_stays_an_ordinary_attachment() {
    let h = setup().await;
    let blob = h
        .engine
        .store_upload(&h.account_id, "image/png", PNG)
        .await
        .unwrap();
    let set = create(
        &h,
        json!({ "attachments": [{ "blobId": blob, "disposition": "inline", "name": "a.png" }] }),
    )
    .await;
    created_id(&set);
    let text = String::from_utf8_lossy(&h.backend.last_draft()).into_owned();
    assert!(text.contains("multipart/mixed"), "{text}");
    assert!(!text.contains("multipart/related"), "{text}");
    assert!(!text.contains("Content-ID"), "{text}");
}

// ── MDN/send ─────────────────────────────────────────────────────────────────

/// The whole path: the request is reported, the receipt goes to the request
/// address with a null reverse path, `$mdnsent` is on the message before the
/// submitter is called, and a second request is refused.
#[tokio::test]
async fn a_receipt_is_sent_once_to_the_request_address() {
    let h = setup().await;
    let id = find(&h, "request").await;

    let before = email(&h, &id).await;
    assert_eq!(
        before["mailwomanMdn"],
        json!({ "requestedBy": ANN, "sameAsSender": true, "fromList": false, "sent": false })
    );
    assert!(before["keywords"]["$mdnsent"].is_null(), "{before}");
    assert!(h.submitter.handed().is_empty(), "nothing sent by reading");

    *h.submitter.watched.lock().unwrap() = Some(id.clone());
    let sent = mdn_send(&h, &id, None).await;
    assert_eq!(
        sent,
        json!({
            "accountId": h.account_id, "emailId": id, "sent": true, "to": ANN, "automatic": false
        })
    );
    let handed = h.submitter.only();
    assert_receipt_to_ann(&handed);
    let report = mw_mime::parse_mdn(&handed.raw).unwrap();
    assert_eq!(
        report.original_message_id.as_deref(),
        Some("request@example.org")
    );
    assert_eq!(report.sending_mode, Some(MdnSendingMode::Manual));
    assert_eq!(report.action_mode, Some(MdnActionMode::Manual));
    assert_eq!(
        *h.submitter.marked_at_submit.lock().unwrap(),
        vec![true],
        "$mdnsent was set before the submitter was called"
    );

    let after = email(&h, &id).await;
    assert_eq!(after["mailwomanMdn"]["sent"], true, "{after}");
    assert_eq!(after["keywords"]["$mdnsent"], true, "{after}");

    for automatic in [None, Some(false), Some(true)] {
        let again = mdn_send(&h, &id, automatic).await;
        assert_eq!(again["type"], "mdnAlreadySent", "{again}");
    }
    assert_eq!(h.submitter.handed().len(), 1, "still one receipt");
}

/// A keyword replacement that drops `$mdnsent` does not make the message
/// answerable again.
#[tokio::test]
async fn dropping_the_keyword_does_not_allow_a_second_receipt() {
    let h = setup().await;
    let id = find(&h, "request").await;
    assert_eq!(mdn_send(&h, &id, None).await["sent"], true);

    let resp = jmap(
        &h,
        json!([["Email/set", { "update": { id.clone(): { "keywords": { "$seen": true } } } }, "u"]]),
    )
    .await;
    assert!(result(&resp, "u")["updated"].get(&id).is_some(), "{resp}");
    let got = email(&h, &id).await;
    assert!(got["keywords"]["$mdnsent"].is_null(), "keyword gone: {got}");
    assert_eq!(got["mailwomanMdn"]["sent"], true, "still reported as sent");

    assert_eq!(mdn_send(&h, &id, None).await["type"], "mdnAlreadySent");
    assert_eq!(h.submitter.handed().len(), 1);
}

/// Messages that ask for nothing, or that must not be answered.
#[tokio::test]
async fn a_message_without_an_answerable_request_gets_no_receipt() {
    let h = setup().await;
    for subject in [
        "plain",
        // A notification is not answered: a report, and a null return path.
        "Return receipt",
        "null return path",
        // Hostile request headers read as no request.
        "folded bcc",
        "two addresses",
        "two headers",
    ] {
        let id = find(&h, subject).await;
        let got = email(&h, &id).await;
        assert!(got["mailwomanMdn"].is_null(), "{subject}: {got}");
        for automatic in [None, Some(true)] {
            let refused = mdn_send(&h, &id, automatic).await;
            assert_eq!(refused["type"], "mdnNotRequested", "{subject}: {refused}");
        }
    }
    assert!(h.submitter.handed().is_empty(), "nothing was sent");

    // The received report is shown as a report.
    let report = email(&h, &find(&h, "Return receipt").await).await;
    assert_eq!(
        report["mailwomanMdnReport"],
        json!({
            "originalMessageId": "sent-earlier@mailwoman.local",
            "finalRecipient": ANN,
            "disposition": "displayed"
        })
    );
}

/// What a hostile original can do to a receipt: nothing. The receipt goes to
/// the one validated mailbox and carries the fixed header fields.
#[tokio::test]
async fn a_hostile_original_cannot_redirect_a_receipt_or_add_a_header() {
    let h = setup().await;

    // A display name that is an address: the mailbox in angle brackets is the
    // request address; the name is not used.
    let id = find(&h, "address in the name").await;
    assert_eq!(email(&h, &id).await["mailwomanMdn"]["requestedBy"], ANN);
    assert_eq!(mdn_send(&h, &id, None).await["sent"], true);

    // A subject that decodes to a line break followed by a Bcc field.
    let id = find(&h, "hostile subject").await;
    assert_eq!(mdn_send(&h, &id, None).await["sent"], true);

    let handed = h.submitter.handed();
    assert_eq!(handed.len(), 2);
    for sent in &handed {
        assert_receipt_to_ann(sent);
    }
    let subject = field_values(&handed[1].raw, "Subject");
    assert_eq!(subject.len(), 1, "one subject field: {subject:?}");
}

/// `automatic: true` needs the stored policy `always`, a request address equal
/// to the return path, and a message that is not list mail — all three.
#[tokio::test]
async fn an_automatic_receipt_needs_the_policy_the_sender_and_no_list() {
    let h = setup().await;
    let same = find(&h, "request").await;

    assert_eq!(h.engine.mdn_policy(&h.account_id).await.unwrap(), "ask");
    for policy in ["ask", "never", "sometimes"] {
        if policy != "ask" {
            set_policy(&h, policy).await;
        }
        let refused = mdn_send(&h, &same, Some(true)).await;
        assert_eq!(
            refused["type"], "mdnAutomaticRefused",
            "{policy}: {refused}"
        );
    }
    assert_eq!(
        h.engine.mdn_policy(&h.account_id).await.unwrap(),
        "ask",
        "a stored value that is no policy reads as ask"
    );

    set_policy(&h, "always").await;
    for subject in ["other address", "list mail", "no return path"] {
        let id = find(&h, subject).await;
        let refused = mdn_send(&h, &id, Some(true)).await;
        assert_eq!(
            refused["type"], "mdnAutomaticRefused",
            "{subject}: {refused}"
        );
        assert_eq!(
            email(&h, &id).await["mailwomanMdn"]["sent"],
            false,
            "{subject}"
        );
    }
    assert!(h.submitter.handed().is_empty(), "nothing sent so far");

    // The refusals above left the messages answerable by hand.
    let other = find(&h, "other address").await;
    assert_eq!(
        email(&h, &other).await["mailwomanMdn"],
        json!({
            "requestedBy": "boss@example.net", "sameAsSender": false, "fromList": false,
            "sent": false
        })
    );

    // All three conditions hold.
    let sent = mdn_send(&h, &same, Some(true)).await;
    assert_eq!(sent["sent"], true, "{sent}");
    assert_eq!(sent["automatic"], true);
    let handed = h.submitter.only();
    assert_eq!(handed.mail_from, "");
    assert_eq!(handed.rcpt_to, vec![ANN.to_string()]);
    assert_eq!(
        field_values(&handed.raw, "Auto-Submitted"),
        vec!["auto-replied".to_string()]
    );
    let report = mw_mime::parse_mdn(&handed.raw).unwrap();
    assert_eq!(report.sending_mode, Some(MdnSendingMode::Automatic));
}

/// Under `never` (and `ask`) a manual request is still honoured; the policy
/// governs what is sent without one.
#[tokio::test]
async fn a_manual_receipt_does_not_depend_on_the_policy() {
    let h = setup().await;
    set_policy(&h, "never").await;
    let id = find(&h, "other address").await;
    assert_eq!(
        mdn_send(&h, &id, Some(true)).await["type"],
        "mdnAutomaticRefused"
    );
    let sent = mdn_send(&h, &id, Some(false)).await;
    assert_eq!(sent["sent"], true, "{sent}");
    let handed = h.submitter.only();
    assert_eq!(handed.rcpt_to, vec!["boss@example.net".to_string()]);
    assert_eq!(handed.mail_from, "");
}

/// A failed submission leaves the message unanswered and answerable.
#[tokio::test]
async fn a_failed_submission_clears_the_mark() {
    let h = setup().await;
    let id = find(&h, "request").await;
    *h.submitter.watched.lock().unwrap() = Some(id.clone());

    h.submitter.down.store(true, Ordering::SeqCst);
    let failed = mdn_send(&h, &id, None).await;
    assert_eq!(failed["type"], "mdnNotSent", "{failed}");
    assert_eq!(
        *h.submitter.marked_at_submit.lock().unwrap(),
        vec![true],
        "the mark was set when the submitter was called"
    );
    let got = email(&h, &id).await;
    assert!(got["keywords"]["$mdnsent"].is_null(), "mark cleared: {got}");
    assert_eq!(got["mailwomanMdn"]["sent"], false, "{got}");
    assert!(h.submitter.handed().is_empty());

    h.submitter.down.store(false, Ordering::SeqCst);
    assert_eq!(mdn_send(&h, &id, None).await["sent"], true);
    assert_eq!(h.submitter.handed().len(), 1);
    assert_eq!(mdn_send(&h, &id, None).await["type"], "mdnAlreadySent");
}

/// No receipt for discarded mail or for the account's own drafts.
#[tokio::test]
async fn junk_and_drafts_are_not_answered() {
    let h = setup().await;
    set_policy(&h, "always").await;

    let junk = find_in(&h, "junk", "junk request").await;
    // The request is still reported; it is the sending that is refused.
    assert_eq!(email(&h, &junk).await["mailwomanMdn"]["requestedBy"], ANN);
    for automatic in [None, Some(true)] {
        let refused = mdn_send(&h, &junk, automatic).await;
        assert_eq!(refused["type"], "mdnNotAllowed", "{refused}");
    }

    let draft = created_id(
        &create(
            &h,
            json!({ "to": [{ "email": ME }], "mailwomanRequestReadReceipt": true }),
        )
        .await,
    );
    let refused = mdn_send(&h, &draft, None).await;
    assert_eq!(refused["type"], "mdnNotAllowed", "{refused}");

    assert!(h.submitter.handed().is_empty());
}

/// A receipt is sent only from an address of the account that the message was
/// addressed to.
#[tokio::test]
async fn a_receipt_needs_an_own_address_the_message_was_sent_to() {
    // Delivered through an address the account does not have.
    let h = setup().await;
    let id = find(&h, "to an alias").await;
    let refused = mdn_send(&h, &id, None).await;
    assert_eq!(refused["type"], "mdnNoRecipientAddress", "{refused}");
    assert_eq!(email(&h, &id).await["mailwomanMdn"]["sent"], false);
    assert!(h.submitter.handed().is_empty());

    // An account whose login name is a bare user name has no address at all.
    let bare = setup_as("me").await;
    let id = find(&bare, "request").await;
    let refused = mdn_send(&bare, &id, None).await;
    assert_eq!(refused["type"], "mdnNoRecipientAddress", "{refused}");
    assert!(bare.submitter.handed().is_empty());
}

/// Another account's message, an unknown id, bad arguments, and an account
/// that is no longer connected.
#[tokio::test]
async fn only_the_owner_of_a_connected_account_can_ask() {
    let h = setup().await;
    let id = find(&h, "request").await;

    for (args, kind) in [
        (json!({}), "invalidArguments"),
        (json!({ "emailId": 7 }), "invalidArguments"),
        (
            json!({ "emailId": id, "automatic": "yes" }),
            "invalidArguments",
        ),
        (json!({ "emailId": "0000" }), "notFound"),
    ] {
        let resp = jmap(&h, json!([["MDN/send", args, "s"]])).await;
        assert_eq!(result(&resp, "s")["type"], kind, "{resp}");
    }

    // A second connected account cannot answer the first one's message.
    let other_id = new_account(&h.store, "other@example.org").await;
    h.engine.register_backend(
        other_id.clone(),
        AccountRuntime::new(
            h.backend.clone() as Arc<dyn AccountBackend>,
            h.submitter.clone() as Arc<dyn MailSubmitter>,
            "other@example.org",
        ),
    );
    let call = json!({ "methodCalls": [["MDN/send", { "emailId": id }, "s"]] });
    let resp = h.engine.handle_jmap(&other_id, &call).await;
    assert_eq!(result(&resp, "s")["type"], "notFound", "{resp}");

    // What the server does to a disabled account: its runtime is removed.
    h.engine.unregister(&h.account_id);
    let resp = h.engine.handle_jmap(&h.account_id, &call).await;
    assert_eq!(result(&resp, "s")["type"], "accountNotFound", "{resp}");

    assert!(h.submitter.handed().is_empty(), "nothing was sent");
}

// ── the two mailbox validators ───────────────────────────────────────────────

/// `mw-mime` repeats `mw-smtp`'s mailbox rule because it cannot depend on it.
/// They must accept and refuse the same strings: the engine checks an address
/// with one and has it written by the other.
#[test]
fn the_mime_and_smtp_mailbox_validators_agree() {
    let long = format!("{}@example.test", "a".repeat(320));
    let longest_accepted = format!("{}@example.test", "a".repeat(307));
    let vectors = [
        // mw-smtp/src/addr.rs `ordinary_mailboxes_pass`
        "a@b",
        "bob@example.test",
        "first.last+tag@sub.example.test",
        "møt@example.com",
        "user@[192.0.2.1]",
        "o'brien@example.test",
        // `injection_shapes_are_refused_on_both_paths`
        "x@example.test>\r\nRCPT TO:<victim@example.test",
        "x@example.test\nDATA",
        "x@example.test\r",
        "x@example.\0test",
        "x@example.test\u{7f}",
        "x@example.test\u{85}",
        "x@example.test\u{2028}",
        "x @example.test",
        "x@example.test ",
        "\tx@example.test",
        "a@b>",
        "<a@b",
        "a@b> SIZE=1",
        "\"a b\"@example.test",
        "a\\@b@example.test",
        "a@b,c@d",
        "a@b;c@d",
        "a(comment)@b",
        "a@b@c",
        // `a_mailbox_needs_both_sides_of_one_at`
        "",
        "bob",
        "@example.test",
        "bob@",
        "@",
        long.as_str(),
        // The length limit from both sides, and a few more shapes.
        longest_accepted.as_str(),
        "a@b\u{a0}",
        "a@b:c",
        "Ann <a@b>",
        "a@[IPv6:2001:db8::1]",
    ];
    let mut accepted = 0;
    for v in vectors {
        let mime = mw_mime::validate_mailbox(v).is_ok();
        let smtp = mw_smtp::validate_mailbox(v).is_ok();
        assert_eq!(mime, smtp, "mw-mime and mw-smtp disagree on {v:?}");
        accepted += usize::from(mime);
    }
    assert_eq!(longest_accepted.len(), 320);
    assert_eq!(accepted, 9, "the vectors exercise both outcomes");
}
