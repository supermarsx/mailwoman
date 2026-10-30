//! t94: the Focused inbox split, decided by the server.
//!
//! `Email/get` fills [`FOCUS_PROPERTY`] (`mailwomanFocus`) when the request
//! names it: `{"tab": "focused"|"other", "reason": ..., "header"?: ...}`. The
//! rules, first match wins:
//!
//! 1. a list, bulk or automated-mail header (`List-Unsubscribe`, `List-Id`,
//!    `Precedence: bulk|list|junk`, `Auto-Submitted` other than `no`) means
//!    Other (`reason: "listHeader"`, `header` names it);
//! 2. a sender in the account's contacts (`"contact"`) or one the user has
//!    written to (`"correspondent"`) means Focused;
//! 3. a message in a thread the user took part in means Focused (`"thread"`);
//! 4. a sender whose name or address looks automated (`no-reply`,
//!    `newsletter`, ...) means Other (`"keyword"`);
//! 5. otherwise Focused (`"default"`).
//!
//! The header signal is read once at ingest and kept on the message's search
//! document ([`mw_search::IndexDoc::list_signal`]), so a list page needs no
//! body read; an index built before t94 is rebuilt from the store at start-up
//! (the backfill). "Written to" and "took part in" come from the newest
//! [`SENT_SCAN`] messages of the Sent folder: their thread ids from the store,
//! and the recipients through the index (`to`/`cc`). A zero-access account has
//! no index, so for it rules 1 and the correspondent half of 2 do not fire.
//!
//! A user's own per-sender choices ("Always move to Focused / Other") are not
//! applied here: they are stored per user by `mw-server`
//! (`/api/account/focused-senders`) and the client lays them over this result.

use std::collections::HashSet;

use serde_json::{Value, json};

use crate::backend::Result;
use crate::engine::Engine;
use mw_search::{Clause, Expr, SearchQuery, Sort, TextField};

/// The `Email/get` property with the server's Focused / Other decision.
pub const FOCUS_PROPERTY: &str = "mailwomanFocus";

/// How many of the newest Sent messages count as "mail you wrote".
pub const SENT_SCAN: i64 = 500;

/// The first header of `raw` that marks it as list, bulk or automated mail,
/// lower-cased (`list-unsubscribe`, `list-id`, `precedence`, `auto-submitted`),
/// or `None`. Reads the header block only; folded lines are joined.
pub fn list_signal(raw: &[u8]) -> Option<String> {
    let end = header_end(raw);
    let head = String::from_utf8_lossy(&raw[..end]);
    let mut headers: Vec<String> = Vec::new();
    for line in head.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.starts_with([' ', '\t']) {
            if let Some(last) = headers.last_mut() {
                last.push(' ');
                last.push_str(line.trim());
            }
        } else if !line.is_empty() {
            headers.push(line.to_string());
        }
    }
    headers.iter().find_map(|h| {
        let (name, value) = h.split_once(':')?;
        let name = name.trim().to_ascii_lowercase();
        let value = value
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        let hit = match name.as_str() {
            "list-unsubscribe" | "list-id" => true,
            "precedence" => matches!(value.as_str(), "bulk" | "list" | "junk"),
            "auto-submitted" => !value.is_empty() && value != "no",
            _ => false,
        };
        hit.then_some(name)
    })
}

/// Where the header block of `raw` ends (the blank line, or the whole input).
fn header_end(raw: &[u8]) -> usize {
    let crlf = raw.windows(4).position(|w| w == b"\r\n\r\n");
    let lf = raw.windows(2).position(|w| w == b"\n\n");
    match (crlf, lf) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => raw.len(),
    }
}

/// The automated-sender words of the client's original heuristic (t73).
const BULK_WORDS: [&str; 11] = [
    "noreply",
    "no-reply",
    "newsletter",
    "notification",
    "marketing",
    "update",
    "digest",
    "mailer",
    "automated",
    "donotreply",
    "do-not-reply",
];

