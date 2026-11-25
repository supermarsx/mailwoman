//! t62-n6a: the outbox transitions on SQLite, and on PostgreSQL when
//! `MW_TEST_PG` names a server (each test in its own schema).

use super::*;
use crate::{AccountKind, Credentials, MailboxUpsert, NewAccount, ServerKey};

struct Fixture {
    store: Store,
    pg: Option<(sqlx::PgConnection, String)>,
}

impl Fixture {
    async fn done(self) {
        self.store.close().await;
        if let Some((mut admin, schema)) = self.pg {
            sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&mut admin).await.unwrap();
        }
    }
}

/// SQLite always; PostgreSQL too when `MW_TEST_PG` is set.
async fn fixtures() -> Vec<Fixture> {
    let mut out = vec![Fixture { store: Store::open_in_memory(ServerKey::generate()).await.unwrap(), pg: None }];
    if let Some(dsn) = std::env::var("MW_TEST_PG").ok().filter(|s| !s.is_empty()) {
        use sqlx::Connection;
        let schema = format!("upstream_outbox_{}", uuid::Uuid::new_v4().simple());
        let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
        sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&mut admin).await.unwrap();
        let mut scoped = url::Url::parse(&dsn).unwrap();
        scoped.query_pairs_mut().append_pair("options", &format!("-csearch_path={schema}"));
        let store = Store::open(scoped.as_str(), ServerKey::generate()).await.unwrap();
        out.push(Fixture { store, pg: Some((admin, schema)) });
    }
    out
}

async fn account(s: &Store) -> String {
    s.create_account(
        &NewAccount {
            kind: AccountKind::Imap,
            host: "imap.example.org",
            port: 993,
            tls: "implicit",
            username: &format!("outbox-{}", crate::seal::random_token()),
            sync_policy_json: r#"{"keep":true}"#,
        },
        &Credentials { username: "u".into(), password: "p".into() },
    )
    .await
    .unwrap()
}

async fn mailbox(s: &Store, account: &str, name: &str, uidvalidity: u32) -> String {
    s.upsert_mailbox(&MailboxUpsert {
        account_id: account,
        name,
        role: None,
        uidvalidity,
        uidnext: 1,
        highestmodseq: 0,
        total: 0,
        unread: 0,
        parent_id: None,
    })
    .await
    .unwrap()
}

async fn message(s: &Store, account: &str, mailbox: &str, uid: u32, uidvalidity: u32, flags: &str) -> String {
    let message_id = format!("<{}@fixture>", crate::seal::random_token());
    s.upsert_message(&MessageUpsert {
        account_id: account,
        mailbox_id: mailbox,
        uid,
        uidvalidity,
        message_id: Some(&message_id),
        thread_id: None,
        internaldate: Some("2026-11-01T10:00:00Z"),
        size: 100,
        flags_json: flags,
        envelope: None,
        blob_ref: None,
    })
    .await
    .unwrap()
}

async fn intents(s: &Store, account: &str) -> Vec<OutboxIntent> {
    s.upstream_intents(account).await.unwrap()
}

async fn backlog_matches(s: &Store, account: &str) {
    let rows = intents(s, account).await.len() as i64;
    let head = s.upstream_outbox_head(account).await.unwrap().map(|h| h.backlog).unwrap_or(0);
    assert_eq!(head, rows, "head backlog equals the row count");
}

async fn email_changes(s: &Store, account: &str) -> i64 {
    q("SELECT COUNT(*) FROM changes WHERE account_id=?1 AND type='Email'").bind(account).fetch_scalar_i64(&s.backend).await.unwrap()
}

fn seen_field() -> String {
    flag_field("\"Seen\"")
}

fn set(id: &str, flags: &str) -> Vec<(String, String)> {
    vec![(id.to_string(), flags.to_string())]
}

