//! Real Store/MIME/Index prepared DATA controls. The standalone source snapshot
//! below substitutes only for the endpoint's not-yet-integrated joint auth SQL;
//! these tests do NOT establish HTTP authorization or a zero-SQL endpoint.
use super::*;
use crate::backend::{MessageRef,RawMailboxRef,RawMessage};
use mw_store::{Store,ServerKey,NewAccount,AccountKind,Credentials,MailboxUpsert};

struct Fixture {engine:Engine,account:String,other:String,ids:Vec<String>}
async fn owner(store:&Store,name:&str)->String {
    store.create_account(&NewAccount {kind:AccountKind::Imap,host:"mail-data.invalid",port:993,tls:"implicit",username:name,sync_policy_json:"{}"},&Credentials {username:name.into(),password:"synthetic-only".into()}).await.unwrap()
}
async fn drained(engine:&Engine) {
    tokio::time::timeout(Duration::from_secs(10),async {
        while engine.index_publication_status().pending_jobs!=0 {tokio::time::sleep(Duration::from_millis(1)).await;}
    }).await.expect("actual accepted publication jobs drain");
}
async fn fixture()->Fixture {
    let store=Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account=owner(&store,"owner@mail-data.invalid").await;
    let other=owner(&store,"other@mail-data.invalid").await;
    let mut folders=Vec::new();
    for (name,role) in [("Inbox","inbox"),("Sent","sent")] {
        folders.push(store.upsert_mailbox(&MailboxUpsert {account_id:&account,name,role:Some(role),uidvalidity:1,uidnext:51,highestmodseq:0,total:0,unread:0,parent_id:None}).await.unwrap());
    }
    let engine=Engine::with_search_and_publication_options(store,Arc::new(mw_search::Index::open_in_ram().unwrap()),crate::PublicationOptions {collection_window:Duration::ZERO,..Default::default()}).unwrap();
    assert!(engine.configure_prepared_mail(PreparedMailOptions {enabled:true,..Default::default()}));
    let mut ids=Vec::new();
    for n in 0..50u32 {
        let folder=usize::from(n>=45);
        let message=RawMessage {message_ref:MessageRef::Imap {mailbox:RawMailboxRef {name:if folder==0{"Inbox"}else{"Sent"}.into(),uidvalidity:1},uidvalidity:1,uid:n+1},raw:format!("Message-ID: <prepared-{n}@mail-data.invalid>\r\nFrom: Sender {} <sender{}@mail-data.invalid>\r\nTo: owner@mail-data.invalid\r\nSubject: Prepared {n}\r\nDate: Mon, 02 Nov 2026 00:00:00 +0000\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nactualpreparedbody{n}",n%25,n%25).into_bytes(),flags:Vec::new(),internaldate:Some("2026-11-02T00:00:00Z".into())};
        ids.push(engine.ingest_import(&account,&folders[folder],&message).await.unwrap());
    }
    drained(&engine).await;
    Fixture {engine,account,other,ids}
}
fn args(account:&str,ids:&[String])->Value {json!({"accountId":account,"ids":ids,"properties":PROPERTIES,"fetchHTMLBodyValues":false,"fetchTextBodyValues":false,"fetchAllBodyValues":false})}
fn index_counts(engine:&Engine)->(u64,u64,u64) {
    let index=engine.search();(index.list_projection_search_count(),index.list_projection_doc_read_count(),index.doc_fetch_count())
}
async fn observed(f:&Fixture,candidate:&PreparedMetadataCandidate)->MailDataFence {
    f.engine.store().mail_data_snapshot(candidate.descriptor()).await.unwrap().unwrap()
}

