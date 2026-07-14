//! V7 engine-wiring integration tests (plan §3 e8 acceptance):
//! - a mock **plugin backend** serves mailboxes/messages through the engine
//!   *indistinguishably from `mw-imap`* (same `handle_jmap` surface);
//! - a **`SyncCursor::Plugin`** bridge cursor round-trips through the engine's
//!   persistence (the backend gets its own native token back on the next sync);
//! - the **GAL directory** resolves entries in recipient lookup + group-expand;
//! - a bridge-advertised **reactions** cap routes to the bridge while a
//!   non-advertising account falls back (`None`);
//! - the **Assist hook** gates enablement;
//! - the **non-plugin / non-directory default path is byte-unchanged** (every V7
//!   accessor is inert until something is attached).

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, Flag, MailboxDelta, MailboxRole, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::v7::{
    AssistHook, BridgeCapabilitySource, BridgeCaps, BridgeFocusedSync, BridgeReaction,
    BridgeReactions, BridgeRecall, BridgeVoting, DirectorySource, GalEntry, V7Hooks,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 42;

// ── A mock PLUGIN/bridge account backend ─────────────────────────────────────────
//
// It is nothing more than an `Arc<dyn AccountBackend>` — exactly what
// `mw_plugin::PluginHandle::as_account_backend()` returns — and it emits a NATIVE
// `SyncCursor::Plugin` token (a stand-in for a Graph deltaLink). It records the
// cursor it was handed so the test can prove the engine round-tripped it.

struct PluginBackend {
    /// The native token this bridge returns as its next cursor.
    native_token: Vec<u8>,
    /// The cursor the engine handed us on the most recent `sync_mailbox`.
    last_cursor: Mutex<Option<SyncCursor>>,
    served: Mutex<bool>,
}

impl PluginBackend {
    fn new() -> Self {
        Self {
            native_token: b"deltatoken=BRIDGE-XYZ\x00\xff".to_vec(),
            last_cursor: Mutex::new(None),
            served: Mutex::new(false),
        }
    }
}

fn plugin_msg() -> Vec<u8> {
    concat!(
        "Message-ID: <bridge-1@graph>\r\n",
        "From: Priya <priya@contoso.com>\r\n",
        "To: me@contoso.com\r\n",
        "Subject: Bridged hello\r\n",
        "Date: Mon, 14 Jul 2026 09:00:00 +0000\r\n",
        "\r\n",
        "sent through the bridge\r\n",
    )
    .as_bytes()
    .to_vec()
}

#[async_trait]
impl AccountBackend for PluginBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: UIDVALIDITY,
            },
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: 2,
            highestmodseq: 0,
            total: 1,
            unread: 1,
        }])
    }

    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        *self.last_cursor.lock().unwrap() = Some(cursor.clone());
        // Only serve the message on the FIRST sync (fresh cursor); on a resync with
        // our own Plugin cursor we return no new messages (idempotent).
        let already = *self.served.lock().unwrap();
        let added = if already {
            Vec::new()
        } else {
            *self.served.lock().unwrap() = true;
            vec![MessageRef::Imap {
                mailbox: mbox.clone(),
                uidvalidity: UIDVALIDITY,
                uid: 1,
            }]
        };
        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed: Vec::new(),
            // The bridge returns its NATIVE token, opaque to the engine.
            next_cursor: SyncCursor::Plugin {
                opaque: self.native_token.clone(),
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Ok(refs
            .iter()
            .map(|r| RawMessage {
                message_ref: r.clone(),
                raw: plugin_msg(),
                flags: Vec::new(),
                internaldate: Some("2026-07-14T09:00:00Z".into()),
            })
            .collect())
    }

    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _d: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Ok(MoveOutcome::RederiveByMessageId)
    }

    async fn append(&self, mbox: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid: 2,
        })
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

// ── A trivial submitter ──────────────────────────────────────────────────────────

