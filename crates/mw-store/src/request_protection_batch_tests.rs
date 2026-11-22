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
    assert_eq!(grouped.read.calls,serial.read.calls);assert_eq!(grouped.write.calls,serial.write.calls);
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
