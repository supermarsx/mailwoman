// Included in cache::tests: these use the actual encrypted Store write paths.
fn body_batch_message<'a>(account: &'a str, mailbox: &'a str, uid: u32, raw: &[u8], envelope: &'a [u8]) -> MessageUpsert<'a> {
    MessageUpsert { account_id: account, mailbox_id: mailbox, uid, uidvalidity: 1,
        message_id: None, thread_id: None, internaldate: Some("2026-11-01T00:00:00Z"),
        size: raw.len() as u64, flags_json: "[]", envelope: Some(envelope), blob_ref: None }
}

async fn body_batch_parity(s: &Store) {
    let account = seed_account(s).await;
    let batch_box = seed_mailbox(s, &account, "Batch", 1).await;
    let scalar_box = seed_mailbox(s, &account, "Scalar", 1).await;
    for mailbox in [&batch_box, &scalar_box] {
        s.update_mailbox_counts(mailbox, 51, 0, 50, 50).await.unwrap();
    }
    let raw: Vec<Vec<u8>> = (1..=50).map(|uid| format!("From: a@fixture.invalid\r\nSubject: row {uid}\r\n\r\nowned body {uid}\r\n").into_bytes()).collect();
    let envelope: Vec<Vec<u8>> = (1..=50).map(|uid| serde_json::to_vec(&serde_json::json!({"subject":format!("row {uid}"),"hasAttachment":false,"bodyValues":{"part":{"value":"kept"}}})).unwrap()).collect();
    let message_ids:Vec<String>=(1..=50).map(|uid|format!("<batch-{uid}@fixture.invalid>")).collect();
    let inputs: Vec<_> = raw.iter().zip(&envelope).enumerate().map(|(i,(raw,env))| (MessageUpsert {message_id:Some(message_ids[i].as_str()),..body_batch_message(&account, &batch_box, i as u32+1, raw, env)}, raw.as_slice())).collect();
    let batch = s.upsert_message_bodies_batch(&inputs).await.unwrap();
    assert_eq!(batch.len(), 50);
    for (i, ((id, existed), raw)) in batch.iter().zip(&raw).enumerate() {
        assert!(!existed);
        let scalar = s.upsert_message_with_body(&MessageUpsert {message_id:Some(message_ids[i].as_str()),..body_batch_message(&account, &scalar_box, i as u32+1, raw, &envelope[i])}, raw).await.unwrap();
        let left = s.get_message(id).await.unwrap();
        let right = s.get_message(&scalar.0).await.unwrap();
        assert_eq!((left.uid,left.uidvalidity,left.size,left.flags_json,left.blob_ref.clone()), (right.uid,right.uidvalidity,right.size,right.flags_json,right.blob_ref));
        assert_eq!(s.get_envelope(id).await.unwrap(), s.get_envelope(&scalar.0).await.unwrap());
        assert_eq!(s.get_body_limited(&account,left.blob_ref.as_ref().unwrap(),raw.len()).await.unwrap().as_deref(),Some(raw.as_slice()));
        let header = s.get_account_header_envelope(&account,id).await.unwrap().unwrap();
        let reference = s.get_account_header_envelope(&account,&scalar.0).await.unwrap().unwrap();
        assert!(header.projected && reference.projected);
        assert_eq!(header.envelope, reference.envelope);
        assert!(!String::from_utf8(header.envelope).unwrap().contains("bodyValues"));
        // Real encrypted bytes, not plaintext inserted under a bulk shortcut.
        let sealed = q("SELECT envelope_json FROM messages WHERE stable_id=?1").bind(id).fetch_one(&s.backend).await.unwrap().get_blob("envelope_json");
        assert_ne!(sealed, envelope[i]);
        let sealed_body=q("SELECT sealed_bytes FROM bodies WHERE blob_ref=?1").bind(left.blob_ref.as_ref().unwrap()).fetch_one(&s.backend).await.unwrap().get_blob("sealed_bytes");
        assert_ne!(sealed_body.as_slice(),raw.as_slice());
    }
    let retry = s.upsert_message_bodies_batch(&[inputs[0].clone(), inputs[0].clone()]).await.unwrap();
    assert_eq!(retry,vec![(batch[0].0.clone(),true),(batch[0].0.clone(),true)]);
    let reset=s.upsert_message_bodies_batch(&[(MessageUpsert {uidvalidity:2,..inputs[0].0.clone()},inputs[0].1)]).await.unwrap();
    assert_eq!(reset,vec![(batch[0].0.clone(),true)],"UIDVALIDITY identity matching retains the stable ID in batch writes");
    assert_eq!(s.get_message(&batch[0].0).await.unwrap().uidvalidity,2);
    assert_eq!(s.list_message_ids(&batch_box,100,0).await.unwrap().len(),50);
    for mailbox in [&batch_box,&scalar_box] {
        let row = s.get_mailbox(mailbox).await.unwrap();
        assert_eq!((row.total,row.unread),(50,50),"source mailbox counters are not incremented by cache upsert");
    }
    assert_eq!(s.current_state(&account,"Email").await.unwrap(),0);
    assert_eq!(s.current_state(&account,"Mailbox").await.unwrap(),0);
    assert_eq!(q("SELECT COUNT(*) FROM mail_data_rows v JOIN messages m ON m.stable_id=v.stable_id WHERE v.account_id<>m.account_id OR v.cache_revision<>m.cache_revision OR v.content_revision<>m.content_revision").fetch_scalar_i64(&s.backend).await.unwrap(),0);
    let descriptor=s.mail_data_descriptor(&account,&batch.iter().map(|r|r.0.clone()).collect::<Vec<_>>()).unwrap();
    let fence=s.mail_data_snapshot(&descriptor).await.unwrap().unwrap();
    assert!(fence.publishable() && s.validate_mail_data_schema(&fence).await.unwrap());
}