#[tokio::test]
async fn actual_fifty_rows_ready_bytes_equal_ordinary_and_hit_without_index_work() {
    let f=fixture().await;let a=args(&f.account,&f.ids);
    let ordinary=f.engine.handle_jmap(&f.account,&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/get",a.clone(),"ordinary"]]})).await;
    let cold=f.engine.prepare_metadata_candidate(&f.account,&a).await.expect("current ingested header projections qualify");
    let joint=observed(&f,&cold).await;
    let value=f.engine.finish_prepared_metadata_value(&cold,&joint).unwrap();
    assert_eq!(ordinary["methodResponses"][0][1],value,"all36 actual properties must equal ordinary reader output");
    let before=index_counts(&f.engine);
    let (hit,queries)=mw_store::pool_telemetry::measure_queries(f.engine.prepare_metadata_candidate(&f.account,&a)).await;
    let hit=hit.unwrap();
    assert_eq!((queries.read.calls,queries.write.calls,queries.other.calls,queries.acquire.calls),(0,0,0,0),"ready preparation is DATA lookup only; final joint source SQL below is separately required");
    assert_eq!(index_counts(&f.engine),before,"ready lookup performs no existing Index list/doc operations");
    eprintln!("prepared warm Engine-only lookup: reads={} writes={} other={} acquires={} indexListSearches=0 indexListDocReads=0 indexDocFetches=0; final joint proof and background tasks excluded",queries.read.calls,queries.write.calls,queries.other.calls,queries.acquire.calls);
    let joint=observed(&f,&hit).await;
    let call="quoted\"\\\ncall";
    let bytes=f.engine.finish_prepared_metadata(hit,&joint,call).unwrap().into_bytes();
    let response:Value=serde_json::from_slice(&bytes).unwrap();
    assert_eq!(response["methodResponses"][0][1],value);
    assert_eq!(response["methodResponses"][0][2],call);
    assert_eq!(response["sessionState"],joint.session_state());
    assert_eq!(index_counts(&f.engine),before,"ready candidate and byte finish perform no Index list/doc reads");
    let status=f.engine.prepared_mail_status();
    assert_eq!(status.completed_builds,1);assert_eq!(status.ready_candidate_hits,1);
    assert!(status.charged_bytes>0 && status.charged_bytes<=status.budget_bytes.unwrap());
    assert_eq!(status.seed_calls,0,"trigger-created source tuples require no seed transaction");
}

#[tokio::test]
async fn requested_order_duplicates_missing_and_foreign_ids_are_not_cached_call_identity() {
    let f=fixture().await;
    let ids=vec![f.ids[3].clone(),"missing".into(),f.ids[1].clone(),f.ids[3].clone(),"missing".into()];
    let a=args(&f.account,&ids);let candidate=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();
    let joint=observed(&f,&candidate).await;
    let bytes=f.engine.finish_prepared_metadata(candidate,&joint,"one").unwrap().into_bytes();
    let response:Value=serde_json::from_slice(&bytes).unwrap();let result=&response["methodResponses"][0][1];
    assert_eq!(result["list"].as_array().unwrap().iter().map(|row|row["id"].as_str().unwrap()).collect::<Vec<_>>(),[f.ids[3].as_str(),f.ids[1].as_str(),f.ids[3].as_str()]);
    assert_eq!(result["notFound"],json!(["missing","missing"]));
    let foreign=f.engine.prepare_metadata_candidate(&f.other,&args(&f.other,&f.ids[..1])).await.unwrap();
    let foreign_joint=observed(&f,&foreign).await;
    let foreign_value=f.engine.finish_prepared_metadata_value(&foreign,&foreign_joint).unwrap();
    assert_eq!(foreign_value["list"],json!([]));assert_eq!(foreign_value["notFound"],json!([f.ids[0]]));
}

#[tokio::test]
async fn changed_selected_source_and_pending_writer_refuse_old_result_and_keep_charge_until_drop() {
    let f=fixture().await;let a=args(&f.account,&f.ids[..2]);
    let candidate=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();
    let joint=observed(&f,&candidate).await;
    let ticket=f.engine.reserve_index_publication(&f.account,1).await.unwrap();
    assert!(f.engine.finish_prepared_metadata_value(&candidate,&joint).is_none(),"actual reserved writer closes prepared eligibility");
    drop(ticket);drained(&f.engine).await;
    f.engine.store().set_flags(&f.ids[0],"[\"Seen\"]").await.unwrap();
    let changed=observed(&f,&candidate).await;
    assert!(f.engine.finish_prepared_metadata_value(&candidate,&changed).is_none(),"independent SQL writer must invalidate selected data even before index publication");
    f.engine.prepared_mail.clear_account(&f.account);
    assert_eq!(f.engine.prepared_mail_status().entries,Some(0));
    assert!(f.engine.prepared_mail_status().charged_bytes>0,"evicted result is still charged while held by candidate");
    drop(candidate);assert_eq!(f.engine.prepared_mail_status().charged_bytes,0);
}

#[test]
fn exact_default36_only_and_unsupported_body_or_result_reference_fall_back() {
    let base=args("account",&["id".into()]);assert!(eligible("account",&base).is_some());
    for (key,value) in [("fetchHTMLBodyValues",json!(true)),("ids",json!({"resultOf":"prior","name":"Email/query","path":"/ids"})),("properties",json!(["id"])),("unknown",json!(false))] {
        let mut changed=base.clone();changed[key]=value;assert!(eligible("account",&changed).is_none());
    }
}

#[tokio::test]
async fn unselected_metadata_publication_recertifies_rows_with_fresh_outer_state() {
    let f=fixture().await;let a=args(&f.account,&f.ids[..2]);
    let candidate=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();
    let old=observed(&f,&candidate).await;
    f.engine.store().set_flags(&f.ids[3],"[\"Seen\"]").await.unwrap();
    f.engine.reserve_index_publication(&f.account,1).await.unwrap().submit(vec![f.ids[3].clone()],crate::index_publication::PublicationMode::Metadata).unwrap().await.unwrap();
    drained(&f.engine).await;
    f.engine.store().record_change(&f.account,"Email",&f.ids[3],"updated").await.unwrap();
    let fresh=observed(&f,&candidate).await;
    assert!(old.same_data(&fresh),"unselected Inbox flags do not change selected row/Focus inputs");
    assert_ne!(old.email_state(),fresh.email_state());
    let before=index_counts(&f.engine);
    let hit=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();
    let value=f.engine.finish_prepared_metadata_value(&hit,&fresh).unwrap();
    assert_eq!(value["state"],fresh.email_state());assert_eq!(index_counts(&f.engine),before);
    assert_eq!(f.engine.prepared_mail_status().completed_builds,1,"unrelated owned metadata must not force selected row reconstruction");
}

#[tokio::test]
async fn independent_legacy_index_write_or_raw_escape_cannot_certify_prepared_bytes() {
    let f=fixture().await;let a=args(&f.account,&f.ids[..1]);
    let candidate=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();let joint=observed(&f,&candidate).await;
    let doc=f.engine.search().fetch_doc(&f.ids[0]).unwrap().unwrap();
    f.engine.search().upsert(&doc).unwrap();
    assert!(f.engine.finish_prepared_metadata_value(&candidate,&joint).is_none(),"unknown direct Index publication invalidates independently of SQL epochs");
    let next=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();let next_joint=observed(&f,&next).await;
    let _external=f.engine.search().tantivy();
    assert!(f.engine.finish_prepared_metadata(next,&next_joint,"raw").is_none(),"raw Index escape disables proof rather than inventing coverage");
    assert!(f.engine.prepare_metadata_candidate(&f.account,&a).await.is_none());
}

#[tokio::test]
async fn unselected_sent_reordering_changes_actual_other_row_focus_and_rejects_old_result() {
    let f=fixture().await;
    let selected=f.engine.store().get_account_message(&f.account,&f.ids[0]).await.unwrap().unwrap();
    let selected_thread=selected.thread_id.as_deref().expect("real ingest assigned a thread");
    let sent=f.engine.store().list_mailboxes(&f.account).await.unwrap().into_iter().find(|folder|folder.role.as_deref()==Some("sent")).unwrap();
    // Actual newest500 cutoff:500 recent unrelated Sent threads exclude one
    // old matching thread, as well as the fixture's original five Sent rows.
    for n in 0..501u32 {
        let thread=if n==0{selected_thread.to_owned()}else{format!("unrelated-sent-{n}")};
        f.engine.store().upsert_message(&mw_store::MessageUpsert {account_id:&f.account,mailbox_id:&sent.id,uid:1000+n,uidvalidity:1,message_id:None,thread_id:Some(&thread),internaldate:Some(if n==0{"2020-01-01T00:00:00Z"}else{"2030-01-01T00:00:00Z"}),size:0,flags_json:"[]",envelope:None,blob_ref:None}).await.unwrap();
    }
    let a=args(&f.account,&f.ids[..1]);let candidate=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();
    let before=observed(&f,&candidate).await;
    let value=f.engine.finish_prepared_metadata_value(&candidate,&before).unwrap();
    assert_eq!(value["list"][0]["mailwomanFocus"]["reason"],"default");
    f.engine.store().upsert_message(&mw_store::MessageUpsert {account_id:&f.account,mailbox_id:&sent.id,uid:1000,uidvalidity:1,message_id:None,thread_id:Some(selected_thread),internaldate:Some("2040-01-01T00:00:00Z"),size:0,flags_json:"[]",envelope:None,blob_ref:None}).await.unwrap();
    let after=observed(&f,&candidate).await;
    assert!(!before.same_data(&after),"unselected Sent ordering changes global Focus dependency");
    assert!(f.engine.finish_prepared_metadata_value(&candidate,&after).is_none());
    let rebuilt=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();
    let final_source=observed(&f,&rebuilt).await;
    let value=f.engine.finish_prepared_metadata_value(&rebuilt,&final_source).unwrap();
    assert_eq!(value["list"][0]["mailwomanFocus"]["reason"],"thread","promotion must change actual selected message classification, not only a numeric stamp");
}

#[test]
fn configured_budget_reserves_capacity_and_borrowed_work_prevents_overadmission() {
    let service=PreparedMail::default();
    {let mut memory=service.memory.lock().unwrap();memory.config.budget_mib=16;memory.config.enabled=true;}
    let held=service.reserve().unwrap();
    assert_eq!(held.num_permits(),tokens(BUILD_RESERVATION));
    assert!(service.reserve().is_none(),"two9MiB live preparations must not fit a16MiB configured cache");
    drop(held);assert!(service.reserve().is_some());
    let mut config=PreparedMailOptions::default();
    for budget in [0,15,16385,u32::MAX] {config.budget_mib=budget;assert!(!config.valid());}
    config.budget_mib=16384;config.max_accounts=10000;config.max_entries=100000;assert!(config.valid());
    assert_eq!(serde_json::to_value(config).unwrap()["budgetMiB"],16384);
}

#[tokio::test]
async fn authorized_configuration_disable_invalidates_held_result_without_freeing_its_charge() {
    let f=fixture().await;let candidate=f.engine.prepare_metadata_candidate(&f.account,&args(&f.account,&f.ids[..1])).await.unwrap();
    let joint=observed(&f,&candidate).await;
    assert!(f.engine.configure_prepared_mail(PreparedMailOptions {enabled:false,..Default::default()}));
    assert_eq!(f.engine.prepared_mail_status().entries,Some(0));
    assert!(f.engine.prepared_mail_status().charged_bytes>0);
    assert!(f.engine.finish_prepared_metadata_value(&candidate,&joint).is_none());
    assert!(f.engine.prepare_metadata_candidate(&f.account,&args(&f.account,&f.ids[..1])).await.is_none());
    drop(candidate);assert_eq!(f.engine.prepared_mail_status().charged_bytes,0);
    assert!(!f.engine.configure_prepared_mail(PreparedMailOptions {budget_mib:0,..Default::default()}));
    assert_eq!(f.engine.prepared_mail_status().enabled,Some(false));
}

#[tokio::test]
async fn delayed_configuration_cannot_restore_an_older_budget_or_enabled_state() {
    let f=fixture().await;
    let enabled=PreparedMailOptions {enabled:true,..Default::default()};
    let older=f.engine.store().settings_read_revision().await.unwrap();
    assert!(f.engine.configure_prepared_mail_at_revision(enabled,&older));
    f.engine.store().put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {key:"performance.preparedMail".into(),global_value:Some("{}".into()),allow_user_override:false,global_locked:true,updated_by:"ordering-fixture".into(),updated_at:1}).await.unwrap();
    let newer=f.engine.store().settings_read_revision().await.unwrap();
    assert!(newer.number()>older.number());
    let disabled=PreparedMailOptions {enabled:false,budget_mib:16,..Default::default()};
    assert!(f.engine.configure_prepared_mail_at_revision(disabled,&newer));
    assert!(!f.engine.configure_prepared_mail_at_revision(enabled,&older));
    assert!(!f.engine.configure_prepared_mail_at_revision(enabled,&newer),"one revision cannot name two different configurations");
    assert!(!f.engine.configure_prepared_mail(enabled),"unfenced setter cannot undo versioned configuration");
    let status=f.engine.prepared_mail_status();
    assert_eq!(status.enabled,Some(false));assert_eq!(status.budget_bytes,Some(16*1024*1024));
    let foreign=Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let foreign_revision=foreign.settings_read_revision().await.unwrap();
    assert!(!f.engine.configure_prepared_mail_at_revision(enabled,&foreign_revision));
    foreign.close().await;
}

