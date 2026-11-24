use super::*;
use crate::{ServerKey, request_protection_model::WindowPolicy};
use serde_json::{Value, json};
fn policy() -> Policy {
    Policy { rate_mode: Mode::Enforce, adaptive_mode: Mode::Enforce,
        rate_limit: 3, rate_window_secs: 60,
        short: WindowPolicy { seconds:60, minimum_samples:2, absolute_anomalies:2, ratio_percent:50 },
        long: WindowPolicy { seconds:3600, minimum_samples:2, absolute_anomalies:2, ratio_percent:50 },
        normal_cap_per_bucket:1000, signals:63, throttle_secs:3, block_secs:300,
        decay_secs:3600, max_stage:2 }
}
async fn inventory(store: &Store) -> (Vec<Value>, Vec<Value>) {
    let states = q("SELECT subject_kind,state_json,updated_at,expires_at FROM request_protection_subject ORDER BY subject_key")
        .fetch_all(&store.backend).await.unwrap().into_iter().map(|r| json!({
            "kind":r.get_string("subject_kind"),"state":serde_json::from_str::<Value>(&r.get_string("state_json")).unwrap(),
            "updated":r.get_i64("updated_at"),"expires":r.get_i64("expires_at")
        })).collect();
    let mut events: Vec<_> = q("SELECT subject_kind,observed_at,reason,action,enforced,retry_after_secs,counts_json FROM request_protection_event")
        .fetch_all(&store.backend).await.unwrap().into_iter().map(|r| json!({
            "kind":r.get_string("subject_kind"),"at":r.get_i64("observed_at"),"reason":r.get_string("reason"),
            "action":r.get_string("action"),"enforced":r.get_i64("enforced"),"retry":r.get_i64("retry_after_secs"),
            "counts":serde_json::from_str::<Value>(&r.get_string("counts_json")).unwrap()
        })).collect();
    events.sort_by_key(Value::to_string); (states,events)
}
#[tokio::test]
async fn grouped_decisions_and_audits_equal_ordered_individual_transactions() {
    let key=ServerKey::generate();
    let serial=Store::open_in_memory(key.clone()).await.unwrap();
    let grouped=Store::open_in_memory(key).await.unwrap();
    let p=policy(); let mut changed=p.clone(); changed.rate_limit=1;
    let signals=[None,Some(Signal::Normal),None,Some(Signal::CredentialRejected),
        None,Some(Signal::Malformed),None,None,Some(Signal::ProbePath),None];
    let requests: Vec<_>=signals.iter().enumerate().map(|(i,signal)| ProtectionObservation {
        kind:"account",value:if i%3==0 {"account-b"}else{"account-a"},
        policy:if i==7 {&changed}else{&p},now:1000+i as i64,signal:*signal,
        source:ProtectionSource {country:Some("PT"),ip:Some("192.0.2.10")},
    }).collect();
    let mut expected=Vec::new();
    for r in &requests { expected.push(serial.request_protection_with_source(r.kind,r.value,r.policy,r.now,r.signal,r.source).await.unwrap()); }
    let actual=grouped.request_protection_sqlite_batch(&requests).await.unwrap();
    assert_eq!(actual.len(),expected.len());
    for (a,b) in actual.iter().zip(&expected) {
        assert_eq!(a.decision,b.decision); assert_eq!(a.subject_key,b.subject_key);
        assert_eq!(serde_json::to_value(&a.snapshot).unwrap(),serde_json::to_value(&b.snapshot).unwrap());
        assert!(!a.revision.is_empty());
    }
    assert_eq!(inventory(&serial).await,inventory(&grouped).await);
    assert!(actual.iter().any(|out| out.decision.enforced),"real refusal must be exercised");
}
#[tokio::test]
async fn grouped_storage_failure_rolls_back_earlier_budget_and_audit_changes() {
    let store=Store::open_in_memory(ServerKey::generate()).await.unwrap();let p=policy();
    for value in ["first","last"] {store.request_protection("account",value,&p,1000,None).await.unwrap();}
    let failing=store.request_protection_subject("account","last").unwrap();
    q(Box::leak(format!("CREATE TRIGGER fail_group_update BEFORE UPDATE ON request_protection_subject WHEN NEW.subject_key='{failing}' BEGIN SELECT RAISE(ABORT,'injected group storage fault'); END").into_boxed_str()))
        .execute(&store.backend).await.unwrap();
    let before=inventory(&store).await;
    let requests=[ProtectionObservation {kind:"account",value:"first",policy:&p,now:1001,signal:Some(Signal::ProbePath),source:ProtectionSource::default()},
        ProtectionObservation {kind:"account",value:"last",policy:&p,now:1001,signal:None,source:ProtectionSource::default()}];
    assert!(store.request_protection_sqlite_batch(&requests).await.is_err());
    assert_eq!(inventory(&store).await,before,"neither earlier charge nor transition audit may survive");
}
#[tokio::test]
async fn grouped_off_empty_and_invalid_batches_do_not_start_transactions() {
    let store=Store::open_in_memory(ServerKey::generate()).await.unwrap();let mut off=policy();
    off.rate_mode=Mode::Off;off.adaptive_mode=Mode::Off;
    let r=ProtectionObservation {kind:"account",value:"idle",policy:&off,now:1000,signal:None,source:ProtectionSource::default()};
    let (result,counters)=crate::pool_telemetry::measure_queries(store.request_protection_sqlite_batch(&[r])).await;
    assert_eq!(result.unwrap().len(),1); assert_eq!(counters.other.calls,0);assert_eq!(counters.write.calls,0);assert_eq!(counters.read.calls,0);
    assert!(store.request_protection_sqlite_batch(&[]).await.unwrap().is_empty());
    assert!(store.request_protection_sqlite_batch(&vec![r;65]).await.is_err());
    let bad=ProtectionObservation {now:-1,..r};assert!(store.request_protection_sqlite_batch(&[bad]).await.is_err());
    assert!(inventory(&store).await.0.is_empty());
}
#[tokio::test]
async fn grouped_admissions_use_one_writer_transaction_instead_of_eight() {
    let store=Store::open_in_memory(ServerKey::generate()).await.unwrap();let mut p=policy();p.rate_limit=1000;
    store.request_protection("account","warm",&p,1000,None).await.unwrap();
    let r=ProtectionObservation {kind:"account",value:"warm",policy:&p,now:1001,signal:None,source:ProtectionSource::default()};
    let (result,grouped)=crate::pool_telemetry::measure_queries(store.request_protection_sqlite_batch(&[r;8])).await;
    assert_eq!(result.unwrap().len(),8);
    let (_,serial)=crate::pool_telemetry::measure_queries(async {for _ in 0..8 {store.request_protection_with_source(r.kind,r.value,r.policy,r.now,r.signal,r.source).await.unwrap();}}).await;
    assert_eq!(grouped.other.calls,1);assert_eq!(serial.other.calls,8);
    // Set-based (t50hp-e8): one SELECT of the rows and one UPDATE for the group,
    // against a lock UPDATE, a SELECT and an UPDATE per observation.
    assert_eq!((grouped.read.calls,grouped.write.calls),(1,1),"{grouped:?}");
    assert_eq!((serial.read.calls,serial.write.calls),(8,16),"{serial:?}");
}
#[tokio::test]
async fn set_based_group_of_64_over_many_subjects_is_two_statements_and_equals_the_serial_path() {
    let key=ServerKey::generate();
    let serial=Store::open_in_memory(key.clone()).await.unwrap();
    let grouped=Store::open_in_memory(key).await.unwrap();
    let mut p=policy();p.rate_limit=20;
    let values:Vec<String>=(0..16).map(|i| format!("subject-{i}")).collect();
    for store in [&serial,&grouped] {
        for value in &values {store.request_protection("account",value,&p,1000,None).await.unwrap();}
    }
    let signals=[None,Some(Signal::Normal),Some(Signal::Malformed),None,Some(Signal::CredentialRejected),None,Some(Signal::ProbePath),None];
    let requests:Vec<_>=(0..64).map(|i| ProtectionObservation {
        kind:"account",value:&values[(i*7)%16],policy:&p,now:1001+(i as i64)/8,signal:signals[i%8],
        source:ProtectionSource{country:None,ip:Some("192.0.2.44")},
    }).collect();
    let mut expected=Vec::new();
    for r in &requests {expected.push(serial.request_protection_with_source(r.kind,r.value,r.policy,r.now,r.signal,r.source).await.unwrap());}
    let (actual,sql)=crate::pool_telemetry::measure_queries(grouped.request_protection_sqlite_batch_with_state(&requests)).await;
    let actual=actual.unwrap();
    for ((a,stored),b) in actual.iter().zip(&expected) {
        assert_eq!(a.decision,b.decision);
        assert_eq!(serde_json::to_value(&a.snapshot).unwrap(),serde_json::to_value(&b.snapshot).unwrap());
        assert_eq!(&stored.as_ref().unwrap().state.snapshot().rate_count,&b.snapshot.rate_count);
    }
    assert_eq!(inventory(&serial).await,inventory(&grouped).await);
    let transitions=inventory(&grouped).await.1.len() as u64;
    assert!(transitions>0,"transitions must be exercised");
    // Rows: one SELECT and one UPDATE; each transition adds its audit statements.
    assert_eq!(sql.other.calls,1,"{sql:?}");
    assert!(sql.read.calls<=1+transitions,"{sql:?}");
    assert!(sql.write.calls<=1+5*transitions,"{sql:?}");
    // Every committed revision is the one the outcome reports.
    for (outcome,stored) in &actual {
        let row=grouped.request_protection_load(&outcome.subject_key,"account",1010).await.unwrap().unwrap();
        let last=actual.iter().rev().find(|(o,_)| o.subject_key==outcome.subject_key).unwrap();
        assert_eq!(row.revision,last.0.revision);assert_eq!(stored.as_ref().unwrap().revision,row.revision);
    }
}
#[tokio::test]
async fn a_group_with_a_new_subject_keeps_the_capacity_checked_row_allocation() {
    let key=ServerKey::generate();
    let serial=Store::open_in_memory(key.clone()).await.unwrap();
    let grouped=Store::open_in_memory(key).await.unwrap();
    let p=policy();
    for store in [&serial,&grouped] {store.request_protection("account","old",&p,1000,None).await.unwrap();}
    let requests=[ProtectionObservation{kind:"account",value:"old",policy:&p,now:1001,signal:None,source:ProtectionSource::default()},
        ProtectionObservation{kind:"account",value:"new",policy:&p,now:1001,signal:None,source:ProtectionSource::default()},
        ProtectionObservation{kind:"account",value:"old",policy:&p,now:1002,signal:None,source:ProtectionSource::default()}];
    for r in &requests {serial.request_protection_with_source(r.kind,r.value,r.policy,r.now,r.signal,r.source).await.unwrap();}
    let out=grouped.request_protection_sqlite_batch(&requests).await.unwrap();
    assert_eq!(out.len(),3);
    assert_eq!(inventory(&serial).await,inventory(&grouped).await);
    // A subject of another kind under the same key is refused, set-based too.
    let wrong=[ProtectionObservation{kind:"admin",value:"old",policy:&p,now:1003,signal:None,source:ProtectionSource::default()}];
    let admin_key=grouped.request_protection_subject("admin","old").unwrap();
    q("UPDATE request_protection_subject SET subject_key=?1 WHERE subject_key=?2").bind(&admin_key)
        .bind(grouped.request_protection_subject("account","old").unwrap()).execute(&grouped.backend).await.unwrap();
    assert!(matches!(grouped.request_protection_sqlite_batch(&wrong).await,Err(StoreError::Corrupt(_))));
}
#[test]
fn precomputed_subject_hmac_equals_the_plain_hmac() {
    let key=ServerKey::generate();
    let subkey=key.derive("mw-request-protection-subject-v1");
    let keyer=SubjectKeyer::new(subkey);
    for (kind,value,message) in [("account","a","account\0a"),("client","::ffff:192.0.2.1","client\0192.0.2.1"),
        ("client","probe-path:2001:DB8::1","client\0probe-path:2001:db8::1"),("admin","root","admin\0root")] {
        let plain:String=crate::seal::hmac_sha256(&subkey,message.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(keyer.subject(kind,value).unwrap(),plain,"{kind} {value}");
    }
    assert!(keyer.subject("other","x").is_err()&&keyer.subject("account","").is_err());
    assert_eq!(format!("{keyer:?}"),"SubjectKeyer(..)");
}
#[tokio::test]
async fn grouped_sqlcipher_commit_survives_encrypted_reopen() {
    use crate::encryption::{EncryptionConfig,EncryptionMode,DbKey};
    let directory=tempfile::tempdir().unwrap();let path=directory.path().join("group.db");
    let cfg=EncryptionConfig::with_key(EncryptionMode::Required,DbKey::from_bytes([37;32]));
    let key=ServerKey::generate();let store=Store::open_with_encryption(path.to_str().unwrap(),key.clone(),&cfg).await.unwrap();let p=policy();
    let r=ProtectionObservation {kind:"account",value:"persisted",policy:&p,now:1000,signal:None,source:ProtectionSource::default()};
    let outcomes=store.request_protection_sqlite_batch(&[r;4]).await.unwrap();assert!(outcomes[3].decision.enforced);
    let before=inventory(&store).await;store.close().await;drop(store);
    let header=std::fs::read(&path).unwrap();assert_ne!(&header[..16],b"SQLite format 3\0");
    let reopened=Store::open_with_encryption(path.to_str().unwrap(),key,&cfg).await.unwrap();assert_eq!(inventory(&reopened).await,before);reopened.close().await;
}
#[tokio::test]
async fn grouped_state_is_the_committed_row_and_load_reads_it_back() {
    let store=Store::open_in_memory(ServerKey::generate()).await.unwrap();let p=policy();
    let r=ProtectionObservation {kind:"account",value:"seeded",policy:&p,now:1000,signal:None,source:ProtectionSource::default()};
    let out=store.request_protection_sqlite_batch_with_state(&[r,r,ProtectionObservation{signal:Some(Signal::Ignored),..r}]).await.unwrap();
    assert!(out[2].1.is_none(),"an ignored observation touches no state");
    let committed=out[1].1.clone().unwrap();
    let key=store.request_protection_subject("account","seeded").unwrap();
    let loaded=store.request_protection_load(&key,"account",1000).await.unwrap().unwrap();
    assert_eq!(loaded.state,committed.state);assert_eq!(loaded.expires_at,committed.expires_at);
    assert_eq!(loaded.revision,committed.revision);assert_eq!(out[1].0.revision,committed.revision);
    assert!(store.request_protection_load(&key,"admin",1000).await.is_err(),"kind is validated");
    let other=store.request_protection_subject("account","absent").unwrap();
    assert!(store.request_protection_load(&other,"account",1000).await.unwrap().is_none());
    // An expired row reads as the default state, as the transactional path treats it.
    let expired=store.request_protection_load(&key,"account",committed.expires_at).await.unwrap().unwrap();
    assert_eq!(expired.state,crate::request_protection_model::State::default());
}
#[tokio::test]
async fn in_memory_evaluator_matches_transactional_decisions_and_rows() {
    // Deterministic pseudo-random sequences over subjects, signals, policies and time.
    let store=Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let mut variants=vec![policy()];let mut v=policy();v.rate_limit=1;variants.push(v);
    let mut v=policy();v.max_stage=1;v.signals=5;variants.push(v);
    let mut v=policy();v.adaptive_mode=Mode::Observe;variants.push(v);
    let signals=[None,None,Some(Signal::Normal),Some(Signal::Malformed),Some(Signal::CredentialRejected),
        Some(Signal::ProbePath),Some(Signal::BotUserAgent),Some(Signal::Protocol),Some(Signal::Ignored)];
    let mut seed=0x9e37_79b9_7f4a_7c15_u64;let mut next=|n:u64| {seed^=seed<<13;seed^=seed>>7;seed^=seed<<17;seed%n};
    let mut local:std::collections::HashMap<String,(crate::request_protection_model::State,i64)>=Default::default();
    let mut now=10_000i64;let mut refused=0;let mut transitions=0;
    for _ in 0..600 {
        now+=next(40) as i64;if next(50)==0 {now+=4000;}
        let kind=["client","account"][next(2) as usize];
        let value=if kind=="client" {["192.0.2.1","192.0.2.2","2001:db8::1"][next(3) as usize]}else{["a","b"][next(2) as usize]};
        let p=&variants[next(variants.len() as u64) as usize];let signal=signals[next(signals.len() as u64) as usize];
        let source=ProtectionSource{country:Some("PT"),ip:None};
        let expected=store.request_protection_with_source(kind,value,p,now,signal,source).await.unwrap();
        let Some(prepared)=store.request_protection_prepare(kind,value,p,now,signal,source).unwrap() else {
            assert!(expected.subject_key.is_empty());continue;
        };
        let (state,expires)=local.entry(prepared.subject_key().to_owned()).or_default();
        let step=evaluate_protection(state,expires,&prepared,p,now,signal,source);
        assert_eq!(step.decision,expected.decision);
        let row=store.request_protection_load(prepared.subject_key(),kind,now).await.unwrap().unwrap();
        assert_eq!(&row.state,state);assert_eq!(row.expires_at,*expires);
        refused+=usize::from(step.decision.enforced);transitions+=usize::from(step.transition);
    }
    assert!(refused>0&&transitions>0,"refusals {refused} transitions {transitions} must be exercised");
}
#[tokio::test]
async fn witness_location_is_private_in_memory_and_a_file_beside_a_database() {
    let memory=Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(matches!(memory.protection_location(),owner::Location::Private));
    let directory=tempfile::tempdir().unwrap();let path=directory.path().join("located.db");
    let file=Store::open(path.to_str().unwrap(),ServerKey::generate()).await.unwrap();
    assert!(matches!(file.protection_location(),owner::Location::File(at) if at.file_name()==Some(std::ffi::OsStr::new("located.db"))));
    file.request_protection_join().await.unwrap();
    assert!(directory.path().join("located.db.mw-request-protection.lock").exists());
}