// Snapshot derived publication identity as well as source counts, so a failed
// second row cannot leave a committed first row or advance coherent epochs.
async fn body_batch_stamp(s: &Store, account: &str) -> serde_json::Value {
    let mut counts=Vec::new();
    for sql in ["SELECT COUNT(*) FROM messages", "SELECT COUNT(*) FROM bodies", "SELECT COUNT(*) FROM message_list_envelope", "SELECT COUNT(*) FROM mail_data_rows", "SELECT COUNT(*) FROM changes"] {
        counts.push(q(sql).fetch_scalar_i64(&s.backend).await.unwrap());
    }
    let epochs:Vec<_>=q("SELECT scope_kind,scope_id,incarnation,version FROM authority_epoch ORDER BY scope_kind,scope_id").fetch_all(&s.backend).await.unwrap().into_iter().map(|r|(r.get_string("scope_kind"),r.get_string("scope_id"),r.get_string("incarnation"),r.get_i64("version"))).collect();
    let data:Vec<_>=q("SELECT incarnation,focus_revision,policy_revision FROM mail_data_accounts WHERE account_id=?1").bind(account).fetch_all(&s.backend).await.unwrap().into_iter().map(|r|(r.get_string("incarnation"),r.get_string("focus_revision"),r.get_string("policy_revision"))).collect();
    let outbox=q("SELECT revision FROM mail_data_outbox WHERE account_id=?1").bind(account).fetch_opt_scalar_string(&s.backend).await.unwrap();
    let authority_outbox:Vec<_>=q("SELECT scope_kind,scope_id,incarnation,target_version,notified_version FROM authority_publication_outbox ORDER BY scope_kind,scope_id").fetch_all(&s.backend).await.unwrap().into_iter().map(|r|(r.get_string("scope_kind"),r.get_string("scope_id"),r.get_string("incarnation"),r.get_i64("target_version"),r.get_i64("notified_version"))).collect();
    serde_json::json!({"counts":counts,"epochs":epochs,"data":data,"outbox":outbox,"authorityOutbox":authority_outbox})
}

async fn body_batch_rollback(s: &Store) {
    let account=seed_account(s).await;
    let mailbox=seed_mailbox(s,&account,"INBOX",1).await;
    let before=body_batch_stamp(s,&account).await;
    match s.backend.dialect() {
        Dialect::Sqlite => {q("CREATE TRIGGER body_batch_failure BEFORE INSERT ON messages WHEN NEW.uid=2 BEGIN SELECT RAISE(ABORT,'synthetic middle-row failure'); END").execute(&s.backend).await.unwrap();}
        Dialect::Postgres => {
            q("CREATE FUNCTION body_batch_failure_fn() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.uid=2 THEN RAISE EXCEPTION 'synthetic middle-row failure'; END IF; RETURN NEW; END $$").execute(&s.backend).await.unwrap();
            q("CREATE TRIGGER body_batch_failure BEFORE INSERT ON messages FOR EACH ROW EXECUTE FUNCTION body_batch_failure_fn()").execute(&s.backend).await.unwrap();
        }
    }
    let raw=b"one owned raw body"; let env=br#"{"subject":"rollback"}"#;
    let inputs=[(body_batch_message(&account,&mailbox,1,raw,env),raw.as_slice()),(body_batch_message(&account,&mailbox,2,raw,env),raw.as_slice())];
    assert!(s.upsert_message_bodies_batch(&inputs).await.is_err());
    assert_eq!(body_batch_stamp(s,&account).await,before,"message/body/projection/epoch/outbox rollback is atomic");
    match s.backend.dialect() {
        Dialect::Sqlite => {q("DROP TRIGGER body_batch_failure").execute(&s.backend).await.unwrap();}
        Dialect::Postgres => {
            q("DROP TRIGGER body_batch_failure ON messages").execute(&s.backend).await.unwrap();
            q("DROP FUNCTION body_batch_failure_fn()").execute(&s.backend).await.unwrap();
        }
    }
    let ok=s.upsert_message_bodies_batch(&inputs).await.unwrap();
    assert_eq!(ok.len(),2); assert!(ok.iter().all(|v|!v.1));
}