#[tokio::test]
async fn ready_peek_then_source_mutation_requires_final_proof_and_next_request_rebuilds() {
    let f=fixture().await;let a=args(&f.account,&f.ids[..1]);
    let original=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();
    let (peek,queries)=mw_store::pool_telemetry::measure_queries(f.engine.prepare_metadata_candidate(&f.account,&a)).await;
    let peek=peek.unwrap();assert!(Arc::ptr_eq(&original.ready,&peek.ready));
    assert_eq!(queries.read.calls+queries.write.calls+queries.other.calls,0);
    // Independent source mutation AFTER the pure ready lookup, BEFORE the
    // mandatory final source observation. No enqueue-time SQL authority reuse.
    f.engine.store().set_flags(&f.ids[0],"[\"Seen\"]").await.unwrap();
    let final_source=observed(&f,&peek).await;
    assert!(f.engine.finish_prepared_metadata(peek,&final_source,"changed").is_none());
    assert_eq!(f.engine.prepared_mail_status().entries,Some(0),"mismatch removes the stale Ready rather than causing permanent fallback");
    let rebuilt=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();
    assert!(!Arc::ptr_eq(&original.ready,&rebuilt.ready));
    let current=observed(&f,&rebuilt).await;
    let value=f.engine.finish_prepared_metadata_value(&rebuilt,&current).unwrap();
    assert_eq!(value["list"][0]["keywords"]["$seen"],true);
    assert_eq!(f.engine.prepared_mail_status().completed_builds,2);
}

