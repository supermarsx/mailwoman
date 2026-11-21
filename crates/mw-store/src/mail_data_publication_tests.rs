use super::*;
use crate::{AccountKind,Credentials,MailboxUpsert,MessageUpsert,NewAccount,ServerKey};

async fn account(store:&Store,name:&str)->String {
    store.create_account(&NewAccount {kind:AccountKind::Imap,host:"mail-data.test",port:993,tls:"implicit",username:name,sync_policy_json:"{}"},&Credentials {username:name.into(),password:"synthetic-only".into()}).await.unwrap()
}
async fn mailbox(store:&Store,account:&str,name:&str)->String {
    store.upsert_mailbox(&MailboxUpsert {account_id:account,name,role:Some("inbox"),uidvalidity:1,uidnext:2,highestmodseq:0,total:0,unread:0,parent_id:None}).await.unwrap()
}
async fn snapshot(store:&Store,descriptor:&MailDataDescriptor)->MailDataFence {
    store.mail_data_snapshot(descriptor).await.unwrap().unwrap()
}
async fn changed(store:&Store,descriptor:&MailDataDescriptor,old:&MailDataFence)->MailDataFence {
    let next=snapshot(store,descriptor).await;
    assert_current_compact_tuples(store).await;
    assert!(!old.same_data(&next),"actual source DML must invalidate the old data observation");next
}
async fn assert_current_compact_tuples(store:&Store) {
    assert_eq!(q("SELECT COUNT(*) FROM mail_data_rows v JOIN messages m ON m.stable_id=v.stable_id WHERE v.account_id<>m.account_id OR v.cache_revision<>m.cache_revision OR v.content_revision<>m.content_revision").fetch_scalar_i64(&store.backend).await.unwrap(),0,"compact tuples must match committed source revisions after every source trigger ordering");
}