async fn body_batch_cancel(s: &Store) {
    let account=seed_account(s).await;
    let mailbox=seed_mailbox(s,&account,"INBOX",1).await;
    let before=body_batch_stamp(s,&account).await;
    let entered=std::sync::Arc::new(tokio::sync::Notify::new());
    let release=std::sync::Arc::new(tokio::sync::Notify::new());
    let store=s.clone();let a=account.clone();
    let worker=tokio::spawn(message_body_batch::ROW_PAUSE.scope((entered.clone(),release),async move {
        let raw=b"cancel while first real row is uncommitted";let env=br#"{"subject":"cancel"}"#;
        store.upsert_message_bodies_batch(&[(body_batch_message(&a,&mailbox,1,raw,env),raw.as_slice()),(body_batch_message(&a,&mailbox,2,raw,env),raw.as_slice())]).await
    }));
    tokio::time::timeout(std::time::Duration::from_secs(5),entered.notified()).await.unwrap();
    worker.abort();assert!(worker.await.unwrap_err().is_cancelled());
    let after=tokio::time::timeout(std::time::Duration::from_secs(5),body_batch_stamp(s,&account)).await.unwrap();
    assert_eq!(after,before,"actual pending transaction must roll back after cancellation");
}

async fn body_batch_admission(s: &Store) {
    let account=seed_account(s).await;let mailbox=seed_mailbox(s,&account,"INBOX",1).await;
    let other=seed_account(s).await;let foreign=seed_mailbox(s,&other,"Foreign",1).await;
    let raw=b"bounded";let env=br#"{"subject":"bounded"}"#;
    let one=(body_batch_message(&account,&mailbox,1,raw,env),raw.as_slice());
    let charge=Store::validate_message_body_batch(std::slice::from_ref(&one)).unwrap();
    assert!(charge>raw.len(),"valid nonempty batches report a real conservative charge for caller chunking");
    let longer=b"bounded longer raw";
    let grown=Store::validate_message_body_batch(&[(one.0.clone(),longer.as_slice())]).unwrap();
    assert_eq!(grown-charge,2*(longer.len()-raw.len()),"owned input plus possible body ciphertext grow together before writes");
    let huge="x".repeat(8*1024*1024+1);
    let mut rejected=vec![vec![one.clone();101],vec![(MessageUpsert {flags_json:&huge,..one.0.clone()},raw.as_slice())],vec![one.clone(),(body_batch_message(&account,&foreign,2,raw,env),raw.as_slice())],vec![(one.0.clone(),huge.as_bytes())],vec![(MessageUpsert {envelope:Some(huge.as_bytes()),..one.0.clone()},raw.as_slice())]];
    for field in 0..6 {
        let mut message=one.0.clone();
        match field {0=>message.account_id=&huge,1=>message.mailbox_id=&huge,2=>message.message_id=Some(&huge),3=>message.thread_id=Some(&huge),4=>message.internaldate=Some(&huge),5=>message.blob_ref=Some(&huge),_=>unreachable!()}
        rejected.push(vec![(message,raw.as_slice())]);
    }
    for rows in rejected {
        assert!(Store::validate_message_body_batch(&rows).is_err());
        let (result,queries)=crate::pool_telemetry::measure_queries(s.upsert_message_bodies_batch(&rows)).await;
        assert!(result.is_err());assert_eq!(queries.read.calls+queries.write.calls+queries.other.calls,0,"admission precedes crypto/SQL/transactions");
    }
    assert!(matches!(s.upsert_message_bodies_batch(&[(body_batch_message(&account,&foreign,1,raw,env),raw.as_slice())]).await,Err(StoreError::NotFound)));
    assert_eq!(q("SELECT COUNT(*) FROM bodies").fetch_scalar_i64(&s.backend).await.unwrap(),0,"foreign mailbox failure rolls back its preceding body insert");
    assert_eq!(Store::validate_message_body_batch(&[]).unwrap(),0);
    assert!(s.upsert_message_bodies_batch(&[]).await.unwrap().is_empty());
}