#[tokio::test]
async fn missing_final_data_discard_is_same_arc_only_and_old_finish_cannot_evict_new_result() {
    let f=fixture().await;let a=args(&f.account,&f.ids[..1]);
    let old=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();
    // Endpoint uses this pure hook when its last joint source row is None or
    // malformed and there is no typed MailDataFence to pass into finish.
    assert!(f.engine.discard_prepared_metadata(&old));
    f.engine.store().set_flags(&f.ids[0],"[\"Seen\"]").await.unwrap();
    let newer=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();
    assert!(!Arc::ptr_eq(&old.ready,&newer.ready));
    assert!(!f.engine.discard_prepared_metadata(&old),"a late old request must not evict a concurrent newer install");
    let current=observed(&f,&newer).await;
    assert!(f.engine.finish_prepared_metadata(old,&current,"old").is_none());
    assert_eq!(f.engine.prepared_mail_status().entries,Some(1));
    let (hit,queries)=mw_store::pool_telemetry::measure_queries(f.engine.prepare_metadata_candidate(&f.account,&a)).await;
    let hit=hit.unwrap();assert!(Arc::ptr_eq(&hit.ready,&newer.ready));
    assert_eq!(queries.read.calls+queries.write.calls+queries.other.calls,0);
    assert!(f.engine.finish_prepared_metadata(hit,&current,"new").is_some());
}

