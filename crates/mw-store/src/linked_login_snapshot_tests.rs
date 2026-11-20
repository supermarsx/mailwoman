//! Same-boundary direct-login inputs, never an authentication cache.
use super::*;
use crate::{Credentials, ServerKey, identity_principal::VerifiedPrimary};
async fn verified(store: &Store, realm: &str, account: &str) -> String {
    let session = store
        .create_session(
            account,
            "user@fixture.invalid",
            &format!("https://fixture.invalid/{realm}/session"),
            &format!("https://fixture.invalid/{realm}/api"),
            &Credentials {
                username: "user@fixture.invalid".into(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap();
    store
        .set_session_authentication(&session, "password+factor")
        .await
        .unwrap();
    store
        .bind_verified_primary(
            &session,
            VerifiedPrimary {
                account_id: account,
                kind: "mailbox",
                authority: realm,
                subject: "verified",
            },
            100,
        )
        .await
        .unwrap()
        .unwrap();
    session
}
async fn contract(store: &Store, writer: &Store) {
    let token = crate::seal::random_token();
    let account = format!("target-{token}");
    let missing_account = format!("unlinked-{token}");
    let owner = verified(store, &format!("owner-{token}"), &format!("owner-{token}")).await;
    let child = verified(store, &format!("child-{token}"), &account).await;
    let link = store
        .attach_verified_link(&owner, &child, "fixture", 5, 200)
        .await
        .unwrap()
        .unwrap();
    for kind in [None, Some(0_i64), Some(1), Some(2)] {
        let id = if let Some(kind) = kind {
            q("UPDATE sessions SET linked_internal=?1 WHERE id=?2")
                .bind(kind)
                .bind(&child)
                .execute(&store.backend)
                .await
                .unwrap();
            child.clone()
        } else {
            format!("missing-{token}")
        };
        for (target, present) in [(&account, true), (&missing_account, false)] {
            let (old, old_queries) = crate::pool_telemetry::measure_queries(async {
                let kind = store.linked_session_kind(&id).await?;
                Ok::<_, StoreError>(kind != Some(1) && store.is_linked_mailbox(target).await?)
            })
            .await;
            let (new, new_queries) = crate::pool_telemetry::measure_queries(
                store.linked_direct_login_policy_required(&id, target),
            )
            .await;
            assert_eq!(old.unwrap(), kind != Some(1) && present);
            assert_eq!(new.unwrap(), kind != Some(1) && present);
            assert_eq!(
                new_queries.read.calls, 1,
                "one fresh scalar SQL per invocation"
            );
            assert_eq!(
                old_queries.read.calls,
                if kind == Some(1) { 1 } else { 2 },
                "retained sources already short-circuited the old second query"
            );
        }
    }
    q("UPDATE sessions SET linked_internal=0 WHERE id=?1")
        .bind(&child)
        .execute(&writer.backend)
        .await
        .unwrap();
    assert!(
        store
            .linked_direct_login_policy_required(&child, &account)
            .await
            .unwrap()
    );
    q("UPDATE sessions SET linked_internal=1 WHERE id=?1")
        .bind(&child)
        .execute(&writer.backend)
        .await
        .unwrap();
    assert!(
        !store
            .linked_direct_login_policy_required(&child, &account)
            .await
            .unwrap(),
        "independent writer promotion must be visible immediately"
    );
    q("UPDATE sessions SET linked_internal=0 WHERE id=?1")
        .bind(&child)
        .execute(&writer.backend)
        .await
        .unwrap();
    assert!(
        store
            .linked_direct_login_policy_required(&child, &account)
            .await
            .unwrap()
    );
    assert!(
        writer
            .unlink_account(&owner, &link.id, &link.revision)
            .await
            .unwrap()
    );
    assert!(
        !store
            .linked_direct_login_policy_required(&child, &account)
            .await
            .unwrap(),
        "link removal cannot leave a cached target classification"
    );
    store.delete_session(&owner).await.unwrap();
    store.delete_session(&child).await.unwrap();
}
#[tokio::test]
async fn sqlite_linked_login_snapshot_is_one_fresh_read_and_observes_independent_writer() {
    let directory = std::env::temp_dir().join(format!(
        "mw-linked-admission-{}",
        crate::seal::random_token()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("fixture.db");
    let key = || ServerKey::from_bytes(&[76; 32]).unwrap();
    let store = Store::open(path.to_str().unwrap(), key()).await.unwrap();
    let writer = Store::open(path.to_str().unwrap(), key()).await.unwrap();
    contract(&store, &writer).await;
    writer.close().await;
    store.close().await;
    drop(writer);
    drop(store);
    std::fs::remove_dir_all(directory).unwrap();
}
#[tokio::test]
async fn postgres_linked_login_snapshot_is_one_fresh_read_and_observes_independent_writer() {
    let Ok(dsn) = std::env::var("MW_BG_TEST_PG") else {
        eprintln!("SKIP: MW_BG_TEST_PG absent");
        return;
    };
    let key = || ServerKey::from_bytes(&[76; 32]).unwrap();
    let store = Store::open_postgres(&dsn, key()).await.unwrap();
    let writer = Store::open_postgres(&dsn, key()).await.unwrap();
    contract(&store, &writer).await;
    writer.close().await;
    store.close().await;
}
#[tokio::test]
async fn linked_login_snapshot_does_not_reuse_a_previous_answer_on_storage_failure() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(
        !store
            .linked_direct_login_policy_required("missing", "missing")
            .await
            .unwrap()
    );
    store.close().await;
    assert!(
        store
            .linked_direct_login_policy_required("missing", "missing")
            .await
            .is_err()
    );
}
