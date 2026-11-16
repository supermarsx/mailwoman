//! Provider-produced response shapes, including page tokens and label removals.
use bridge_gmail::backend::{GmailBackend, HttpResponse, Transport};
use bridge_gmail::model::{BridgeError, MailboxRef, Result, SyncCursor};
use std::cell::RefCell;

struct Script(RefCell<Vec<(&'static str, u16, &'static str)>>);
impl Transport for &Script {
    fn oauth_token(&self, _: &str) -> Result<String> {
        Ok("fixture-access".into())
    }
    fn http(
        &self,
        method: &str,
        url: &str,
        _: Vec<(String, String)>,
        _: Option<Vec<u8>>,
    ) -> Result<HttpResponse> {
        assert_eq!(method, "GET");
        let (path, status, body) = self.0.borrow_mut().remove(0);
        assert!(url.ends_with(path), "expected {path}, received {url}");
        Ok(HttpResponse {
            status,
            body: body.as_bytes().to_vec(),
        })
    }
}
const LABELS: &str = r#"{"labels":[{"id":"INBOX","name":"INBOX","type":"system"}]}"#;
fn inbox() -> MailboxRef {
    MailboxRef {
        name: "INBOX".into(),
        uidvalidity: 1,
    }
}

#[test]
fn full_sync_reads_every_page_and_captures_head_before_enumerating() {
    let script = Script(RefCell::new(vec![
        ("/labels", 200, LABELS),
        ("/profile", 200, r#"{"historyId":"100"}"#),
        (
            "messages?labelIds=INBOX&maxResults=500&includeSpamTrash=true",
            200,
            r#"{"messages":[{"id":"a"}],"nextPageToken":"p +/"}"#,
        ),
        (
            "messages?labelIds=INBOX&maxResults=500&includeSpamTrash=true&pageToken=p%20%2B%2F",
            200,
            r#"{"messages":[{"id":"b"}]}"#,
        ),
    ]));
    let delta = GmailBackend::new(&script)
        .sync_mailbox(&inbox(), &SyncCursor { opaque: vec![] })
        .unwrap();
    assert_eq!(delta.added.len(), 2);
    assert_eq!(delta.next_cursor.history_id().as_deref(), Some("100"));
    assert!(script.0.borrow().is_empty());
}

#[test]
fn history_consumes_pages_before_advancing_and_tracks_label_membership() {
    let script = Script(RefCell::new(vec![
        ("/labels", 200, LABELS),
        (
            "history?startHistoryId=100&maxResults=500",
            200,
            r#"{"historyId":"200","nextPageToken":"p2","history":[{"labelsRemoved":[{"message":{"id":"gone","labelIds":["UNREAD"]},"labelIds":["INBOX"]}]}]}"#,
        ),
        (
            "history?startHistoryId=100&maxResults=500&pageToken=p2",
            200,
            r#"{"historyId":"201","history":[{"labelsAdded":[{"message":{"id":"arrived","labelIds":["INBOX","STARRED"]},"labelIds":["INBOX"]}]},{"messagesAdded":[{"message":{"id":"elsewhere","labelIds":["SENT"]}}]}]}"#,
        ),
    ]));
    let delta = GmailBackend::new(&script)
        .sync_mailbox(&inbox(), &SyncCursor::from_history_id("100"))
        .unwrap();
    assert_eq!(
        delta
            .added
            .iter()
            .map(|r| r.decode_gmail().unwrap().1)
            .collect::<Vec<_>>(),
        ["arrived"]
    );
    assert_eq!(
        delta
            .removed
            .iter()
            .map(|r| r.decode_gmail().unwrap().1)
            .collect::<Vec<_>>(),
        ["gone"]
    );
    assert_eq!(
        delta.flag_changes.len(),
        0,
        "new members get current flags with their MIME fetch"
    );
    assert_eq!(delta.next_cursor.history_id().as_deref(), Some("201"));
    assert!(script.0.borrow().is_empty());
}

#[test]
fn expired_history_requests_engine_reconciliation_instead_of_missing_mailbox() {
    let script = Script(RefCell::new(vec![
        ("/labels", 200, LABELS),
        ("history?startHistoryId=1&maxResults=500", 404, "{}"),
    ]));
    assert!(
        matches!(GmailBackend::new(&script).sync_mailbox(&inbox(),&SyncCursor::from_history_id("1")),Err(BridgeError::Protocol(ref message)) if message=="mailwoman:sync-reset")
    );
}

#[test]
fn repeated_page_token_fails_without_committing_a_partial_cursor() {
    let script = Script(RefCell::new(vec![
        ("/labels", 200, LABELS),
        ("/profile", 200, r#"{"historyId":"100"}"#),
        (
            "messages?labelIds=INBOX&maxResults=500&includeSpamTrash=true",
            200,
            r#"{"nextPageToken":"again"}"#,
        ),
        (
            "messages?labelIds=INBOX&maxResults=500&includeSpamTrash=true&pageToken=again",
            200,
            r#"{"nextPageToken":"again"}"#,
        ),
    ]));
    assert!(matches!(
        GmailBackend::new(&script).sync_mailbox(&inbox(), &SyncCursor { opaque: vec![] }),
        Err(BridgeError::Protocol(_))
    ));
    assert!(script.0.borrow().is_empty());
}
