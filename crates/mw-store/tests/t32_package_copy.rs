use mw_store::{AccountKind,Credentials,NewAccount,ServerKey,Store};
use sqlx::{Connection,Executor};
#[path="../src/test_db.rs"] mod test_db;
fn key()->ServerKey {ServerKey::from_bytes(&[22;32]).unwrap()}
async fn exercise(target:&Store) {
    let dir=test_db::unique_dir("package-copy");let path=dir.join("source.db");let source=Store::open(path.to_str().unwrap(),key()).await.unwrap();
    let account=source.create_account(&NewAccount{kind:AccountKind::Imap,host:"host",port:993,tls:"implicit",username:"one@example.test",sync_policy_json:"{}"},&Credentials{username:"one@example.test".into(),password:"mail-secret".into()}).await.unwrap();
    let provider=source.put_package_provider(true,900,Some("provider-secret"),None).await.unwrap().unwrap();
    let revision=source.put_package_account(&account,"{\"enabled\":true,\"packages\":[]}",None).await.unwrap().unwrap();
    let report=target.migrate_from_sqlite(path.to_str().unwrap()).await.unwrap();
    assert!(report.tables.iter().any(|(name,count)|name=="package_provider"&&*count==1));
    assert!(report.tables.iter().any(|(name,count)|name=="package_account"&&*count==1));
    assert_eq!(target.package_provider_key(&provider.revision).await.unwrap().as_deref(),Some("provider-secret"));
    assert_eq!(target.package_account(&account).await.unwrap().unwrap().1,revision);
    assert!(target.package_account("foreign-account").await.unwrap().is_none());
    let (left,right)=tokio::join!(target.put_package_account(&account,"left",Some(&revision)),target.put_package_account(&account,"right",Some(&revision)));
    assert_ne!(left.unwrap().is_some(),right.unwrap().is_some());
    target.put_package_provider(false,900,None,Some(&provider.revision)).await.unwrap().unwrap();
    assert!(target.package_provider_key(&provider.revision).await.unwrap().is_none());
}
#[tokio::test]
async fn sqlite_package_copy_preserves_credentials_and_account_revisions() {
    let target=Store::open_in_memory(key()).await.unwrap();exercise(&target).await;
}
#[tokio::test]
async fn postgres_package_copy_preserves_credentials_and_account_revisions() {
    let Ok(dsn)=std::env::var("MW_TEST_PG") else {eprintln!("SKIP: MW_TEST_PG absent");return;};
    let schema=format!("package_copy_{}",uuid::Uuid::new_v4().simple());let mut admin=sqlx::PgConnection::connect(&dsn).await.unwrap();admin.execute(format!("CREATE SCHEMA {schema}").as_str()).await.unwrap();
    let mut scoped=url::Url::parse(&dsn).unwrap();scoped.query_pairs_mut().append_pair("options",&format!("-csearch_path={schema}"));
    let target=Store::open(scoped.as_str(),key()).await.unwrap();exercise(&target).await;drop(target);admin.execute(format!("DROP SCHEMA {schema} CASCADE").as_str()).await.unwrap();
}
