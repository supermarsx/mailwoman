use mw_store::nextcloud_links::{NextcloudInstance, NextcloudLink};
use mw_store::{ServerKey, Store};
use sqlx::{Connection, Executor};
#[path = "../src/test_db.rs"]
mod test_db;
fn key() -> ServerKey {
    ServerKey::from_bytes(&[17; 32]).unwrap()
}
async fn copy_into(target: &Store) {
    let dir = test_db::unique_dir("nextcloud-copy");
    let path = dir.join("source.db");
    let source = Store::open(path.to_str().unwrap(), key()).await.unwrap();
    let account = source
        .create_account(
            &mw_store::NewAccount {
                kind: mw_store::AccountKind::Imap,
                host: "mail.test",
                port: 993,
                tls: "implicit",
                username: "alice@test",
                sync_policy_json: "{}",
            },
            &mw_store::Credentials {
                username: "alice@test".into(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap();
    let instance = source
        .put_nextcloud_instance(
            &NextcloudInstance {
                id: "cloud".into(),
                name: "Private cloud".into(),
                base_url: "https://cloud.test/prefix/".into(),
                ca_pem: "fixture-public-ca-material".into(),
                enabled: true,
                revision: String::new(),
            },
            None,
        )
        .await
        .unwrap()
        .unwrap();
    let link = source
        .put_verified_nextcloud_link(
            &NextcloudLink {
                account_id: account.clone(),
                instance_id: instance.id.clone(),
                instance_revision: instance.revision.clone(),
                username: "remote-user".into(),
                revision: String::new(),
            },
            "fixture-app-secret",
            None,
        )
        .await
        .unwrap()
        .unwrap();
    let binding = source
        .apply_nextcloud_pim(mw_store::nextcloud_pim::NextcloudPimApply {
            account_id: &account,
            instance_id: "cloud",
            link_revision: &link.revision,
            kind: "calendar",
            href: "https://cloud.example.test/remote.php/dav/calendars/remote-user/work/",
            name: "Work",
        })
        .await
        .unwrap()
        .unwrap();
    let report = target
        .migrate_from_sqlite(path.to_str().unwrap())
        .await
        .unwrap();
    assert!(
        report
            .tables
            .iter()
            .any(|(name, count)| name == "nextcloud_instance" && *count == 1)
    );
    assert!(
        report
            .tables
            .iter()
            .any(|(name, count)| name == "nextcloud_account_link" && *count == 1)
    );
    assert_eq!(
        target.nextcloud_pim_bindings(&account).await.unwrap(),
        vec![binding.clone()]
    );
    assert_eq!(
        target
            .get_calendar(&binding.collection_id)
            .await
            .unwrap()
            .unwrap()
            .account_id,
        account
    );
    let (copied, copied_link, password) = target
        .nextcloud_authority(&account, "cloud")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(copied, instance);
    assert_eq!(copied_link, link);
    assert_eq!(password, "fixture-app-secret");
    assert!(
        target
            .nextcloud_authority("bob", "cloud")
            .await
            .unwrap()
            .is_none()
    );
    let mut revoked = copied.clone();
    revoked.enabled = false;
    target
        .put_nextcloud_instance(&revoked, Some(&copied.revision))
        .await
        .unwrap()
        .unwrap();
    assert!(
        target
            .nextcloud_authority(&account, "cloud")
            .await
            .unwrap()
            .is_none()
    );
}
#[tokio::test]
async fn sqlite_copy_preserves_nextcloud_authority_ca_and_sealed_account_link() {
    let dir = test_db::unique_dir("nextcloud-copy-target");
    let target = Store::open(dir.join("target.db").to_str().unwrap(), key())
        .await
        .unwrap();
    copy_into(&target).await;
}
#[tokio::test]
async fn postgres_copy_preserves_nextcloud_authority_ca_and_sealed_account_link() {
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        eprintln!("SKIP: MW_TEST_PG absent");
        return;
    };
    let schema = format!("nextcloud_copy_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    admin
        .execute(format!("CREATE SCHEMA {schema}").as_str())
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let target = Store::open(scoped.as_str(), key()).await.unwrap();
    copy_into(&target).await;
    drop(target);
    admin
        .execute(format!("DROP SCHEMA {schema} CASCADE").as_str())
        .await
        .unwrap();
}