/// Whether a sender's name or address looks automated.
pub fn bulk_keyword(name: &str, email: &str) -> bool {
    let hay = format!("{name} {email}").to_lowercase();
    BULK_WORDS.iter().any(|w| hay.contains(w))
        || hay.contains("do-notreply")
        || hay.contains("donot-reply")
}

/// The two inbox tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Focused,
    Other,
}

/// Why a message got its tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    ListHeader,
    Contact,
    Correspondent,
    Thread,
    Keyword,
    Default,
}

impl Reason {
    fn as_str(self) -> &'static str {
        match self {
            Reason::ListHeader => "listHeader",
            Reason::Contact => "contact",
            Reason::Correspondent => "correspondent",
            Reason::Thread => "thread",
            Reason::Keyword => "keyword",
            Reason::Default => "default",
        }
    }
}

/// What is known about one message.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Signals {
    pub list_header: Option<String>,
    pub contact: bool,
    pub correspondent: bool,
    pub thread: bool,
    pub keyword: bool,
}

/// The rules, in order (see the module docs).
pub fn decide(s: &Signals) -> (Tab, Reason) {
    if s.list_header.is_some() {
        (Tab::Other, Reason::ListHeader)
    } else if s.contact {
        (Tab::Focused, Reason::Contact)
    } else if s.correspondent {
        (Tab::Focused, Reason::Correspondent)
    } else if s.thread {
        (Tab::Focused, Reason::Thread)
    } else if s.keyword {
        (Tab::Other, Reason::Keyword)
    } else {
        (Tab::Focused, Reason::Default)
    }
}

/// The JSON value of [`FOCUS_PROPERTY`].
pub fn focus_json(s: &Signals) -> Value {
    let (tab, reason) = decide(s);
    let mut out = json!({
        "tab": if tab == Tab::Focused { "focused" } else { "other" },
        "reason": reason.as_str(),
    });
    if let (Reason::ListHeader, Some(h)) = (reason, &s.list_header) {
        out["header"] = json!(h);
    }
    out
}

/// The `EMAIL` values of a vCard, lower-cased (any group prefix, any params).
pub fn vcard_emails(vcard: &str) -> Vec<String> {
    vcard
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            let prop = name.split(';').next()?.rsplit('.').next()?;
            prop.eq_ignore_ascii_case("EMAIL")
                .then(|| value.trim().trim_start_matches("mailto:").to_lowercase())
        })
        .filter(|v| !v.is_empty())
        .collect()
}

/// The first `from` address of a JMAP email, lower-cased, with its name.
fn sender_of(email: &Value) -> Option<(String, String)> {
    let first = email.get("from")?.as_array()?.first()?;
    let addr = first.get("email")?.as_str()?.trim().to_lowercase();
    if addr.is_empty() {
        return None;
    }
    let name = first
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Some((addr, name))
}

/// Per-request knowledge shared by every message of one `Email/get`.
#[derive(Debug, Default)]
struct FocusContext {
    sent_mailbox: Option<String>,
    sent_ids: HashSet<String>,
    my_threads: HashSet<String>,
    contacts: HashSet<String>,
    correspondents: HashSet<String>,
}

