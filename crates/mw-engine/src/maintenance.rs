//! Admin-opt-in, one-shot engine maintenance (plan t14 §WS3 / E5).
//!
//! 26.13 shipped JWZ threading as **new-ingest-only**: newly-arriving mail is
//! threaded, historical `thread_id`s are never re-keyed ([`crate::thread`]). This
//! module adds the deliberate, explicit escape hatch — a **one-shot re-thread**
//! that runs the full, corpora-tested JWZ *set* algorithm ([`thread::thread`])
//! over an account's complete stored message set and re-keys every message's
//! `thread_id` accordingly.
//!
//! ## Never automatic
//! Nothing in the engine calls this on its own. The sole callers are the admin
//! surfaces `mw-server` mounts (an admin-session-gated endpoint + a
//! `maintenance rethread` CLI subcommand). Re-keying `thread_id` is a **visible**
//! change to thread identity, so it is strictly admin opt-in.
//!
//! ## Idempotent, no migration
//! The thread key for a group is its RFC 5256 thread root (`References[0]`, else
//! `In-Reply-To`, else the message's own `Message-ID`) mapped through the store's
//! [`assign_thread`](mw_store::Store::assign_thread), which persists a stable
//! `root_message_id → thread_id` row. The same corpus therefore always produces
//! the same forest, the same root keys, and — because `assign_thread` reuses the
//! existing row — the same `thread_id`s. A second run rewrites each message to
//! the value it already holds: a no-op. It reuses the shipped `messages` /
//! `threads` tables via `set_thread` / `assign_thread`, so **no migration** is
//! needed.

use std::collections::{HashMap, HashSet};

use crate::backend::Result;
use crate::engine::Engine;
use crate::thread::{self, Message as ThreadMessage, ThreadNode};

/// Page size for enumerating a mailbox's stored message ids.
const PAGE: i64 = 500;

/// What a re-thread pass touched. Per-account calls return `accounts == 1`;
/// [`merge`](RethreadSummary::merge) lets a caller (the admin endpoint / CLI)
/// aggregate several accounts into one report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RethreadSummary {
    /// Accounts re-threaded (1 per [`Engine::rethread_account`] call).
    pub accounts: usize,
    /// Stored messages considered.
    pub messages: usize,
    /// Distinct threads (re-)keyed.
    pub threads: usize,
    /// Messages whose `thread_id` actually changed (0 on an idempotent re-run).
    pub reassigned: usize,
}

impl RethreadSummary {
    /// Fold another account's summary into this one.
    pub fn merge(&mut self, other: &RethreadSummary) {
        self.accounts += other.accounts;
        self.messages += other.messages;
        self.threads += other.threads;
        self.reassigned += other.reassigned;
    }
}

/// One stored message reconstructed for re-threading.
struct Loaded {
    stable_id: String,
    current_thread: Option<String>,
    msg: ThreadMessage,
}

/// Re-thread every message of `account_id` in one shot (see the module docs).
///
/// Invoked only through [`Engine::rethread_account`]; never automatic.
pub(crate) async fn rethread_account(engine: &Engine, account_id: &str) -> Result<RethreadSummary> {
    let store = engine.store();

    // 1. Enumerate every stored message for the account across all mailboxes,
    //    reconstructing the JWZ `Message` from each sealed body. The envelope
    //    (`envelope_json`) does not carry References/In-Reply-To, so the raw body
    //    is re-parsed; the `message_id` column is the fallback identity.
    let mut loaded: Vec<Loaded> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for mb in store.list_mailboxes(account_id).await? {
        let mut offset = 0i64;
        loop {
            let ids = store.list_message_ids(&mb.id, PAGE, offset).await?;
            let got = ids.len() as i64;
            for sid in ids {
                if !seen.insert(sid.clone()) {
                    continue; // a message lives in exactly one mailbox; guard anyway
                }
                if let Some(l) = load_message(engine, &sid).await? {
                    loaded.push(l);
                }
            }
            if got < PAGE {
                break;
            }
            offset += PAGE;
        }
    }

    // Deterministic input order → deterministic grouping + keys across re-runs.
    loaded.sort_by(|a, b| a.stable_id.cmp(&b.stable_id));

    // Message-ID → indices into `loaded` (duplicates share a Message-ID).
    let mut by_mid: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, l) in loaded.iter().enumerate() {
        if let Some(mid) = &l.msg.message_id {
            by_mid.entry(mid.clone()).or_default().push(i);
        }
    }

    // 2. Run the full, corpora-tested JWZ set algorithm for the grouping (the
    //    prune + subject-gather the incremental ingest path deliberately skips).
    let msgs: Vec<ThreadMessage> = loaded.iter().map(|l| l.msg.clone()).collect();
    let forest = thread::thread(&msgs);

    // 3. Re-key each thread on its RFC 5256 root, then write every member.
    let mut summary = RethreadSummary {
        accounts: 1,
        ..Default::default()
    };
    for tree in &forest {
        let mut mids = Vec::new();
        collect_mids(tree, &mut mids);
        mids.sort();
        mids.dedup();
        // An all-id-less group has no stable key; leave it unthreaded, matching
        // the new-ingest path (a message with no usable identity is never
        // threaded). Such messages also carry `message_id = None`, so they are
        // not represented in `mids` and are untouched here.
        let Some(key) = root_key(&mids, &by_mid, &loaded) else {
            continue;
        };
        let thread_id = store.assign_thread(account_id, &key).await?;
        summary.threads += 1;
        for mid in &mids {
            let Some(idxs) = by_mid.get(mid) else {
                continue;
            };
            for &i in idxs {
                let l = &loaded[i];
                if l.current_thread.as_deref() != Some(thread_id.as_str()) {
                    summary.reassigned += 1;
                }
                store.set_thread(&l.stable_id, &thread_id).await?;
            }
        }
    }
    summary.messages = loaded.len();
    Ok(summary)
}