struct NoopSubmitter;
#[async_trait]
impl MailSubmitter for NoopSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

// ── A mock GAL directory ─────────────────────────────────────────────────────────

struct MockDirectory;
#[async_trait]
impl DirectorySource for MockDirectory {
    async fn search_gal(&self, query: &str, _page: u32) -> mw_directory::Result<Vec<GalEntry>> {
        Ok(vec![
            GalEntry {
                dn: "cn=Priya,ou=people,dc=contoso,dc=com".into(),
                display_name: format!("Priya (match {query})"),
                mail: "priya@contoso.com".into(),
                is_group: false,
            },
            GalEntry {
                dn: "cn=Sales,ou=groups,dc=contoso,dc=com".into(),
                display_name: "Sales Team".into(),
                mail: "sales@contoso.com".into(),
                is_group: true,
            },
        ])
    }

    async fn expand_group(&self, _dn: &str) -> mw_directory::Result<Vec<GalEntry>> {
        Ok(vec![GalEntry {
            dn: "cn=Priya,ou=people,dc=contoso,dc=com".into(),
            display_name: "Priya".into(),
            mail: "priya@contoso.com".into(),
            is_group: false,
        }])
    }

    async fn lookup_cert(&self, _email: &str) -> mw_directory::Result<Vec<mw_directory::Der>> {
        Ok(vec![vec![0x30, 0x82, 0x01, 0x02]]) // a stand-in DER prefix
    }

    async fn lookup_photo(&self, _email: &str) -> mw_directory::Result<Option<mw_directory::Der>> {
        Ok(None)
    }

    async fn bind_auth(
        &self,
        _user: &str,
        _pass: &str,
    ) -> mw_directory::Result<mw_directory::BindOutcome> {
        Ok(mw_directory::BindOutcome::Denied)
    }
}

// ── A mock bridge-capability source: reactions ONLY for the bridge account ───────

struct MockReactions;
#[async_trait]
impl BridgeReactions for MockReactions {
    async fn set_reaction(&self, _m: &MessageRef, _e: &str, _add: bool) -> Result<()> {
        Ok(())
    }
    async fn get_reactions(&self, _m: &MessageRef) -> Result<Vec<BridgeReaction>> {
        Ok(vec![BridgeReaction {
            actor: "priya@contoso.com".into(),
            emoji: "👍".into(),
        }])
    }
}

struct MockBridgeCaps {
    /// The one account that is bridge-backed and advertises reactions.
    bridge_account: String,
}
impl BridgeCapabilitySource for MockBridgeCaps {
    fn caps(&self, account_id: &str) -> BridgeCaps {
        if account_id == self.bridge_account {
            BridgeCaps {
                reactions: true,
                ..BridgeCaps::default()
            }
        } else {
            BridgeCaps::default()
        }
    }
    fn reactions(&self, account_id: &str) -> Option<Arc<dyn BridgeReactions>> {
        (account_id == self.bridge_account)
            .then(|| Arc::new(MockReactions) as Arc<dyn BridgeReactions>)
    }
    fn voting(&self, _a: &str) -> Option<Arc<dyn BridgeVoting>> {
        None
    }
    fn recall(&self, _a: &str) -> Option<Arc<dyn BridgeRecall>> {
        None
    }
    fn focused_sync(&self, _a: &str) -> Option<Arc<dyn BridgeFocusedSync>> {
        None
    }
}

// ── A mock Assist hook ───────────────────────────────────────────────────────────

struct MockAssist {
    enabled: bool,
}
impl AssistHook for MockAssist {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    fn granted_capabilities(&self) -> Vec<String> {
        vec!["summarize".into(), "grammar".into()]
    }
}

// ── Harness ──────────────────────────────────────────────────────────────────────