impl Engine {
    /// Fill [`FOCUS_PROPERTY`] on every assembled email (t94).
    pub(crate) async fn fill_focus(
        &self,
        account_id: &str,
        emails: &mut [Option<Value>],
    ) -> Result<()> {
        let senders: HashSet<String> = emails
            .iter()
            .flatten()
            .filter_map(sender_of)
            .map(|(addr, _)| addr)
            .collect();
        if emails.iter().all(Option::is_none) {
            return Ok(());
        }
        let ctx = self.focus_context(account_id, &senders).await?;
        for email in emails.iter_mut().flatten() {
            let id = email
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let sender = sender_of(email);
            let list_header = self
                .search()
                .fetch_doc(&id)
                .ok()
                .flatten()
                .and_then(|doc| doc.list_signal);
            let in_sent = ctx.sent_ids.contains(&id)
                || ctx.sent_mailbox.as_ref().is_some_and(|sent| {
                    email
                        .get("mailboxIds")
                        .and_then(Value::as_object)
                        .is_some_and(|m| m.contains_key(sent))
                });
            let thread = !in_sent
                && email
                    .get("threadId")
                    .and_then(Value::as_str)
                    .is_some_and(|t| ctx.my_threads.contains(t));
            let signals = Signals {
                list_header,
                contact: sender
                    .as_ref()
                    .is_some_and(|(a, _)| ctx.contacts.contains(a)),
                correspondent: sender
                    .as_ref()
                    .is_some_and(|(a, _)| ctx.correspondents.contains(a)),
                thread,
                keyword: sender.as_ref().is_some_and(|(a, n)| bulk_keyword(n, a)),
            };
            email[FOCUS_PROPERTY] = focus_json(&signals);
        }
        Ok(())
    }

    async fn focus_context(
        &self,
        account_id: &str,
        senders: &HashSet<String>,
    ) -> Result<FocusContext> {
        let mut ctx = FocusContext::default();
        let sent = self
            .store()
            .list_mailboxes(account_id)
            .await?
            .into_iter()
            .find(|m| m.role.as_deref() == Some("sent"));
        if let Some(sent) = &sent {
            let ids = self
                .store()
                .list_message_ids(&sent.id, SENT_SCAN, 0)
                .await?;
            for msg in self.store().get_messages(&ids).await?.into_iter().flatten() {
                if let Some(t) = msg.thread_id {
                    ctx.my_threads.insert(t);
                }
            }
            ctx.sent_ids = ids.into_iter().collect();
            ctx.sent_mailbox = Some(sent.id.clone());
        }
        for addr in senders {
            // Contacts: the store's own match over `vcard_raw`, confirmed on
            // the card's EMAIL values so a substring never counts.
            let cards = self
                .store()
                .autocomplete_contacts(account_id, addr, 20)
                .await?;
            if cards
                .iter()
                .any(|c| vcard_emails(&c.vcard_raw).iter().any(|e| e == addr))
            {
                ctx.contacts.insert(addr.clone());
            }
            if let Some(sent) = &sent
                && self.wrote_to(&sent.id, &ctx.sent_ids, addr)
            {
                ctx.correspondents.insert(addr.clone());
            }
        }
        Ok(ctx)
    }