#[test]
fn plan_value_follows_the_coalescing_table() {
    let row = |state: IntentState, want: &str, base: &str, shadow: Option<&str>| OutboxIntent {
        account_id: "a".into(),
        stable_id: "m".into(),
        field: seen_field(),
        seq: 1,
        first_at: String::new(),
        updated_at: String::new(),
        want: Some(want.into()),
        sent: None,
        base: Some(base.into()),
        shadow: shadow.map(Into::into),
        base_location: None,
        state,
        claim: None,
        attempts: 0,
        next_at: None,
        error_class: None,
        has_error_detail: false,
        origin: IntentOrigin::User,
        upstream_gen: 0,
    };
    assert_eq!(plan_value(None, "1", "0"), ValuePlan::Insert);
    assert_eq!(plan_value(None, "1", "1"), ValuePlan::Nothing);
    let pending = row(IntentState::Pending, "1", "0", None);
    assert_eq!(plan_value(Some(&pending), "0", "1"), ValuePlan::Cancel, "set then clear cancels");
    assert_eq!(plan_value(Some(&pending), "1", "1"), ValuePlan::Nothing, "same value twice: one row");
    let shadowed = row(IntentState::Pending, "1", "0", Some("1"));
    assert_eq!(plan_value(Some(&shadowed), "1", "1"), ValuePlan::Cancel, "want equals shadow: cancel");
    assert_eq!(plan_value(Some(&shadowed), "0", "1"), ValuePlan::Overwrite, "shadow wins over base");
    let inflight = row(IntentState::Inflight, "1", "0", None);
    assert_eq!(plan_value(Some(&inflight), "0", "1"), ValuePlan::OverwriteInFlight);
    assert_eq!(with_flag(r#"["Seen",{"Keyword":"$x"}]"#, "\"Seen\"", false).unwrap(), r#"[{"Keyword":"$x"}]"#);
    assert_eq!(with_flag("[]", r#"{"Keyword":"$x"}"#, true).unwrap(), r#"[{"Keyword":"$x"}]"#);
}

#[tokio::test]
async fn flag_intents_insert_overwrite_cancel_and_keep_counters_and_change_rows() {
    for f in fixtures().await {
        let s = &f.store;
        let a = account(s).await;
        let inbox = mailbox(s, &a, "INBOX", 7).await;
        let m = message(s, &a, &inbox, 5, 7, "[]").await;
        let local = message(s, &a, &inbox, 9, 0, "[]").await;
        let other_account = account(s).await;
        let other_box = mailbox(s, &other_account, "INBOX", 7).await;
        let foreign = message(s, &other_account, &other_box, 1, 7, "[]").await;
        let changes0 = email_changes(s, &a).await;

        // Mark read: one intent (base '0', want '1'), unread counter down, one change row.
        let w = s.set_flags_with_intents(&a, &[(m.clone(), r#"["Seen"]"#.into()), (local.clone(), r#"["Seen"]"#.into()), (foreign.clone(), r#"["Seen"]"#.into())], IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert_eq!(w.written, vec![true, true, false], "a foreign id is not written");
        assert_eq!(w.tally, IntentTally { created: 1, overwritten: 0, cancelled: 0 }, "the engine-local row has no server coordinates");
        assert!(w.email_state.is_some());
        assert_eq!(email_changes(s, &a).await, changes0 + 2, "both written ids recorded in the same transaction");
        assert_eq!(s.get_message(&foreign).await.unwrap().flags_json, "[]");
        let rows = intents(s, &a).await;
        assert_eq!(rows.len(), 1);
        assert_eq!((rows[0].field.as_str(), rows[0].want.as_deref(), rows[0].base.as_deref(), rows[0].state), (seen_field().as_str(), Some("1"), Some("0"), IntentState::Pending));
        assert_eq!(s.get_mailbox(&inbox).await.unwrap().unread, 0);
        backlog_matches(s, &a).await;

        // Same value again: nothing changes in the outbox.
        let w = s.set_flags_with_intents(&a, &set(&m, r#"["Seen"]"#), IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert_eq!(w.tally, IntentTally::default());

        // A second flag: a second row; then clear Seen: that row cancels.
        let w = s.set_flags_with_intents(&a, &set(&m, r#"["Seen","Flagged"]"#), IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert_eq!(w.tally.created, 1);
        let w = s.set_flags_with_intents(&a, &set(&m, r#"["Flagged"]"#), IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert_eq!(w.tally, IntentTally { created: 0, overwritten: 0, cancelled: 1 }, "set then clear before send cancels");
        let rows = intents(s, &a).await;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].field, flag_field("\"Flagged\""));
        assert_eq!(s.get_mailbox(&inbox).await.unwrap().unread, 1, "local value is unread again");
        backlog_matches(s, &a).await;
        f.done().await;
    }
}

#[tokio::test]
async fn a_flag_changed_while_in_flight_is_requeued_with_base_sent() {
    for f in fixtures().await {
        let s = &f.store;
        let a = account(s).await;
        let inbox = mailbox(s, &a, "INBOX", 7).await;
        let m = message(s, &a, &inbox, 5, 7, "[]").await;
        let field = seen_field();
        s.set_flags_with_intents(&a, &set(&m, r#"["Seen"]"#), IntentOrigin::User, 100).await.unwrap();
        let due = s.due_upstream_intents(&a, "9999", 10).await.unwrap();
        assert_eq!(due.len(), 1);
        let keys = vec![(m.clone(), field.clone(), due[0].seq)];
        let claim = s.claim_upstream_intents(&a, &keys, "t1").await.unwrap();
        assert_eq!(claim.claimed.len(), 1);
        assert_eq!(claim.claimed[0].intent.sent.as_deref(), Some("1"));
        assert_eq!(s.claim_upstream_intents(&a, &keys, "t2").await.unwrap().claimed.len(), 0, "claimed once");

        // The user clears it while the STORE is on the wire.
        let w = s.set_flags_with_intents(&a, &set(&m, "[]"), IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert_eq!(w.tally.overwritten, 1);
        assert_eq!(s.confirm_upstream_intents(&a, "wrong", &[(m.clone(), field.clone(), Confirmation::Applied)]).await.unwrap(), vec![Confirmed::Stale]);
        assert_eq!(s.confirm_upstream_intents(&a, "t1", &[(m.clone(), field.clone(), Confirmation::Applied)]).await.unwrap(), vec![Confirmed::Requeued]);
        let rows = intents(s, &a).await;
        assert_eq!((rows[0].state, rows[0].base.as_deref(), rows[0].want.as_deref(), rows[0].sent.as_deref()), (IntentState::Pending, Some("1"), Some("0"), None));
        // Send the clear; acknowledged: gone.
        let claim = s.claim_upstream_intents(&a, &[(m.clone(), field.clone(), rows[0].seq)], "t3").await.unwrap();
        assert_eq!(claim.claimed.len(), 1);
        assert_eq!(s.confirm_upstream_intents(&a, "t3", &[(m.clone(), field.clone(), Confirmation::Applied)]).await.unwrap(), vec![Confirmed::Done]);
        assert!(intents(s, &a).await.is_empty());
        backlog_matches(s, &a).await;

        // Changed and changed back while in flight: want == sent, done at ack.
        s.set_flags_with_intents(&a, &set(&m, r#"["Seen"]"#), IntentOrigin::User, 100).await.unwrap();
        let seq = intents(s, &a).await[0].seq;
        s.claim_upstream_intents(&a, &[(m.clone(), field.clone(), seq)], "t4").await.unwrap();
        s.set_flags_with_intents(&a, &set(&m, "[]"), IntentOrigin::User, 100).await.unwrap();
        s.set_flags_with_intents(&a, &set(&m, r#"["Seen"]"#), IntentOrigin::User, 100).await.unwrap();
        assert_eq!(s.confirm_upstream_intents(&a, "t4", &[(m.clone(), field.clone(), Confirmation::Applied)]).await.unwrap(), vec![Confirmed::Done]);
        assert!(intents(s, &a).await.is_empty());
        backlog_matches(s, &a).await;
        f.done().await;
    }
}

#[tokio::test]
async fn want_equal_to_shadow_is_cancelled_at_claim_and_pause_blocks_claims() {
    for f in fixtures().await {
        let s = &f.store;
        let a = account(s).await;
        let inbox = mailbox(s, &a, "INBOX", 7).await;
        let m = message(s, &a, &inbox, 5, 7, "[]").await;
        let field = seen_field();
        s.set_flags_with_intents(&a, &set(&m, r#"["Seen"]"#), IntentOrigin::Rule, 100).await.unwrap();
        let seq = intents(s, &a).await[0].seq;
        s.set_upstream_paused(&a, Some("auth")).await.unwrap();
        let claim = s.claim_upstream_intents(&a, &[(m.clone(), field.clone(), seq)], "t").await.unwrap();
        assert_eq!(claim.paused.as_deref(), Some("auth"));
        assert!(claim.claimed.is_empty());
        assert!(s.set_upstream_paused(&a, Some("bogus")).await.is_err());
        s.set_upstream_paused(&a, None).await.unwrap();
        // Another client set it too: sync wrote the shadow.
        assert_eq!(s.set_upstream_shadow(&a, &[(m.clone(), field.clone(), Some("1".into()))]).await.unwrap(), 1);
        let claim = s.claim_upstream_intents(&a, &[(m.clone(), field.clone(), seq)], "t").await.unwrap();
        assert_eq!(claim.cancelled, vec![(m.clone(), field.clone())]);
        assert!(intents(s, &a).await.is_empty());
        backlog_matches(s, &a).await;
        f.done().await;
    }
}

#[tokio::test]
async fn moves_insert_overwrite_and_cancel_back_to_the_base() {
    for f in fixtures().await {
        let s = &f.store;
        let a = account(s).await;
        let inbox = mailbox(s, &a, "INBOX", 7).await;
        let archive = mailbox(s, &a, "Archive", 8).await;
        let trash = mailbox(s, &a, "Trash", 9).await;
        let m = message(s, &a, &inbox, 5, 7, "[]").await;
        s.update_mailbox_counts(&inbox, 6, 0, 1, 1).await.ok();
        let changes0 = email_changes(s, &a).await;

        let msg = s.get_message(&m).await.unwrap();
        let r = s.relocate_with_intent(&msg, &archive, IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert!(r.moved);
        assert_eq!(r.uidvalidity, 0, "a moved row has no server coordinates");
        assert_eq!(r.tally.created, 1);
        assert_eq!(email_changes(s, &a).await, changes0 + 1);
        let row = s.get_message(&m).await.unwrap();
        assert_eq!((row.mailbox_id.as_str(), row.uidvalidity, row.uid), (archive.as_str(), 0, r.uid));
        let rows = intents(s, &a).await;
        assert_eq!(rows[0].field, FIELD_LOCATION);
        assert_eq!(rows[0].want.as_deref(), Some(archive.as_str()));
        assert_eq!(rows[0].base_location, Some(ServerCoords { mailbox_id: inbox.clone(), uidvalidity: 7, uid: 5 }));
        assert_eq!((s.get_mailbox(&inbox).await.unwrap().total, s.get_mailbox(&archive).await.unwrap().total), (0, 1));

        // A flag change on the moved message still gets an intent (base coordinates).
        let w = s.set_flags_with_intents(&a, &set(&m, r#"["Seen"]"#), IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert_eq!(w.tally.created, 1);

        // A -> B -> C: one row, want C, base A.
        let msg = s.get_message(&m).await.unwrap();
        let r = s.relocate_with_intent(&msg, &trash, IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert_eq!(r.tally.overwritten, 1);
        let location = intents(s, &a).await.into_iter().find(|i| i.field == FIELD_LOCATION).unwrap();
        assert_eq!(location.want.as_deref(), Some(trash.as_str()));
        assert_eq!(location.base_location.as_ref().unwrap().mailbox_id, inbox);

        // Stale source: nothing written.
        assert!(!s.relocate_with_intent(&msg, &inbox, IntentOrigin::User, 100).await.unwrap().done().unwrap().moved);

        // -> A: cancelled, the base coordinates come back.
        let msg = s.get_message(&m).await.unwrap();
        let r = s.relocate_with_intent(&msg, &inbox, IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert_eq!(r.tally.cancelled, 1);
        let row = s.get_message(&m).await.unwrap();
        assert_eq!((row.mailbox_id.as_str(), row.uidvalidity, row.uid), (inbox.as_str(), 7, 5));
        assert!(intents(s, &a).await.iter().all(|i| i.field != FIELD_LOCATION));
        assert_eq!((s.get_mailbox(&inbox).await.unwrap().total, s.get_mailbox(&trash).await.unwrap().total), (1, 0));
        backlog_matches(s, &a).await;
        f.done().await;
    }
}

#[tokio::test]
async fn a_move_in_flight_then_moved_back_gets_a_new_base_and_a_confirmed_move_takes_its_coordinates() {
    for f in fixtures().await {
        let s = &f.store;
        let a = account(s).await;
        let inbox = mailbox(s, &a, "INBOX", 7).await;
        let archive = mailbox(s, &a, "Archive", 8).await;
        let m = message(s, &a, &inbox, 5, 7, "[]").await;
        let msg = s.get_message(&m).await.unwrap();
        s.relocate_with_intent(&msg, &archive, IntentOrigin::User, 100).await.unwrap();
        let seq = intents(s, &a).await[0].seq;
        s.claim_upstream_intents(&a, &[(m.clone(), FIELD_LOCATION.into(), seq)], "t1").await.unwrap();
        // Back to INBOX while the MOVE is on the wire: want only.
        let msg = s.get_message(&m).await.unwrap();
        let r = s.relocate_with_intent(&msg, &inbox, IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert_eq!(r.tally.overwritten, 1);
        assert_eq!(r.uidvalidity, 0, "not cancelled while in flight");
        let out = s.confirm_upstream_intents(&a, "t1", &[(m.clone(), FIELD_LOCATION.into(), Confirmation::Moved { uidvalidity: 8, uid: 40 })]).await.unwrap();
        assert_eq!(out, vec![Confirmed::Requeued]);
        let l = intents(s, &a).await.remove(0);
        assert_eq!(l.state, IntentState::Pending);
        assert_eq!(l.want.as_deref(), Some(inbox.as_str()));
        assert_eq!(l.base_location, Some(ServerCoords { mailbox_id: archive.clone(), uidvalidity: 8, uid: 40 }));
        // The move back is sent and acknowledged: the row takes the new coordinates.
        s.claim_upstream_intents(&a, &[(m.clone(), FIELD_LOCATION.into(), l.seq)], "t2").await.unwrap();
        let out = s.confirm_upstream_intents(&a, "t2", &[(m.clone(), FIELD_LOCATION.into(), Confirmation::Moved { uidvalidity: 7, uid: 77 })]).await.unwrap();
        assert_eq!(out, vec![Confirmed::Done]);
        let row = s.get_message(&m).await.unwrap();
        assert_eq!((row.mailbox_id.as_str(), row.uidvalidity, row.uid), (inbox.as_str(), 7, 77));
        assert!(intents(s, &a).await.is_empty());
        backlog_matches(s, &a).await;

        // Collision: another row already holds the reported coordinates.
        let other = message(s, &a, &archive, 41, 8, "[]").await;
        let _ = other;
        let msg = s.get_message(&m).await.unwrap();
        s.relocate_with_intent(&msg, &archive, IntentOrigin::User, 100).await.unwrap();
        let seq = intents(s, &a).await[0].seq;
        s.claim_upstream_intents(&a, &[(m.clone(), FIELD_LOCATION.into(), seq)], "t3").await.unwrap();
        let out = s.confirm_upstream_intents(&a, "t3", &[(m.clone(), FIELD_LOCATION.into(), Confirmation::Moved { uidvalidity: 8, uid: 41 })]).await.unwrap();
        assert_eq!(out, vec![Confirmed::Collision]);
        let l = intents(s, &a).await.remove(0);
        assert_eq!((l.state, l.error_class.as_deref()), (IntentState::Unresolved, Some("collision")));
        backlog_matches(s, &a).await;
        f.done().await;
    }
}

#[tokio::test]
async fn reject_reverts_to_shadow_else_base_with_a_notice() {
    for f in fixtures().await {
        let s = &f.store;
        let a = account(s).await;
        let inbox = mailbox(s, &a, "INBOX", 7).await;
        let archive = mailbox(s, &a, "Archive", 8).await;
        let m = message(s, &a, &inbox, 5, 7, "[]").await;
        let field = seen_field();
        s.set_flags_with_intents(&a, &set(&m, r#"["Seen"]"#), IntentOrigin::User, 100).await.unwrap();
        let state0 = email_changes(s, &a).await;
        let Reverted::Done { email_state, notice_id } = s.reject_and_revert(&a, &m, &field, None, "noperm").await.unwrap() else { panic!("reverted") };
        assert!(email_state.is_some());
        assert_eq!(email_changes(s, &a).await, state0 + 1, "a new change-log state");
        assert_eq!(s.get_message(&m).await.unwrap().flags_json, "[]", "back to base");
        assert_eq!(s.get_mailbox(&inbox).await.unwrap().unread, 1);
        let notices = s.upstream_notices(&a, 10).await.unwrap();
        assert_eq!((notices[0].id.as_str(), notices[0].op.as_str(), notices[0].reason.as_str(), notices[0].stable_id.as_deref()), (notice_id.as_str(), "flag", "noperm", Some(m.as_str())));
        assert!(s.dismiss_upstream_notice(&a, &notice_id).await.unwrap());
        assert!(!s.dismiss_upstream_notice(&a, &notice_id).await.unwrap());
        assert_eq!(s.reject_and_revert(&a, &m, &field, None, "noperm").await.unwrap(), Reverted::Missing);

        // Shadow wins: another client set it meanwhile.
        s.set_flags_with_intents(&a, &set(&m, r#"["Seen"]"#), IntentOrigin::User, 100).await.unwrap();
        s.set_flags_with_intents(&a, &set(&m, "[]"), IntentOrigin::User, 100).await.unwrap();
        s.set_flags_with_intents(&a, &set(&m, r#"["Seen"]"#), IntentOrigin::User, 100).await.unwrap();
        s.set_upstream_shadow(&a, &[(m.clone(), field.clone(), Some("0".into()))]).await.unwrap();
        s.set_flags_with_intents(&a, &set(&m, r#"["Seen","Flagged"]"#), IntentOrigin::User, 100).await.unwrap();
        s.reject_and_revert(&a, &m, &flag_field("\"Flagged\""), None, "cannot").await.unwrap();
        s.reject_and_revert(&a, &m, &field, None, "cannot").await.unwrap();
        assert_eq!(s.get_message(&m).await.unwrap().flags_json, "[]");

        // A move goes back to its base coordinates.
        let msg = s.get_message(&m).await.unwrap();
        s.relocate_with_intent(&msg, &archive, IntentOrigin::User, 100).await.unwrap();
        let seq = intents(s, &a).await[0].seq;
        s.claim_upstream_intents(&a, &[(m.clone(), FIELD_LOCATION.into(), seq)], "t").await.unwrap();
        assert_eq!(s.reject_and_revert(&a, &m, FIELD_LOCATION, Some("other"), "nonexistent").await.unwrap(), Reverted::Stale);
        s.reject_and_revert(&a, &m, FIELD_LOCATION, Some("t"), "nonexistent").await.unwrap();
        let row = s.get_message(&m).await.unwrap();
        assert_eq!((row.mailbox_id.as_str(), row.uidvalidity, row.uid), (inbox.as_str(), 7, 5));
        assert_eq!((s.get_mailbox(&inbox).await.unwrap().total, s.get_mailbox(&archive).await.unwrap().total), (1, 0));

        // ... unless sync saw it vanish: then the row goes.
        let msg = s.get_message(&m).await.unwrap();
        s.relocate_with_intent(&msg, &archive, IntentOrigin::User, 100).await.unwrap();
        s.set_upstream_shadow(&a, &[(m.clone(), FIELD_LOCATION.into(), Some(SHADOW_GONE.into()))]).await.unwrap();
        s.reject_and_revert(&a, &m, FIELD_LOCATION, None, "superseded").await.unwrap();
        assert!(matches!(s.get_message(&m).await, Err(StoreError::NotFound)));
        assert_eq!(s.get_mailbox(&archive).await.unwrap().total, 0);
        assert!(intents(s, &a).await.is_empty());
        backlog_matches(s, &a).await;
        f.done().await;
    }
}

#[tokio::test]
async fn drafts_append_coalesce_and_discard() {
    for f in fixtures().await {
        let s = &f.store;
        let a = account(s).await;
        let drafts = mailbox(s, &a, "Drafts", 3).await;
        let draft = |n: u32| MessageUpsert {
            account_id: &a,
            mailbox_id: &drafts,
            uid: 1000 + n,
            uidvalidity: 0,
            message_id: None,
            thread_id: None,
            internaldate: Some("2026-11-01T10:00:00Z"),
            size: 5,
            flags_json: r#"["Draft"]"#,
            envelope: None,
            blob_ref: None,
        };
        let raw = b"Subject: x\r\n\r\nbody\r\n";
        // Created then discarded before the APPEND: nothing remains.
        let d = s.ingest_draft_with_append_intent(&draft(1), raw, IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert_eq!(d.tally.created, 1);
        assert_eq!(intents(s, &a).await[0].field, FIELD_APPEND);
        let msg = s.get_message(&d.stable_id).await.unwrap();
        let del = s.delete_draft_with_copy_intent(&msg, None, IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert!(del.deleted);
        assert_eq!(del.tally.cancelled, 1);
        assert!(intents(s, &a).await.is_empty(), "nothing is sent");
        backlog_matches(s, &a).await;

        // Discarded while the APPEND is in flight: the acknowledgement queues an expunge of the copy.
        let d = s.ingest_draft_with_append_intent(&draft(2), raw, IntentOrigin::User, 100).await.unwrap().done().unwrap();
        let seq = intents(s, &a).await[0].seq;
        s.claim_upstream_intents(&a, &[(d.stable_id.clone(), FIELD_APPEND.into(), seq)], "t").await.unwrap();
        let msg = s.get_message(&d.stable_id).await.unwrap();
        s.delete_draft_with_copy_intent(&msg, None, IntentOrigin::User, 100).await.unwrap();
        assert_eq!(intents(s, &a).await[0].want, None);
        let out = s.confirm_upstream_intents(&a, "t", &[(d.stable_id.clone(), FIELD_APPEND.into(), Confirmation::Appended { uidvalidity: 3, uid: 60 })]).await.unwrap();
        assert_eq!(out, vec![Confirmed::ExpungeQueued]);
        let rows = intents(s, &a).await;
        assert_eq!((rows.len(), rows[0].field.as_str()), (1, FIELD_EXPUNGE_COPY));
        assert_eq!(rows[0].base_location, Some(ServerCoords { mailbox_id: drafts.clone(), uidvalidity: 3, uid: 60 }));
        s.claim_upstream_intents(&a, &[(d.stable_id.clone(), FIELD_EXPUNGE_COPY.into(), rows[0].seq)], "u").await.unwrap();
        assert_eq!(s.confirm_upstream_intents(&a, "u", &[(d.stable_id.clone(), FIELD_EXPUNGE_COPY.into(), Confirmation::Applied)]).await.unwrap(), vec![Confirmed::Done]);
        assert!(intents(s, &a).await.is_empty());

        // An appended draft deleted later: its remembered copy becomes an expunge intent.
        let d = s.ingest_draft_with_append_intent(&draft(3), raw, IntentOrigin::User, 100).await.unwrap().done().unwrap();
        let seq = intents(s, &a).await[0].seq;
        s.claim_upstream_intents(&a, &[(d.stable_id.clone(), FIELD_APPEND.into(), seq)], "v").await.unwrap();
        assert_eq!(s.confirm_upstream_intents(&a, "v", &[(d.stable_id.clone(), FIELD_APPEND.into(), Confirmation::Appended { uidvalidity: 3, uid: 61 })]).await.unwrap(), vec![Confirmed::Done]);
        let msg = s.get_message(&d.stable_id).await.unwrap();
        let copy = ServerCoords { mailbox_id: drafts.clone(), uidvalidity: 3, uid: 61 };
        let del = s.delete_draft_with_copy_intent(&msg, Some(&copy), IntentOrigin::User, 100).await.unwrap().done().unwrap();
        assert_eq!(del.tally.created, 1);
        assert_eq!(intents(s, &a).await[0].base_location, Some(copy));
        assert!(s.ingest_draft_with_append_intent(&MessageUpsert { uidvalidity: 3, ..draft(4) }, raw, IntentOrigin::User, 100).await.is_err());
        backlog_matches(s, &a).await;
        f.done().await;
    }
}

#[tokio::test]
async fn the_backlog_cap_refuses_new_rows_and_writes_nothing() {
    for f in fixtures().await {
        let s = &f.store;
        let a = account(s).await;
        let inbox = mailbox(s, &a, "INBOX", 7).await;
        let archive = mailbox(s, &a, "Archive", 8).await;
        let m1 = message(s, &a, &inbox, 1, 7, "[]").await;
        let m2 = message(s, &a, &inbox, 2, 7, "[]").await;
        let m3 = message(s, &a, &inbox, 3, 7, "[]").await;
        s.set_flags_with_intents(&a, &[(m1.clone(), r#"["Seen"]"#.into()), (m2.clone(), r#"["Seen"]"#.into())], IntentOrigin::User, 2).await.unwrap().done().unwrap();
        let changes = email_changes(s, &a).await;
        let at_cap = s.set_flags_with_intents(&a, &set(&m3, r#"["Seen"]"#), IntentOrigin::User, 2).await.unwrap();
        assert_eq!(at_cap, OutboxWrite::AtCap { backlog: 2, cap: 2 });
        assert_eq!(s.get_message(&m3).await.unwrap().flags_json, "[]", "nothing written at the cap");
        assert_eq!(email_changes(s, &a).await, changes);
        let msg = s.get_message(&m3).await.unwrap();
        assert!(matches!(s.relocate_with_intent(&msg, &archive, IntentOrigin::User, 2).await.unwrap(), OutboxWrite::AtCap { .. }));
        assert_eq!(s.get_message(&m3).await.unwrap().mailbox_id, inbox);
        // Rows that only overwrite or cancel are always accepted.
        assert!(s.set_flags_with_intents(&a, &set(&m1, "[]"), IntentOrigin::User, 2).await.unwrap().done().is_some());
        // A change that cancels one row and adds one stays within the cap.
        let w = s.set_flags_with_intents(&a, &[(m2.clone(), "[]".into()), (m3.clone(), r#"["Seen"]"#.into())], IntentOrigin::User, 1).await.unwrap().done().unwrap();
        assert_eq!((w.tally.created, w.tally.cancelled), (1, 1));
        backlog_matches(s, &a).await;
        f.done().await;
    }
}

#[tokio::test]
async fn restart_recovery_retry_and_sealed_detail() {
    for f in fixtures().await {
        let s = &f.store;
        let a = account(s).await;
        let inbox = mailbox(s, &a, "INBOX", 7).await;
        let archive = mailbox(s, &a, "Archive", 8).await;
        let m = message(s, &a, &inbox, 5, 7, "[]").await;
        let n = message(s, &a, &inbox, 6, 7, "[]").await;
        s.set_flags_with_intents(&a, &set(&n, r#"["Seen"]"#), IntentOrigin::User, 100).await.unwrap();
        let msg = s.get_message(&m).await.unwrap();
        s.relocate_with_intent(&msg, &archive, IntentOrigin::User, 100).await.unwrap();
        let keys: Vec<(String, String, i64)> = intents(s, &a).await.into_iter().map(|i| (i.stable_id, i.field, i.seq)).collect();
        assert_eq!(s.claim_upstream_intents(&a, &keys, "t").await.unwrap().claimed.len(), 2);
        assert_eq!(s.recover_upstream_after_restart(&a).await.unwrap(), (1, 1));
        let rows = intents(s, &a).await;
        let flag = rows.iter().find(|i| i.field == seen_field()).unwrap();
        let location = rows.iter().find(|i| i.field == FIELD_LOCATION).unwrap();
        assert_eq!((flag.state, flag.sent.as_deref(), flag.claim.as_deref()), (IntentState::Pending, None, None));
        assert_eq!((location.state, location.sent.as_deref()), (IntentState::Unresolved, Some(archive.as_str())));
        // An unresolved row is claimable for recovery and keeps what was sent.
        let claim = s.claim_upstream_intents(&a, &[(m.clone(), FIELD_LOCATION.into(), location.seq)], "r").await.unwrap();
        assert_eq!(claim.claimed[0].was, IntentState::Unresolved);
        assert_eq!(claim.claimed[0].intent.sent.as_deref(), Some(archive.as_str()));
        // A transient failure: pending again later, the server text sealed.
        let detail = "NO [UNAVAILABLE] backend down for INBOX/Secret";
        assert_eq!(s.retry_upstream_intents(&a, "r", &[(m.clone(), FIELD_LOCATION.into())], "transient", Some(detail), "2026-11-25T00:00:10Z").await.unwrap(), 1);
        let raw = q("SELECT error_detail FROM upstream_outbox WHERE account_id=?1 AND field='location'").bind(&a).fetch_all(&s.backend).await.unwrap();
        let sealed = raw[0].get_opt_blob("error_detail").unwrap();
        assert!(!sealed.windows(6).any(|w| w == b"Secret"), "error_detail is sealed");
        assert_eq!(s.upstream_error_detail(&a, &m, FIELD_LOCATION).await.unwrap().as_deref(), Some(detail));
        assert!(s.due_upstream_intents(&a, "2026-11-25T00:00:05Z", 10).await.unwrap().iter().all(|i| i.field != FIELD_LOCATION));
        assert!(s.due_upstream_intents(&a, "2026-11-25T00:00:10Z", 10).await.unwrap().iter().any(|i| i.field == FIELD_LOCATION));
        // mark_unresolved keeps sent; drop removes everything of a message.
        let l = intents(s, &a).await.into_iter().find(|i| i.field == FIELD_LOCATION).unwrap();
        s.claim_upstream_intents(&a, &[(m.clone(), FIELD_LOCATION.into(), l.seq)], "w").await.unwrap();
        assert_eq!(s.mark_upstream_unresolved(&a, "w", &[(m.clone(), FIELD_LOCATION.into())], "lost", None).await.unwrap(), 1);
        assert_eq!(s.drop_upstream_intents(&a, &[m.clone(), n.clone()]).await.unwrap(), 2);
        assert!(intents(s, &a).await.is_empty());
        backlog_matches(s, &a).await;
        f.done().await;
    }
}

#[tokio::test]
async fn erasure_leaves_no_outbox_rows() {
    for f in fixtures().await {
        let s = &f.store;
        let a = account(s).await;
        let keep = account(s).await;
        for acc in [&a, &keep] {
            let inbox = mailbox(s, acc, "INBOX", 7).await;
            let m = message(s, acc, &inbox, 5, 7, "[]").await;
            s.set_flags_with_intents(acc, &set(&m, r#"["Seen"]"#), IntentOrigin::User, 100).await.unwrap();
            s.reject_and_revert(acc, &m, &seen_field(), None, "noperm").await.unwrap();
            s.set_flags_with_intents(acc, &set(&m, r#"["Seen"]"#), IntentOrigin::User, 100).await.unwrap();
        }
        let person = crate::erasure::PersonIds { account_ids: vec![a.clone()], ..Default::default() };
        let (_, errors) = s.dsr_purge_person(&person).await;
        assert!(errors.is_empty(), "{errors:?}");
        for table in ["upstream_outbox", "upstream_outbox_head", "upstream_notice"] {
            let sql: &'static str = match table {
                "upstream_outbox" => "SELECT COUNT(*) FROM upstream_outbox WHERE account_id=?1",
                "upstream_outbox_head" => "SELECT COUNT(*) FROM upstream_outbox_head WHERE account_id=?1",
                _ => "SELECT COUNT(*) FROM upstream_notice WHERE account_id=?1",
            };
            assert_eq!(q(sql).bind(&a).fetch_scalar_i64(&s.backend).await.unwrap(), 0, "{table} erased");
            assert_eq!(q(sql).bind(&keep).fetch_scalar_i64(&s.backend).await.unwrap(), 1, "{table} of another person kept");
        }
        let verify = s.dsr_verify_person(&person).await;
        assert!(!verify.keys().any(|t| t.starts_with("upstream_")), "{verify:?}");
        f.done().await;
    }
}

/// Seq order equals commit order: every writer allocates its seq and its
/// Email change state (MAX+1 under the same account lock) in one transaction,
/// so ordering the intents by seq must order their change states too.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn concurrent_writers_keep_seq_order_equal_to_commit_order() {
    for f in fixtures().await {
        let s = std::sync::Arc::new(f.store.clone());
        let a = account(&s).await;
        let inbox = mailbox(&s, &a, "INBOX", 7).await;
        let mut ids = Vec::new();
        for uid in 1..=64u32 {
            ids.push(message(&s, &a, &inbox, uid, 7, "[]").await);
        }
        let mut tasks = Vec::new();
        for writer in 0..16usize {
            let s = s.clone();
            let a = a.clone();
            let mine: Vec<String> = ids.iter().skip(writer).step_by(16).cloned().collect();
            tasks.push(tokio::spawn(async move {
                let mut states = Vec::new();
                for id in mine {
                    let w = loop {
                        match s.set_flags_with_intents(&a, &set(&id, r#"["Seen"]"#), IntentOrigin::User, 1_000).await {
                            Ok(w) => break w,
                            Err(StoreError::SourceChanged) => continue,
                            Err(e) => panic!("{e}"),
                        }
                    };
                    states.push((id, w.done().unwrap().email_state.unwrap()));
                }
                states
            }));
        }
        let mut state_of: HashMap<String, u64> = HashMap::new();
        for t in tasks {
            for (id, state) in t.await.unwrap() {
                state_of.insert(id, state);
            }
        }
        let rows = intents(&s, &a).await;
        assert_eq!(rows.len(), 64);
        let states: Vec<u64> = rows.iter().map(|r| state_of[&r.stable_id]).collect();
        assert!(states.windows(2).all(|w| w[0] < w[1]), "seq order == change-state order: {states:?}");
        let seqs: BTreeSet<i64> = rows.iter().map(|r| r.seq).collect();
        assert_eq!(seqs.len(), 64, "seqs are distinct");
        backlog_matches(&s, &a).await;
        drop(s);
        f.done().await;
    }
}

/// With no caller, the module writes nothing: a fresh store has empty tables
/// and the existing flag and move writers leave them empty.
#[tokio::test]
async fn existing_writers_never_touch_the_outbox() {
    for f in fixtures().await {
        let s = &f.store;
        let a = account(s).await;
        let inbox = mailbox(s, &a, "INBOX", 7).await;
        let archive = mailbox(s, &a, "Archive", 8).await;
        let m = message(s, &a, &inbox, 5, 7, "[]").await;
        s.set_flags_batch(&set(&m, r#"["Seen"]"#)).await.unwrap();
        let msg = s.get_message(&m).await.unwrap();
        assert!(s.relocate_message_if_current(&msg, &archive, 9, 8).await.unwrap());
        for sql in ["SELECT COUNT(*) FROM upstream_outbox", "SELECT COUNT(*) FROM upstream_outbox_head", "SELECT COUNT(*) FROM upstream_notice"] {
            assert_eq!(q(sql).fetch_scalar_i64(&s.backend).await.unwrap(), 0, "{sql}");
        }
        f.done().await;
    }
}

/// migrate-store copies the three tables; rows in flight arrive `pending`
/// (flags) or `unresolved` (moves, APPENDs) and never with a claim token.
#[tokio::test]
async fn migrate_store_round_trip_carries_intents_and_resolves_claims() {
    let dir = std::env::temp_dir().join(format!("mw-outbox-migrate-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("src.sqlite");
    let key = ServerKey::from_bytes(&[5u8; 32]).unwrap();
    let src = Store::open(path.to_str().unwrap(), key.clone()).await.unwrap();
    let a = account(&src).await;
    let inbox = mailbox(&src, &a, "INBOX", 7).await;
    let archive = mailbox(&src, &a, "Archive", 8).await;
    let m = message(&src, &a, &inbox, 5, 7, "[]").await;
    let n = message(&src, &a, &inbox, 6, 7, "[]").await;
    let p = message(&src, &a, &inbox, 7, 7, "[]").await;
    src.set_flags_with_intents(&a, &[(n.clone(), r#"["Seen"]"#.into()), (p.clone(), r#"["Flagged"]"#.into())], IntentOrigin::Sieve, 100).await.unwrap();
    let msg = src.get_message(&m).await.unwrap();
    src.relocate_with_intent(&msg, &archive, IntentOrigin::User, 100).await.unwrap();
    let keys: Vec<(String, String, i64)> = intents(&src, &a).await.into_iter().filter(|i| i.stable_id != p).map(|i| (i.stable_id, i.field, i.seq)).collect();
    src.claim_upstream_intents(&a, &keys, "source-token").await.unwrap();
    src.retry_upstream_intents(&a, "source-token", &[(n.clone(), seen_field())], "transient", Some("server said no"), "2026-11-25T00:00:00Z").await.unwrap();
    let seq = intents(&src, &a).await.into_iter().find(|i| i.stable_id == n).unwrap().seq;
    src.claim_upstream_intents(&a, &[(n.clone(), seen_field(), seq)], "source-token").await.unwrap();
    src.set_flags_with_intents(&a, &set(&p, "[]"), IntentOrigin::User, 100).await.unwrap();
    src.set_flags_with_intents(&a, &set(&p, r#"["Flagged"]"#), IntentOrigin::User, 100).await.unwrap();
    src.reject_and_revert(&a, &p, &flag_field("\"Flagged\""), None, "noperm").await.unwrap();
    src.set_upstream_paused(&a, Some("operator")).await.unwrap();
    let before = intents(&src, &a).await;
    let head = src.upstream_outbox_head(&a).await.unwrap().unwrap();
    src.close().await;

    for f in fixtures().await {
        // Same server key: the sealed error detail must open on the destination.
        let dest = if f.pg.is_some() {
            let (mut admin, schema) = f.pg.unwrap();
            f.store.close().await;
            let dsn = std::env::var("MW_TEST_PG").unwrap();
            let mut scoped = url::Url::parse(&dsn).unwrap();
            scoped.query_pairs_mut().append_pair("options", &format!("-csearch_path={schema}"));
            let dest = Store::open(scoped.as_str(), key.clone()).await.unwrap();
            dest.migrate_from_sqlite(path.to_str().unwrap()).await.unwrap();
            let check = check_copy(&dest, &a, &before, &head, &m, &n).await;
            dest.close().await;
            sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&mut admin).await.unwrap();
            check
        } else {
            f.store.close().await;
            let dest = Store::open_in_memory(key.clone()).await.unwrap();
            dest.migrate_from_sqlite(path.to_str().unwrap()).await.unwrap();
            let check = check_copy(&dest, &a, &before, &head, &m, &n).await;
            dest.close().await;
            check
        };
        let _ = dest;
    }
    let _ = std::fs::remove_dir_all(&dir);
}

async fn check_copy(dest: &Store, a: &str, before: &[OutboxIntent], head: &OutboxHead, m: &str, n: &str) {
    let after = intents(dest, a).await;
    assert_eq!(after.len(), before.len());
    for (b, x) in before.iter().zip(&after) {
        assert_eq!((&b.stable_id, &b.field, b.seq, &b.want, &b.base, &b.base_location, b.origin, b.attempts), (&x.stable_id, &x.field, x.seq, &x.want, &x.base, &x.base_location, x.origin, x.attempts));
        assert_eq!(x.claim, None, "no claim token crosses a migration");
        let expected = match (b.state, x.field.as_str()) {
            (IntentState::Inflight, FIELD_LOCATION | FIELD_APPEND) => IntentState::Unresolved,
            (IntentState::Inflight, _) => IntentState::Pending,
            (state, _) => state,
        };
        assert_eq!(x.state, expected, "{}", x.field);
    }
    let location = after.iter().find(|i| i.stable_id == m).unwrap();
    assert_eq!((location.state, location.sent.is_some()), (IntentState::Unresolved, true), "recovery needs what was sent");
    let flag = after.iter().find(|i| i.stable_id == n).unwrap();
    assert_eq!((flag.state, flag.sent.as_deref()), (IntentState::Pending, None));
    assert_eq!(dest.upstream_error_detail(a, n, &seen_field()).await.unwrap().as_deref(), Some("server said no"));
    assert_eq!(dest.upstream_outbox_head(a).await.unwrap().as_ref(), Some(head));
    assert_eq!(dest.upstream_notices(a, 10).await.unwrap().len(), 1);
    backlog_matches(dest, a).await;
}