async fn body_batch_expiry(s: &Store) {
    let account=seed_account(s).await;let mailbox=seed_mailbox(s,&account,"INBOX",1).await;
    let raw=b"immutable expired source";let env=br#"{"subject":"expiry","bodyValues":{"part":{"value":"secret body"}}}"#;
    let message=body_batch_message(&account,&mailbox,1,raw,env);
    let first=s.upsert_message_with_body(&message,raw).await.unwrap().0;
    s.register_refetchable_cached_body(&account,&first).await.unwrap();
    q("UPDATE message_body_cache_lease SET last_used_at=1 WHERE stable_id=?1").bind(&first).execute(&s.backend).await.unwrap();
    let report=s.expire_cached_bodies(crate::body_cache_expiry::CachedBodyExpiryOptions{max_run_ms:2000,..Default::default()},Default::default(),vec![account.clone()]).await.unwrap();
    assert_eq!(report.detached_messages,1,"{}",report.status);
    let cleaned=s.get_envelope(&first).await.unwrap();
    let retry=s.upsert_message_bodies_batch(&[(message.clone(),raw.as_slice())]).await.unwrap();
    assert_eq!(retry,vec![(first.clone(),true)]);
    assert!(s.get_message(&first).await.unwrap().blob_ref.is_none());assert_eq!(s.get_envelope(&first).await.unwrap(),cleaned);
    // A changed first row followed by the old immutable source must observe
    // each preceding transaction-local row, not a stale pre-batch expiry hint.
    let changed=b"changed source";
    let rows=[(MessageUpsert{size:changed.len() as u64,..message.clone()},changed.as_slice()),(message,raw.as_slice())];
    let retry=s.upsert_message_bodies_batch(&rows).await.unwrap();assert!(retry.iter().all(|r|r.0==first));
    let blob=s.get_message(&first).await.unwrap().blob_ref.unwrap();
    assert_eq!(s.get_body(&blob).await.unwrap().unwrap().as_slice(),raw.as_slice());
}

async fn body_batch_pg(which:u8) {
    use sqlx::Connection;
    let dsn=std::env::var("MW_TEST_PG").expect("owned MW_TEST_PG required");
    let schema=format!("message_body_batch_{}",uuid::Uuid::new_v4().simple());
    let mut admin=sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&mut admin).await.unwrap();
    let mut url=url::Url::parse(&dsn).unwrap();url.query_pairs_mut().append_pair("options",&format!("-csearch_path={schema}"));
    let s=Store::open(url.as_str(),ServerKey::generate()).await.unwrap();let worker=s.clone();
    let result=tokio::spawn(async move {match which {0=>body_batch_parity(&worker).await,1=>body_batch_rollback(&worker).await,2=>body_batch_cancel(&worker).await,3=>body_batch_admission(&worker).await,4=>body_batch_expiry(&worker).await,_=>unreachable!()}}).await;
    s.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&mut admin).await.unwrap();
    result.unwrap();
}

#[tokio::test] async fn message_body_batch_parity_sqlite(){body_batch_parity(&store().await).await;}
#[tokio::test] async fn message_body_batch_rollback_sqlite(){body_batch_rollback(&store().await).await;}
#[tokio::test] async fn message_body_batch_cancel_sqlite(){body_batch_cancel(&store().await).await;}
#[tokio::test] async fn message_body_batch_admission_sqlite(){body_batch_admission(&store().await).await;}
#[tokio::test] async fn message_body_batch_expiry_sqlite(){body_batch_expiry(&store().await).await;}
#[tokio::test] #[ignore="requires owned MW_TEST_PG; focused gate must include ignored"] async fn message_body_batch_parity_postgres(){body_batch_pg(0).await;}
#[tokio::test] #[ignore="requires owned MW_TEST_PG; focused gate must include ignored"] async fn message_body_batch_rollback_postgres(){body_batch_pg(1).await;}
#[tokio::test] #[ignore="requires owned MW_TEST_PG; focused gate must include ignored"] async fn message_body_batch_cancel_postgres(){body_batch_pg(2).await;}
#[tokio::test] #[ignore="requires owned MW_TEST_PG; focused gate must include ignored"] async fn message_body_batch_admission_postgres(){body_batch_pg(3).await;}
#[tokio::test] #[ignore="requires owned MW_TEST_PG; focused gate must include ignored"] async fn message_body_batch_expiry_postgres(){body_batch_pg(4).await;}