    /// Whether one of the scanned Sent messages has `addr` in To or Cc, read
    /// from the index (a phrase match, confirmed on the stored document).
    fn wrote_to(&self, sent_mailbox: &str, scanned: &HashSet<String>, addr: &str) -> bool {
        let text = |field| {
            Expr::Clause(Clause::Text {
                field,
                value: addr.to_string(),
                phrase: true,
            })
        };
        let query = SearchQuery {
            raw: String::new(),
            expr: Expr::And(vec![
                Expr::Clause(Clause::Mailbox(sent_mailbox.to_string())),
                Expr::Or(vec![text(TextField::To), text(TextField::Cc)]),
            ]),
            sort: Sort::default(),
        };
        let Ok(hits) = self.search().search(&query, 20) else {
            return false;
        };
        hits.iter().filter(|id| scanned.contains(*id)).any(|id| {
            self.search()
                .fetch_doc(id)
                .ok()
                .flatten()
                .is_some_and(|doc| {
                    format!("{} {}", doc.to, doc.cc)
                        .to_lowercase()
                        .split(|c: char| c.is_whitespace() || c == ',' || c == '<' || c == '>')
                        .any(|word| word == addr)
                })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_signal_reads_each_header_and_ignores_the_body() {
        let mail = |h: &str| format!("From: a@x\r\n{h}\r\nSubject: s\r\n\r\nbody\r\n");
        assert_eq!(
            list_signal(mail("List-Unsubscribe: <mailto:u@x>").as_bytes()).as_deref(),
            Some("list-unsubscribe")
        );
        assert_eq!(
            list_signal(mail("list-id: News <news.x>").as_bytes()).as_deref(),
            Some("list-id")
        );
        for p in ["bulk", "list", "junk", " Bulk "] {
            assert_eq!(
                list_signal(mail(&format!("Precedence: {p}")).as_bytes()).as_deref(),
                Some("precedence"),
                "{p}"
            );
        }
        assert_eq!(
            list_signal(mail("Precedence: first-class").as_bytes()),
            None
        );
        assert_eq!(
            list_signal(mail("Auto-Submitted: auto-generated").as_bytes()).as_deref(),
            Some("auto-submitted")
        );
        assert_eq!(
            list_signal(mail("Auto-Submitted: auto-replied; owner=x").as_bytes()).as_deref(),
            Some("auto-submitted")
        );
        assert_eq!(list_signal(mail("Auto-Submitted: no").as_bytes()), None);
        assert_eq!(list_signal(mail("X-Other: y").as_bytes()), None);
        // A header-looking line in the body is not a header.
        assert_eq!(list_signal(b"From: a@x\r\n\r\nList-Id: <x>\r\n"), None);
        // Folded value, LF-only line ends.
        assert_eq!(
            list_signal(b"From: a@x\nPrecedence:\n bulk\n\nbody").as_deref(),
            Some("precedence")
        );
    }

    #[test]
    fn rules_apply_in_order() {
        let base = Signals::default();
        assert_eq!(decide(&base), (Tab::Focused, Reason::Default));
        let all = Signals {
            list_header: Some("list-id".into()),
            contact: true,
            correspondent: true,
            thread: true,
            keyword: true,
        };
        assert_eq!(decide(&all), (Tab::Other, Reason::ListHeader));
        let known = Signals {
            list_header: None,
            ..all.clone()
        };
        assert_eq!(decide(&known), (Tab::Focused, Reason::Contact));
        let wrote = Signals {
            contact: false,
            ..known.clone()
        };
        assert_eq!(decide(&wrote), (Tab::Focused, Reason::Correspondent));
        let thread = Signals {
            correspondent: false,
            ..wrote.clone()
        };
        assert_eq!(decide(&thread), (Tab::Focused, Reason::Thread));
        let keyword = Signals {
            thread: false,
            ..thread
        };
        assert_eq!(decide(&keyword), (Tab::Other, Reason::Keyword));
        assert_eq!(
            focus_json(&all),
            json!({"tab":"other","reason":"listHeader","header":"list-id"})
        );
        assert_eq!(
            focus_json(&base),
            json!({"tab":"focused","reason":"default"})
        );
    }

    #[test]
    fn keyword_rule_matches_the_t73_heuristic() {
        for (name, email) in [
            ("", "no-reply@x.test"),
            ("", "noreply@x.test"),
            ("Shop News", "newsletter@shop.example"),
            ("", "notifications@git.example"),
            ("Weekly Digest", "a@b.test"),
            ("", "do-not-reply@x.test"),
            ("", "donotreply@x.test"),
            ("", "mailer-daemon@x.test"),
        ] {
            assert!(bulk_keyword(name, email), "{name} {email}");
        }
        for (name, email) in [("Ana Person", "ana@example.net"), ("", "replies@x.test")] {
            assert!(!bulk_keyword(name, email), "{name} {email}");
        }
    }

    #[test]
    fn vcard_emails_reads_every_email_property() {
        let card = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Ann\r\nEMAIL;TYPE=work:Ann@Example.org\r\nitem1.EMAIL:mailto:two@x.test\r\nNOTE:other@example.org\r\nEND:VCARD\r\n";
        assert_eq!(vcard_emails(card), vec!["ann@example.org", "two@x.test"]);
    }
}