async fn engine_with_account() -> (Arc<Engine>, String, Arc<PluginBackend>) {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "graph.microsoft.com",
                port: 443,
                tls: "implicit",
                username: "me@contoso.com",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@contoso.com".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let engine = Arc::new(Engine::new(store));
    let backend = Arc::new(PluginBackend::new());
    engine.register_plugin_backend(
        account_id.clone(),
        "bridge-graph",
        AccountRuntime::new(
            backend.clone() as Arc<dyn AccountBackend>,
            Arc::new(NoopSubmitter) as Arc<dyn MailSubmitter>,
            "me@contoso.com",
        ),
    );
    (engine, account_id, backend)
}

async fn jmap(engine: &Engine, account_id: &str, calls: Value) -> Value {
    engine
        .handle_jmap(account_id, &json!({ "methodCalls": calls }))
        .await
}

// ── Tests ────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn plugin_backend_serves_through_engine_like_imap() {
    let (engine, account_id, _backend) = engine_with_account().await;
    assert!(engine.is_plugin_backed(&account_id));
    assert_eq!(
        engine.plugin_backend_id(&account_id).as_deref(),
        Some("bridge-graph")
    );

    engine.resync(&account_id).await.unwrap();

    // The JMAP surface is served identically to an IMAP account: one INBOX with the
    // bridged message.
    let mb = jmap(&engine, &account_id, json!([["Mailbox/get", {}, "mb"]])).await;
    let mailboxes = &mb["methodResponses"][0][1]["list"];
    let inbox = mailboxes
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "inbox")
        .expect("inbox present");
    let inbox_id = inbox["id"].as_str().unwrap();

    let q = jmap(
        &engine,
        &account_id,
        json!([["Email/query", { "filter": { "inMailbox": inbox_id } }, "q"]]),
    )
    .await;
    let ids = q["methodResponses"][0][1]["ids"].as_array().unwrap();
    assert_eq!(
        ids.len(),
        1,
        "the bridged message is served through the engine"
    );
}

#[tokio::test]
async fn plugin_sync_cursor_round_trips() {
    let (engine, account_id, backend) = engine_with_account().await;

    // First sync: the backend sees the engine's initial UID-window cursor and
    // returns its native Plugin token.
    engine.resync(&account_id).await.unwrap();

    // Second sync: the engine must have persisted + reloaded the Plugin cursor and
    // handed the SAME native bytes back to the backend (lossless round-trip).
    engine.resync(&account_id).await.unwrap();
    let seen = backend.last_cursor.lock().unwrap().clone();
    match seen {
        Some(SyncCursor::Plugin { opaque }) => {
            assert_eq!(
                opaque, backend.native_token,
                "native bridge token survived persistence"
            );
        }
        other => panic!("expected the engine to re-hand a Plugin cursor, got {other:?}"),
    }
}

#[tokio::test]
async fn gal_resolves_in_recipient_lookup() {
    let (engine, _account_id, _backend) = engine_with_account().await;

    // Default path (no directory attached): GAL lookups are inert/empty.
    assert!(!engine.directory_attached());
    assert!(
        engine
            .resolve_recipients("pri", 0)
            .await
            .unwrap()
            .is_empty()
    );

    // Attach the GAL source (as e14 does at mount).
    engine.attach_v7(V7Hooks::new().with_directory(Arc::new(MockDirectory)));
    assert!(engine.directory_attached());

    let hits = engine.resolve_recipients("pri", 0).await.unwrap();
    assert_eq!(hits.len(), 2, "GAL entries resolve in recipient lookup");
    assert!(hits.iter().any(|g| g.mail == "priya@contoso.com"));
    assert!(
        hits.iter()
            .any(|g| g.is_group && g.mail == "sales@contoso.com")
    );

    // Expand-before-send resolves the group's members.
    let members = engine
        .expand_group("cn=Sales,ou=groups,dc=contoso,dc=com")
        .await
        .unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].mail, "priya@contoso.com");

    // S/MIME cert lookup feeds mw-crypto.
    let certs = engine.gal_lookup_cert("priya@contoso.com").await.unwrap();
    assert_eq!(certs.len(), 1);
}

