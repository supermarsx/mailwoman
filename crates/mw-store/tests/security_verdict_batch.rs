//! `Store::get_security_verdicts`: the batch read behind the message list's
//! compact authentication summary.

use mw_store::{AccountKind, Credentials, NewAccount, SecurityVerdictRow, ServerKey, Store};

async fn account(store: &Store, user: &str) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: user,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: user.into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap()
}

fn row(email_id: &str, account_id: &str, body: &str) -> SecurityVerdictRow {
    SecurityVerdictRow {
        email_id: email_id.into(),
        account_id: account_id.into(),
        raw_hash: "h".into(),
        verdict_json: body.as_bytes().to_vec(),
        computed_at: "2026-11-06T00:00:00Z".into(),
    }
}

#[tokio::test]
async fn batch_returns_only_cached_verdicts_of_the_account() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let mine = account(&store, "me@example.org").await;
    let other = account(&store, "other@example.org").await;
    store.upsert_security_verdict(&row("m1", &mine, "one")).await.unwrap();
    store.upsert_security_verdict(&row("m2", &mine, "two")).await.unwrap();
    store.upsert_security_verdict(&row("m3", &other, "three")).await.unwrap();

    let ids: Vec<String> = ["m1", "m2", "m3", "m4"].iter().map(|s| s.to_string()).collect();
    let found = store.get_security_verdicts(&mine, &ids).await.unwrap();

    assert_eq!(found.len(), 2, "m3 belongs to another account, m4 has no verdict");
    assert_eq!(found["m1"], b"one");
    assert_eq!(found["m2"], b"two");
    assert!(store.get_security_verdicts(&mine, &[]).await.unwrap().is_empty());
}