struct UnsupportedPosture;
impl crate::v6::AccountPostureSource for UnsupportedPosture {
    fn posture(&self,_:&str)->crate::v6::AccountPosture {crate::v6::AccountPosture::Standard}
}
#[tokio::test]
async fn ready_lookup_does_not_bypass_pending_or_unsupported_posture_before_final_proof() {
    let f=fixture().await;let a=args(&f.account,&f.ids[..1]);
    let _ready=f.engine.prepare_metadata_candidate(&f.account,&a).await.unwrap();
    let ticket=f.engine.reserve_index_publication(&f.account,1).await.unwrap();
    let (blocked,queries)=mw_store::pool_telemetry::measure_queries(f.engine.prepare_metadata_candidate(&f.account,&a)).await;
    assert!(blocked.is_none());assert_eq!(queries.read.calls+queries.write.calls+queries.other.calls,0);
    drop(ticket);drained(&f.engine).await;
    f.engine.attach_v6(crate::v6::V6Hooks::new().with_posture_source(Arc::new(UnsupportedPosture)));
    let (unsupported,queries)=mw_store::pool_telemetry::measure_queries(f.engine.prepare_metadata_candidate(&f.account,&a)).await;
    assert!(unsupported.is_none());assert_eq!(queries.read.calls+queries.write.calls+queries.other.calls,0);
}