/// Reconstruct one stored message's JWZ inputs. `Ok(None)` if it vanished
/// mid-scan.
async fn load_message(engine: &Engine, stable_id: &str) -> Result<Option<Loaded>> {
    let store = engine.store();
    let msg = match store.get_message(stable_id).await {
        Ok(m) => m,
        Err(mw_store::StoreError::NotFound) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    // Prefer the sealed body (carries the References/In-Reply-To chain); fall
    // back to just the `message_id` column when the body is absent/unparseable.
    let parsed = match &msg.blob_ref {
        Some(blob) => match store.get_body(blob).await? {
            Some(raw) => mw_mime::parse(&raw).ok(),
            None => None,
        },
        None => None,
    };
    let tmsg = match parsed {
        Some(p) => ThreadMessage::from_envelope(&p.envelope, p.email.subject.as_deref()),
        None => ThreadMessage {
            message_id: msg.message_id.clone(),
            ..ThreadMessage::default()
        },
    };
    Ok(Some(Loaded {
        stable_id: stable_id.to_string(),
        current_thread: msg.thread_id.clone(),
        msg: tmsg,
    }))
}

/// Depth-first collect of the real Message-IDs in a container tree (phantom /
/// synthetic containers carry `None` and are skipped).
fn collect_mids(node: &ThreadNode, out: &mut Vec<String>) {
    if let Some(mid) = &node.message_id {
        out.push(mid.clone());
    }
    for c in &node.children {
        collect_mids(c, out);
    }
}

/// The stable thread key for a tree: the lexicographically-smallest RFC 5256
/// thread root over its member messages. Every reply in a thread shares
/// `References[0]` (the original's id), and the original itself contributes its
/// own `Message-ID`, so this recovers the thread's root — even a never-seen
/// (phantom) original — and lands on the same id the new-ingest path would.
/// `None` only for an all-id-less group.
fn root_key(
    mids: &[String],
    by_mid: &HashMap<String, Vec<usize>>,
    loaded: &[Loaded],
) -> Option<String> {
    let mut best: Option<String> = None;
    for mid in mids {
        let Some(idxs) = by_mid.get(mid) else {
            continue;
        };
        for &i in idxs {
            if let Some(cand) = message_root_candidate(&loaded[i].msg)
                && best.as_deref().is_none_or(|b| cand.as_str() < b)
            {
                best = Some(cand);
            }
        }
    }
    best
}

/// The RFC 5256 thread-root candidate for a single message: `References[0]`,
/// else `In-Reply-To`, else its own `Message-ID`.
fn message_root_candidate(m: &ThreadMessage) -> Option<String> {
    m.references
        .first()
        .cloned()
        .or_else(|| m.in_reply_to.clone())
        .or_else(|| m.message_id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::{
        AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
    };

    /// A minimal RFC822 message with the threading headers under test.
    fn raw_msg(mid: &str, refs: &[&str], irt: Option<&str>, subject: &str) -> Vec<u8> {
        let mut h = format!(
            "Message-ID: <{mid}>\r\n\
             From: s@x\r\n\
             To: me@x\r\n\
             Subject: {subject}\r\n\
             Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n"
        );
        if !refs.is_empty() {
            let joined = refs
                .iter()
                .map(|r| format!("<{r}>"))
                .collect::<Vec<_>>()
                .join(" ");
            h.push_str(&format!("References: {joined}\r\n"));
        }
        if let Some(i) = irt {
            h.push_str(&format!("In-Reply-To: <{i}>\r\n"));
        }
        h.push_str("\r\nbody text\r\n");
        h.into_bytes()
    }

    async fn seed(
        store: &Store,
        account: &str,
        mailbox: &str,
        uid: u32,
        mid: &str,
        raw: &[u8],
    ) -> String {
        let blob = store.put_body(account, raw).await.unwrap();
        store
            .upsert_message(&MessageUpsert {
                account_id: account,
                mailbox_id: mailbox,
                uid,
                uidvalidity: 1,
                message_id: Some(mid),
                thread_id: None,
                internaldate: Some("2026-07-01T09:00:00Z"),
                size: raw.len() as u64,
                flags_json: "[]",
                envelope: None,
                blob_ref: Some(&blob),
            })
            .await
            .unwrap()
    }

    /// Out-of-order corpus (replies before their original + a truncated
    /// References chain) re-threads to the full-set JWZ grouping and a second
    /// run is a no-op.
    #[tokio::test]
    async fn backfill_converges_and_is_idempotent() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "imap.example.org",
                    port: 993,
                    tls: "implicit",
                    username: "me@x",
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: "me@x".into(),
                    password: "pw".into(),
                },
            )
            .await
            .unwrap();
        let mailbox = store
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 1,
                uidnext: 100,
                highestmodseq: 0,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();

        // Seed OUT OF ORDER: replies (incl. one with a truncated References chain
        // that only reaches `a@x`) arrive before the original; an unrelated
        // message forms a second thread.
        let b = seed(
            &store,
            &account,
            &mailbox,
            1,
            "b@x",
            &raw_msg("b@x", &["o@x", "a@x"], Some("a@x"), "Re: Plan"),
        )
        .await;
        let c = seed(
            &store,
            &account,
            &mailbox,
            2,
            "c@x",
            &raw_msg("c@x", &["a@x"], Some("a@x"), "Re: Plan"), // truncated: only reaches a@x
        )
        .await;
        let a = seed(
            &store,
            &account,
            &mailbox,
            3,
            "a@x",
            &raw_msg("a@x", &["o@x"], Some("o@x"), "Re: Plan"),
        )
        .await;
        let o = seed(
            &store,
            &account,
            &mailbox,
            4,
            "o@x",
            &raw_msg("o@x", &[], None, "Plan"),
        )
        .await;
        let other = seed(
            &store,
            &account,
            &mailbox,
            5,
            "other@x",
            &raw_msg("other@x", &[], None, "Something else"),
        )
        .await;

        let engine = Engine::new(store);

        // Nothing is threaded yet.
        for id in [&o, &a, &b, &c, &other] {
            assert_eq!(
                engine.store().get_message(id).await.unwrap().thread_id,
                None
            );
        }

        // ---- first run: re-thread ----
        let s1 = engine.rethread_account(&account).await.unwrap();
        assert_eq!(s1.accounts, 1);
        assert_eq!(s1.messages, 5);
        assert_eq!(
            s1.threads, 2,
            "the reply chain + the standalone = 2 threads"
        );
        assert_eq!(s1.reassigned, 5, "every message moved from unthreaded");

        let tid = |id: &str| {
            let engine = &engine;
            let id = id.to_string();
            async move { engine.store().get_message(&id).await.unwrap().thread_id }
        };
        let to = tid(&o).await;
        let ta = tid(&a).await;
        let tb = tid(&b).await;
        let tc = tid(&c).await;
        let tother = tid(&other).await;

        // The reply chain (incl. the truncated-References `c@x`) converges.
        assert!(to.is_some());
        assert_eq!(to, ta);
        assert_eq!(to, tb);
        assert_eq!(to, tc, "truncated References chain still converges");
        // The unrelated message is a distinct thread.
        assert!(tother.is_some());
        assert_ne!(to, tother);

        // ---- second run: idempotent no-op ----
        let s2 = engine.rethread_account(&account).await.unwrap();
        assert_eq!(s2.messages, 5);
        assert_eq!(s2.threads, 2);
        assert_eq!(s2.reassigned, 0, "re-run reassigns nothing");
        assert_eq!(tid(&o).await, to);
        assert_eq!(tid(&a).await, ta);
        assert_eq!(tid(&b).await, tb);
        assert_eq!(tid(&c).await, tc);
        assert_eq!(tid(&other).await, tother);
    }

    #[test]
    fn summary_merges() {
        let mut acc = RethreadSummary::default();
        acc.merge(&RethreadSummary {
            accounts: 1,
            messages: 3,
            threads: 2,
            reassigned: 3,
        });
        acc.merge(&RethreadSummary {
            accounts: 1,
            messages: 4,
            threads: 1,
            reassigned: 0,
        });
        assert_eq!(
            acc,
            RethreadSummary {
                accounts: 2,
                messages: 7,
                threads: 3,
                reassigned: 3,
            }
        );
    }
}