async fn all_source_dml(store:Store){
    let a=account(&store,"a@mail-data.test").await;
    let b=account(&store,"b@mail-data.test").await;
    let inbox=mailbox(&store,&a,"Inbox").await;
    let sent=mailbox(&store,&a,"Sent").await;
    let raw=store.put_body(&a,b"From: a@mail-data.test\r\nSubject: actual-source\r\n\r\nbody").await.unwrap();
    // A real multi-MiB bodyValues field remains in the FULL source while the
    // strict publication getter returns only the <=128KiB derived projection.
    let envelope=serde_json::to_vec(&serde_json::json!({"subject":"actual-source","messageId":["fixture@mail-data.test"],"inReplyTo":[],"references":[],"hasAttachment":false,"attachments":[],"bodyValues":{"part":{"value":"x".repeat(2*1024*1024)}}})).unwrap();
    let id=store.upsert_message(&MessageUpsert {account_id:&a,mailbox_id:&inbox,uid:1,uidvalidity:1,message_id:Some("<fixture@mail-data.test>"),thread_id:Some("thread-one"),internaldate:Some("2026-01-01T00:00:00Z"),size:64,flags_json:"[]",envelope:Some(&envelope),blob_ref:Some(&raw)}).await.unwrap();
    let descriptor=store.mail_data_descriptor(&a,&[id.clone(),"missing".into(),id.clone()]).unwrap();
    store.seed_mail_data(&descriptor).await.unwrap();
    let mut before=snapshot(&store,&descriptor).await;
    assert_current_compact_tuples(&store).await;
    assert!(before.publishable());
    assert!(store.validate_mail_data_schema(&before).await.unwrap(),"actual installed0109/0110/0113 definitions must qualify in this backend");
    let light=store.get_account_projected_header_envelopes(&a,std::slice::from_ref(&id)).await.unwrap().remove(0).unwrap();
    assert!(light.projected && light.envelope.len()<128*1024);
    assert!(serde_json::from_slice::<serde_json::Value>(&light.envelope).unwrap().get("bodyValues").is_none());
    assert!(store.get_account_envelopes(&a,std::slice::from_ref(&id)).await.unwrap()[0].as_ref().unwrap().len()>2*1024*1024,"full source must not be truncated or replaced by the publication");
    assert!(store.get_account_projected_header_envelopes(&b,std::slice::from_ref(&id)).await.unwrap()[0].is_none());

    // Missing derived state is NOT a missing source message or a publishable
    // notFound. Only an explicit bounded seed may restore its tuple.
    q("DELETE FROM mail_data_rows WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    let missing_tuple=snapshot(&store,&descriptor).await;
    assert!(missing_tuple.needs_seed() && !missing_tuple.publishable());
    assert!(missing_tuple.payload.rows.iter().any(|row|row.id==id && row.owned==1 && row.epoch.is_none()));
    let seeded=store.seed_mail_data(&descriptor).await.unwrap();
    assert_eq!((seeded.account_rows,seeded.message_rows),(0,1));
    before=changed(&store,&descriptor,&before).await;
    let unchanged_seed=store.seed_mail_data(&descriptor).await.unwrap();
    assert_eq!((unchanged_seed.account_rows,unchanged_seed.message_rows),(0,0));

    // Direct opaque revision writes (including legacy empty revisions) mirror
    // the compact tuple without requiring a giant envelope rewrite.
    q("UPDATE messages SET cache_revision='',content_revision='' WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    // PostgreSQL0105 deliberately replaces cache_revision in a BEFORE trigger.
    // Compare with the committed source, while retaining the opaque empty
    // content revision on both backends rather than inventing a nonce format.
    let actual_cache=q("SELECT cache_revision FROM messages WHERE stable_id=?1").bind(&id).fetch_one(&store.backend).await.unwrap().get_string("cache_revision");
    assert!(before.matches_owned_cache_revision(&id,&actual_cache));
    assert!(before.payload.rows.iter().any(|row|row.id==id && row.content.as_deref()==Some("")));
    q("UPDATE messages SET cache_revision='independent-cache',content_revision='independent-content' WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;

    // Row metadata INSERT/UPDATE/DELETE, and raw source UPDATE/rollback/no-op.
    q("INSERT INTO message_meta(stable_id,pinned) VALUES(?1,1)").bind(&id).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    q("UPDATE message_meta SET snoozed_until='2030-01-01T00:00:00Z' WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    q("DELETE FROM message_meta WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    let mut tx=store.backend.begin().await.unwrap();
    q("UPDATE messages SET flags_json='[\"temporary\"]' WHERE stable_id=?1").bind(&id).execute_tx(&mut tx).await.unwrap();
    tx.rollback().await.unwrap();
    assert!(before.same_data(&snapshot(&store,&descriptor).await));
    q("UPDATE messages SET flags_json=flags_json WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    //0105 deliberately advances opaque cache identity even on a semantic
    // no-op. Both backends must mirror that committed source version, including
    // PostgreSQL BEFORE-trigger writes to columns not named in the SET list.
    before=changed(&store,&descriptor,&before).await;
    q("UPDATE messages SET flags_json='[\"seen\"]' WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    let other=store.mail_data_descriptor(&b,std::slice::from_ref(&id)).unwrap();
    let foreign=snapshot(&store,&other).await;
    assert!(foreign.payload.rows.iter().all(|r|r.epoch.is_none()),"foreign identity cannot become an owned row");

    // Source body bytes change in place:0110 changes the content proof,113
    // changes the mail row version, with no body BLOB selected by the fence.
    let replacement=store.key.seal(b"From: changed@mail-data.test\r\n\r\nchanged-body").unwrap();
    q("UPDATE bodies SET sealed_bytes=?2 WHERE blob_ref=?1").bind(&raw).bind(replacement).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    let unchanged=snapshot(&store,&descriptor).await;
    q("UPDATE bodies SET sealed_bytes=sealed_bytes WHERE blob_ref=?1").bind(&raw).execute(&store.backend).await.unwrap();
    assert!(unchanged.same_data(&snapshot(&store,&descriptor).await));
    q("DELETE FROM bodies WHERE blob_ref=?1").bind(&raw).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    let reinserted=store.key.seal(b"From: recreated@mail-data.test\r\n\r\nrecreated-body").unwrap();
    q("INSERT INTO bodies(blob_ref,account_id,sealed_bytes) VALUES(?1,?2,?3)").bind(&raw).bind(&a).bind(reinserted).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;

    // Header companion update/delete and normal bounded lazy repair insert.
    q("UPDATE message_list_envelope SET sealed_envelope=sealed_envelope WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    assert!(before.same_data(&snapshot(&store,&descriptor).await));
    q("DELETE FROM message_list_envelope WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    assert!(!before.publishable());
    assert!(store.get_account_projected_header_envelopes(&a,std::slice::from_ref(&id)).await.unwrap()[0].is_none());
    assert!(store.get_account_header_envelopes(&a,std::slice::from_ref(&id)).await.unwrap()[0].is_some());
    before=changed(&store,&descriptor,&before).await;

    q("UPDATE message_list_envelope SET sealed_envelope=?2 WHERE stable_id=?1").bind(&id).bind(vec![7u8;16]).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    assert!(store.get_account_projected_header_envelopes(&a,std::slice::from_ref(&id)).await.unwrap()[0].is_none(),"authenticated projection corruption must not trigger full-source fallback in strict reader");
    store.get_account_header_envelopes(&a,std::slice::from_ref(&id)).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    q("UPDATE message_list_envelope SET sealed_envelope=?2 WHERE stable_id=?1").bind(&id).bind(vec![0u8;crate::header_envelope::MAX_HEADER_ENVELOPE_BYTES+45]).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    assert!(!before.publishable());
    assert!(store.get_account_projected_header_envelopes(&a,std::slice::from_ref(&id)).await.unwrap()[0].is_none());
    store.get_account_header_envelopes(&a,std::slice::from_ref(&id)).await.unwrap();
    before=changed(&store,&descriptor,&before).await;

    // Verdict INSERT/UPDATE/DELETE change the source version even while mail
    // envelope/flags are unchanged. Legacy NULL proofs never become trusted.
    q("INSERT INTO security_verdicts(email_id,account_id,raw_hash,verdict_json,computed_at,source_revision) VALUES(?1,?2,'synthetic',?3,'2026-01-01T00:00:00Z',NULL)").bind(&id).bind(&a).bind(b"{}".to_vec()).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    q("UPDATE security_verdicts SET computed_at='2026-01-02T00:00:00Z' WHERE email_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    q("DELETE FROM security_verdicts WHERE email_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;

    // Focus dependencies: actual role, Sent move, book/contact insert/update/
    // delete and cross-owner book move. These do not rely on a local setter.
    q("UPDATE mailboxes SET role='sent' WHERE id=?1").bind(&sent).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    q("UPDATE messages SET mailbox_id=?2 WHERE stable_id=?1").bind(&id).bind(&sent).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    // Observe an unrelated/empty selected set: Sent ordering changes must
    // invalidate Focus for OTHER messages, not only this message's row tuple.
    let focus_only=store.mail_data_descriptor(&a,&[]).unwrap();
    let focus_before=snapshot(&store,&focus_only).await;
    q("UPDATE messages SET internaldate='2035-01-01T00:00:00Z',uid=9 WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    changed(&store,&focus_only,&focus_before).await;
    before=changed(&store,&descriptor,&before).await;
    q("INSERT INTO address_books(id,account_id,name) VALUES('mail-data-book',?1,'owned')").bind(&a).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    q("INSERT INTO contacts(id,address_book_id,uid,vcard_raw) VALUES('mail-data-card','mail-data-book','synthetic','BEGIN:VCARD\nVERSION:3.0\nEMAIL:a@mail-data.test\nEND:VCARD')").execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    q("UPDATE contacts SET full_name='Current Name' WHERE id='mail-data-card'").execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    q("DELETE FROM contacts WHERE id='mail-data-card'").execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    q("UPDATE address_books SET account_id=?1 WHERE id='mail-data-book'").bind(&b).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    q("DELETE FROM address_books WHERE id='mail-data-book'").execute(&store.backend).await.unwrap();

    q("INSERT INTO zeroaccess_accounts(account_id,enabled,wrapped_root_key,kdf_params) VALUES(?1,1,?2,'{}')").bind(&a).bind(vec![0u8;16]).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;assert!(!before.standard());
    q("UPDATE zeroaccess_accounts SET enabled=0 WHERE account_id=?1").bind(&a).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;assert!(before.standard());
    q("DELETE FROM zeroaccess_accounts WHERE account_id=?1").bind(&a).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;

    let old_hint=store.mail_data_invalidations(32).await.unwrap().into_iter().find(|hint|hint.account()==a).unwrap();
    q("UPDATE messages SET flags_json='[\"Flagged\"]' WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    assert!(!store.acknowledge_mail_data_invalidation(&old_hint).await.unwrap(),"old hint acknowledgement must not erase a newer committed change");
    let current_hint=store.mail_data_invalidations(32).await.unwrap().into_iter().find(|hint|hint.account()==a).unwrap();
    assert!(store.acknowledge_mail_data_invalidation(&current_hint).await.unwrap());
    assert!(!store.acknowledge_mail_data_invalidation(&current_hint).await.unwrap());

    let empty=store.mail_data_descriptor(&a,&[]).unwrap();
    let old_empty=snapshot(&store,&empty).await;
    q("DELETE FROM messages WHERE stable_id=?1").bind(&id).execute(&store.backend).await.unwrap();
    before=changed(&store,&descriptor,&before).await;
    assert!(before.payload.rows.iter().all(|r|r.epoch.is_none()));
    // A raw-free message demonstrates that the NEW derived FK does not forbid
    // a stable-ID update formerly permitted when no other children existed.
    let rekey=store.upsert_message(&MessageUpsert {account_id:&a,mailbox_id:&inbox,uid:2,uidvalidity:1,message_id:None,thread_id:None,internaldate:None,size:0,flags_json:"[]",envelope:Some(b"{}"),blob_ref:None}).await.unwrap();
    q("UPDATE messages SET stable_id='mail-data-rekeyed' WHERE stable_id=?1").bind(&rekey).execute(&store.backend).await.unwrap();
    assert_eq!(q("SELECT COUNT(*) FROM mail_data_rows WHERE stable_id=?1").bind(&rekey).fetch_scalar_i64(&store.backend).await.unwrap(),0);
    assert_eq!(q("SELECT COUNT(*) FROM mail_data_rows WHERE stable_id='mail-data-rekeyed'").fetch_scalar_i64(&store.backend).await.unwrap(),1);
    q("DELETE FROM mailboxes WHERE id=?1").bind(&sent).execute(&store.backend).await.unwrap();
    q("DELETE FROM accounts WHERE id=?1").bind(&a).execute(&store.backend).await.unwrap();
    assert!(store.mail_data_snapshot(&descriptor).await.unwrap().is_none());
    for sql in ["SELECT COUNT(*) FROM mail_data_accounts WHERE account_id=?1","SELECT COUNT(*) FROM mail_data_rows WHERE account_id=?1","SELECT COUNT(*) FROM mail_data_outbox WHERE account_id=?1"] {
        assert_eq!(q(sql).bind(&a).fetch_scalar_i64(&store.backend).await.unwrap(),0);
    }
    let replacement=account(&store,"replacement@mail-data.test").await;
    q("UPDATE accounts SET id=?2 WHERE id=?1").bind(&replacement).bind(&a).execute(&store.backend).await.unwrap();
    assert!(!old_empty.same_data(&snapshot(&store,&empty).await),"exact account-ID recreation must not ABA into prior result data");
    assert_eq!(q("SELECT COUNT(*) FROM mail_data_accounts WHERE account_id=?1").bind(&replacement).fetch_scalar_i64(&store.backend).await.unwrap(),0,"renamed account cannot leave an orphan personal scope");

    // Actual erasure inventory, not just a direct DELETE, must remove the
    // derived source and outbox at the final source-deletion ordering point.
    let b_descriptor=store.mail_data_descriptor(&b,&[]).unwrap();
    let b_before=snapshot(&store,&b_descriptor).await;
    let person=crate::erasure::PersonIds {address:"b@mail-data.test".into(),account_ids:vec![b.clone()],owners:vec![b.clone()],principals:Vec::new()};
    let (_,errors)=store.dsr_purge_person(&person).await;
    assert!(errors.is_empty(),"actual person purge must retain all source/derived inventory contracts: {errors:?}");
    assert!(store.mail_data_snapshot(&b_descriptor).await.unwrap().is_none());
    let absent_seed=store.seed_mail_data(&b_descriptor).await.unwrap();
    assert_eq!((absent_seed.account_rows,absent_seed.message_rows),(0,0));
    for sql in ["SELECT COUNT(*) FROM mail_data_accounts WHERE account_id=?1","SELECT COUNT(*) FROM mail_data_rows WHERE account_id=?1","SELECT COUNT(*) FROM mail_data_outbox WHERE account_id=?1"] {
        assert_eq!(q(sql).bind(&b).fetch_scalar_i64(&store.backend).await.unwrap(),0,"seeding absent source after erasure cannot recreate derived personal state");
    }
    assert_eq!(b_before.descriptor().account(),b);
    store.close().await;
}

#[tokio::test]
async fn sqlite_actual_all_source_dml_invalidation_and_projected_only_read(){
    all_source_dml(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}

#[tokio::test]
async fn postgres_actual_all_source_dml_invalidation_and_projected_only_read(){
    let Ok(uri)=std::env::var("MW_TEST_PG") else{eprintln!("SKIP actual PostgreSQL mail-data contract: MW_TEST_PG absent");return};
    let admin=sqlx::postgres::PgPoolOptions::new().max_connections(1).connect(&uri).await.unwrap();
    let schema=format!("mw_mail_data_{}",uuid::Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA \"{schema}\"")).execute(&admin).await.unwrap();
    let mut url=url::Url::parse(&uri).unwrap();url.query_pairs_mut().append_pair("options",&format!("-csearch_path={schema}"));
    let store=Store::open_postgres(url.as_str(),ServerKey::generate()).await.unwrap();
    all_source_dml(store).await;
    sqlx::query(&format!("DROP SCHEMA \"{schema}\" CASCADE")).execute(&admin).await.unwrap();admin.close().await;
}