#[tokio::test]
async fn bridge_reactions_prefer_bridge_else_fallback() {
    let (engine, bridge_account, _backend) = engine_with_account().await;

    // Nothing attached ⇒ every account uses the fallback (None).
    assert!(engine.bridge_reactions(&bridge_account).is_none());
    assert!(!engine.bridge_caps(&bridge_account).reactions);

    engine.attach_v7(V7Hooks::new().with_bridge_caps(Arc::new(MockBridgeCaps {
        bridge_account: bridge_account.clone(),
    })));

    // The bridge account advertises reactions ⇒ routes to the bridge impl.
    assert!(engine.bridge_caps(&bridge_account).reactions);
    let reactions = engine
        .bridge_reactions(&bridge_account)
        .expect("advertised ⇒ routes to the bridge");
    let msg = MessageRef::Imap {
        mailbox: RawMailboxRef {
            name: "INBOX".into(),
            uidvalidity: UIDVALIDITY,
        },
        uidvalidity: UIDVALIDITY,
        uid: 1,
    };
    assert_eq!(reactions.get_reactions(&msg).await.unwrap().len(), 1);

    // A different (non-advertising) account ⇒ None ⇒ the existing standards
    // fallback, byte-unchanged.
    assert!(engine.bridge_reactions("some-other-imap-account").is_none());
    assert!(engine.bridge_voting(&bridge_account).is_none());
    assert!(engine.bridge_recall(&bridge_account).is_none());
    assert!(engine.bridge_focused_sync(&bridge_account).is_none());
}

#[tokio::test]
async fn assist_hook_gates_enablement() {
    let (engine, _account_id, _backend) = engine_with_account().await;

    // Unconfigured ⇒ no hook ⇒ the web hides all Assist UI.
    assert!(engine.assist().is_none());
    assert!(!engine.assist_enabled());

    engine.attach_v7(V7Hooks::new().with_assist(Arc::new(MockAssist { enabled: true })));
    assert!(engine.assist_enabled());
    assert_eq!(
        engine.assist().unwrap().granted_capabilities(),
        vec!["summarize".to_string(), "grammar".to_string()]
    );

    // A disabled hook keeps the UI hidden.
    engine.attach_v7(V7Hooks::new().with_assist(Arc::new(MockAssist { enabled: false })));
    assert!(!engine.assist_enabled());
}

#[tokio::test]
async fn attach_v7_preserves_plugin_backings() {
    // Registering a plugin backend then attaching other V7 seams must NOT drop the
    // plugin-backed bookkeeping (attach preserves the registry).
    let (engine, account_id, _backend) = engine_with_account().await;
    assert!(engine.is_plugin_backed(&account_id));
    engine.attach_v7(V7Hooks::new().with_directory(Arc::new(MockDirectory)));
    assert!(
        engine.is_plugin_backed(&account_id),
        "attach_v7 preserved the plugin backing"
    );
    engine.attach_v7(V7Hooks::new().with_assist(Arc::new(MockAssist { enabled: true })));
    assert!(engine.is_plugin_backed(&account_id));
}

#[tokio::test]
async fn default_path_is_inert() {
    // With no V7 attach + no plugin registration, every V7 accessor is inert — the
    // non-plugin / non-directory default path is byte-unchanged.
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let engine = Engine::new(store);
    assert!(!engine.directory_attached());
    assert!(engine.assist().is_none());
    assert!(!engine.assist_enabled());
    assert!(!engine.is_plugin_backed("nope"));
    assert!(engine.plugin_backend_id("nope").is_none());
    assert!(engine.resolve_recipients("x", 0).await.unwrap().is_empty());
    assert!(engine.expand_group("x").await.unwrap().is_empty());
    assert!(engine.gal_lookup_cert("x@y").await.unwrap().is_empty());
    assert!(engine.bridge_reactions("x").is_none());
    assert_eq!(engine.bridge_caps("x"), BridgeCaps::default());
}
