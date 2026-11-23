//! t34-e12: each joined read answers exactly what the statements it replaces
//! answer, on the same data, on SQLite and (with `DATABASE_URL_PG`) PostgreSQL,
//! including the account boundaries and the fail-closed zero-access flag.
use crate::settings_policy::{SettingPolicyRow, SettingScope, SettingSubject, SettingValueRow};
use crate::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, SavedSearchRow, ServerKey,
    Store, StoredMeta, ZeroAccessRow, q,
};

async fn account(store: &Store, name: &str) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "hot.invalid",
                port: 993,
                tls: "implicit",
                username: name,
                sync_policy_json: "{}",
            },
            &Credentials { username: name.into(), password: "p".into() },
        )
        .await
        .unwrap()
}

async fn mailbox(store: &Store, account: &str, name: &str, role: Option<&str>) -> String {
    store
        .upsert_mailbox(&MailboxUpsert {
            account_id: account,
            name,
            role,
            uidvalidity: 7,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap()
}

async fn message(store: &Store, account: &str, mailbox: &str, uid: u32, date: Option<&str>, thread: Option<&str>) -> String {
    store
        .upsert_message(&MessageUpsert {
            account_id: account,
            mailbox_id: mailbox,
            uid,
            uidvalidity: 7,
            message_id: None,
            thread_id: thread,
            internaldate: date,
            size: 10,
            flags_json: "[]",
            envelope: Some(b"{}"),
            blob_ref: None,
        })
        .await
        .unwrap()
}

/// Two accounts with mailboxes, messages (some with metadata, some undated,
/// some threaded), saved searches, policies and values.
struct Data {
    store: Store,
    a: String,
    b: String,
    a_ids: Vec<String>,
    b_ids: Vec<String>,
}

async fn seed(store: Store) -> Data {
    let tag = uuid::Uuid::new_v4().simple().to_string();
    let a = account(&store, &format!("a-{tag}@hot.invalid")).await;
    let b = account(&store, &format!("b-{tag}@other.invalid")).await;
    let mut ids = Vec::new();
    for account in [&a, &b] {
        // Names out of insertion order, to exercise the ORDER BY.
        let inbox = mailbox(&store, account, "INBOX", Some("inbox")).await;
        mailbox(&store, account, "Archive", None).await;
        let sent = mailbox(&store, account, "Sent", Some("sent")).await;
        mailbox(&store, account, "Sent Old", Some("sent")).await;
        let mut own = Vec::new();
        for uid in 1..=6u32 {
            let date = (uid % 3 != 0).then(|| format!("2026-11-0{uid}T00:00:00Z"));
            own.push(message(&store, account, &inbox, uid, date.as_deref(), None).await);
        }
        for uid in 1..=5u32 {
            let date = (uid != 2).then(|| format!("2026-10-0{uid}T00:00:00Z"));
            let thread = (uid % 2 == 1).then(|| format!("t{uid}"));
            message(&store, account, &sent, 100 + uid, date.as_deref(), thread.as_deref()).await;
        }
        store
            .upsert_message_meta(&own[1], &StoredMeta { pinned: true, snoozed_until: Some("2027-01-01T00:00:00Z".into()), follow_up_at: None })
            .await
            .unwrap();
        let owner = store.native_personal_subject(account).await.unwrap().unwrap();
        for (id, name, folder) in [("s2", "zeta", true), ("s1", "alpha", false), ("s3", "alpha", true)] {
            store
                .upsert_personal_saved_search(&SavedSearchRow {
                    id: format!("{id}-{tag}-{}", &account[..6]),
                    user: owner.clone(),
                    name: name.into(),
                    query_json: "{}".into(),
                    as_folder: folder,
                })
                .await
                .unwrap();
        }
        ids.push(own);
    }
    for (key, value) in [("mail.folderRole.trash", r#"{"path":"Trash"}"#), ("mail.folderRole.archive", "null")] {
        store
            .put_setting_policy(&SettingPolicyRow {
                key: key.into(),
                global_value: Some(value.into()),
                allow_user_override: true,
                global_locked: false,
                updated_by: "test".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }
    store
        .put_setting_value(&SettingValueRow {
            scope: SettingScope::User,
            scope_id: a.clone(),
            key: "mail.folderRole.trash".into(),
            value: r#"{"path":"Deleted"}"#.into(),
            locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    let b_ids = ids.pop().unwrap();
    let a_ids = ids.pop().unwrap();
    Data { store, a, b, a_ids, b_ids }
}

async fn sqlite() -> Data {
    seed(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await
}

async fn pg() -> Option<Data> {
    let dsn = mw_test_gate::pg_dsn()?;
    Some(seed(Store::open(&dsn, ServerKey::generate()).await.unwrap()).await)
}

async fn zero_access(store: &Store, account: &str, enabled: bool) {
    store
        .upsert_zeroaccess(&ZeroAccessRow {
            account_id: account.into(),
            enabled,
            wrapped_root_key: vec![1; 16],
            kdf_params_json: "{}".into(),
            recovery_wrapped: None,
            paired_devices_json: "[]".into(),
        })
        .await
        .unwrap();
}

async fn equivalence(d: &Data) {
    let s = &d.store;
    for account in [&d.a, &d.b] {
        // Mailbox list + username.
        let (username, boxes) = s.account_username_and_mailboxes(account).await.unwrap();
        assert_eq!(username, s.get_account(account).await.unwrap().username);
        assert_eq!(format!("{boxes:?}"), format!("{:?}", s.list_mailboxes(account).await.unwrap()));
        // Saved searches + owner.
        let (owner, saved) = s.native_personal_saved_searches(account).await.unwrap().unwrap();
        let expected_owner = s.native_personal_subject(account).await.unwrap().unwrap();
        assert_eq!(owner, expected_owner);
        let expected = s.list_personal_saved_searches(&expected_owner).await.unwrap();
        assert_eq!(saved.len(), 3);
        assert_eq!(format!("{saved:?}"), format!("{expected:?}"));
        // Focus sent scan, at a limit below and above the Sent size.
        for limit in [3, 500] {
            let (sent, page) = s.focus_sent_scan(account, limit).await.unwrap().unwrap();
            let expected_sent = s
                .list_mailboxes(account)
                .await
                .unwrap()
                .into_iter()
                .find(|m| m.role.as_deref() == Some("sent"))
                .unwrap();
            assert_eq!(sent, expected_sent.id);
            let ids = s.list_message_ids(&sent, limit, 0).await.unwrap();
            assert_eq!(page.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>(), ids, "limit {limit}");
            let threads: Vec<Option<String>> = s
                .get_account_messages(account, &ids)
                .await
                .unwrap()
                .into_iter()
                .map(|m| m.and_then(|m| m.thread_id))
                .collect();
            assert_eq!(page.iter().map(|(_, t)| t.clone()).collect::<Vec<_>>(), threads);
        }
        // Role values + policies under one revision.
        let address = s.get_account(account).await.unwrap().username;
        let subject = SettingSubject {
            personal_owner: None,
            domain: address.rsplit_once('@').map_or("", |(_, d)| d),
            account_id: account,
            address: &address,
        };
        let keys = ["mail.folderRole.trash", "mail.folderRole.archive", "mail.folderRole.junk", "mail.folderRole.trash"];
        for _ in 0..2 {
            // First call fills the policy memo, the second is served from it.
            let (values, policies) = s.setting_values_and_policies_for_subject(&subject, &keys).await.unwrap();
            assert_eq!(values, s.setting_values_for_subject(&subject).await.unwrap());
            assert_eq!(policies, s.get_setting_policies_batch(&keys[..3]).await.unwrap());
        }
    }
    // A policy write is seen at once (the revision moves).
    s.put_setting_policy(&SettingPolicyRow {
        key: "mail.folderRole.junk".into(),
        global_value: Some(r#"{"path":"Spam"}"#.into()),
        allow_user_override: true,
        global_locked: false,
        updated_by: "test".into(),
        updated_at: 2,
    })
    .await
    .unwrap();
    let subject = SettingSubject { personal_owner: None, domain: "", account_id: &d.a, address: "" };
    let (_, policies) = s.setting_values_and_policies_for_subject(&subject, &["mail.folderRole.junk"]).await.unwrap();
    assert_eq!(policies.len(), 1, "a new policy is visible on the next read");

    // Messages + metadata + posture: same rows, foreign ids absent.
    let mut asked: Vec<String> = d.a_ids.clone();
    asked.extend(d.b_ids[..2].iter().cloned());
    asked.push(d.a_ids[0].clone());
    asked.push("absent".into());
    for (enabled, expect) in [(None, false), (Some(true), true), (Some(false), false)] {
        if let Some(enabled) = enabled {
            zero_access(s, &d.a, enabled).await;
        }
        let (rows, flag) = s.get_account_messages_with_metas_and_posture(&d.a, &asked).await.unwrap();
        let plain = s.get_account_messages_with_metas(&d.a, &asked).await.unwrap();
        assert_eq!(format!("{rows:?}"), format!("{plain:?}"));
        assert!(rows[d.a_ids.len()].is_none() && rows[d.a_ids.len() + 1].is_none(), "B's ids are absent");
        assert_eq!(flag, Some(expect), "flag for {enabled:?}");
        let live = s.get_zeroaccess(&d.a).await.unwrap().is_some_and(|r| r.enabled);
        assert_eq!(flag, Some(live), "the flag is get_zeroaccess's answer");
    }
    // B's zero-access never colours A's rows, and nothing owned means no flag.
    zero_access(s, &d.a, false).await;
    zero_access(s, &d.b, true).await;
    let (_, flag) = s.get_account_messages_with_metas_and_posture(&d.a, &d.a_ids[..2]).await.unwrap();
    assert_eq!(flag, Some(false));
    let (rows, flag) = s.get_account_messages_with_metas_and_posture(&d.a, &d.b_ids[..2]).await.unwrap();
    assert!(rows.iter().all(Option::is_none));
    assert_eq!(flag, None);
    let (_, flag) = s.get_account_messages_with_metas_and_posture(&d.b, &d.b_ids[..2]).await.unwrap();
    assert_eq!(flag, Some(true));

    // Sync cursors: the set load_cursor answers Some for, per account.
    let boxes = s.list_mailboxes(&d.a).await.unwrap();
    s.save_cursor(&d.a, &boxes[0].id, "{}").await.unwrap();
    let with = s.mailboxes_with_cursor(&d.a).await.unwrap();
    for m in &boxes {
        assert_eq!(with.contains(&m.id), s.load_cursor(&d.a, &m.id).await.unwrap().is_some());
    }
    assert!(s.mailboxes_with_cursor(&d.b).await.unwrap().is_empty());

    // A missing account: the same refusals as the statements replaced.
    assert!(matches!(s.account_username_and_mailboxes("missing").await, Err(crate::StoreError::NotFound)));
    assert!(s.native_personal_saved_searches("missing").await.unwrap().is_none());
    assert!(s.focus_sent_scan("missing", 10).await.unwrap().is_none());
}

#[tokio::test]
async fn joined_hot_route_reads_equal_the_reads_they_replace_sqlite() {
    equivalence(&sqlite().await).await;
}

#[tokio::test]
async fn joined_hot_route_reads_equal_the_reads_they_replace_postgres() {
    let Some(d) = pg().await else {
        mw_test_gate::skip("DATABASE_URL_PG and MW_E14_PG_DSN unset: t34-e12 joined-read equivalence on PostgreSQL");
        return;
    };
    equivalence(&d).await;
}

/// SQLite stores whatever a raw writer puts in `enabled`; a value that is not
/// `0` must read as zero-access (fail closed), as the posture source's own
/// failed read does.
#[tokio::test]
async fn a_malformed_zero_access_flag_fails_closed() {
    let d = sqlite().await;
    zero_access(&d.store, &d.a, false).await;
    q("UPDATE zeroaccess_accounts SET enabled='broken' WHERE account_id=?1")
        .bind(&d.a)
        .execute(&d.store.backend)
        .await
        .unwrap();
    let (_, flag) = d.store.get_account_messages_with_metas_and_posture(&d.a, &d.a_ids[..1]).await.unwrap();
    assert_eq!(flag, Some(true));
}

/// A Sent-less account has no scan, like the `find` over the mailbox list.
#[tokio::test]
async fn no_sent_mailbox_means_no_focus_scan() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let a = account(&store, "nosent@hot.invalid").await;
    mailbox(&store, &a, "INBOX", Some("inbox")).await;
    assert!(store.focus_sent_scan(&a, 10).await.unwrap().is_none());
    // Sent exists but is empty: the mailbox, no ids.
    let sent = mailbox(&store, &a, "Sent", Some("sent")).await;
    assert_eq!(store.focus_sent_scan(&a, 10).await.unwrap(), Some((sent, Vec::new())));
}
