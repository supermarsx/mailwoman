//! Compliance documents: policy versions and overrides, records, DPIA history.

use mw_store::compliance::{ComplianceRecord, Dpia, PolicyOverride, PolicyVersion};
use mw_store::{ServerKey, Store};

async fn store() -> Store {
    Store::open_in_memory(ServerKey::from_bytes(&[7u8; 32]).unwrap()).await.unwrap()
}

#[tokio::test]
async fn policy_versions_are_ordered_and_unique_per_domain() {
    let s = store().await;
    assert!(s.latest_policy_version("").await.unwrap().is_none());
    let v = |n: i64, d: &str| PolicyVersion {
        id: format!("{d}-{n}"),
        domain: d.into(),
        version: n,
        kind: "auto".into(),
        facts_json: "{}".into(),
        changes_json: "[]".into(),
        created_by: "t".into(),
        created_at: n,
    };
    s.add_policy_version(&v(1, "")).await.unwrap();
    s.add_policy_version(&v(2, "")).await.unwrap();
    s.add_policy_version(&v(1, "a.test")).await.unwrap();
    assert!(s.add_policy_version(&PolicyVersion { id: "dup".into(), ..v(2, "") }).await.is_err());
    assert_eq!(s.latest_policy_version("").await.unwrap().unwrap().version, 2);
    let all = s.list_policy_versions("", 10).await.unwrap();
    assert_eq!(all.iter().map(|x| x.version).collect::<Vec<_>>(), vec![2, 1]);
    assert_eq!(s.list_policy_versions("a.test", 10).await.unwrap().len(), 1);
}

#[tokio::test]
async fn overrides_replace_per_section_and_language() {
    let s = store().await;
    let o = |body: &str| PolicyOverride {
        id: uuid::Uuid::new_v4().to_string(),
        domain: String::new(),
        lang: "en".into(),
        section: "contact".into(),
        body: body.into(),
        updated_by: "t".into(),
        updated_at: 1,
    };
    s.put_policy_override(&o("one")).await.unwrap();
    s.put_policy_override(&o("two")).await.unwrap();
    let list = s.list_policy_overrides("").await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].body, "two");
    assert!(s.delete_policy_override("", "en", "contact").await.unwrap());
    assert!(!s.delete_policy_override("", "en", "contact").await.unwrap());
}

#[tokio::test]
async fn records_round_trip_by_kind() {
    let s = store().await;
    let r = |id: &str, kind: &str, name: &str| ComplianceRecord {
        id: id.into(),
        kind: kind.into(),
        name: name.into(),
        body_json: "{}".into(),
        updated_by: "t".into(),
        updated_at: 1,
    };
    s.put_compliance_record(&r("1", "activity", "B")).await.unwrap();
    s.put_compliance_record(&r("2", "activity", "A")).await.unwrap();
    s.put_compliance_record(&r("3", "processor", "P")).await.unwrap();
    s.put_compliance_record(&r("1", "activity", "C")).await.unwrap();
    let names: Vec<_> = s.list_compliance_records("activity").await.unwrap().into_iter().map(|x| x.name).collect();
    assert_eq!(names, vec!["A", "C"]);
    assert!(s.delete_compliance_record("2").await.unwrap());
    assert!(s.get_compliance_record("2").await.unwrap().is_none());
}

#[tokio::test]
async fn dpia_keeps_a_version_per_save() {
    let s = store().await;
    let mut d = Dpia {
        id: "d1".into(),
        title: "Assistant".into(),
        status: "draft".into(),
        template: "generic".into(),
        body_json: "{}".into(),
        version: 1,
        review_at: None,
        created_by: "t".into(),
        created_at: 1,
        updated_by: "t".into(),
        updated_at: 1,
    };
    s.save_dpia(&d).await.unwrap();
    d.version = 2;
    d.status = "review".into();
    d.review_at = Some(99);
    s.save_dpia(&d).await.unwrap();
    let got = s.get_dpia("d1").await.unwrap().unwrap();
    assert_eq!((got.version, got.status.as_str(), got.review_at), (2, "review", Some(99)));
    let versions = s.list_dpia_versions("d1").await.unwrap();
    assert_eq!(versions.iter().map(|v| v.version).collect::<Vec<_>>(), vec![2, 1]);
    assert!(s.delete_dpia("d1").await.unwrap());
    assert!(s.list_dpia_versions("d1").await.unwrap().is_empty());
}

#[tokio::test]
async fn notification_claims_are_atomic_and_payloads_are_bound_to_their_id(){
    let s=store().await;
    let sealed=s.seal_compliance_delivery("notice",b"private recipient and message").unwrap();
    assert_eq!(s.open_compliance_delivery("notice",&sealed).unwrap(),b"private recipient and message");
    assert!(s.open_compliance_delivery("other",&sealed).is_err());
    let row=ComplianceRecord{id:"notice".into(),kind:"compliance-mail:pending".into(),name:"event".into(),body_json:"sealed-payload".into(),updated_by:"system".into(),updated_at:1};
    assert!(s.insert_compliance_record_once(&row).await.unwrap());assert!(!s.insert_compliance_record_once(&row).await.unwrap());
    let mut claimed=row.clone();claimed.kind="compliance-mail:sending".into();claimed.updated_at=2;
    let (a,b)=tokio::join!(s.transition_compliance_record(&row,&claimed),s.transition_compliance_record(&row,&claimed));
    assert_eq!(usize::from(a.unwrap())+usize::from(b.unwrap()),1);
    let mut sent=claimed.clone();sent.kind="compliance-mail:sent".into();sent.updated_at=3;assert!(s.transition_compliance_record(&claimed,&sent).await.unwrap());
    let mut pending=row.clone();pending.id="pending".into();s.insert_compliance_record_once(&pending).await.unwrap();
    assert_eq!(s.purge_compliance_delivery_history(10).await.unwrap(),1);
    assert!(s.get_compliance_record("pending").await.unwrap().is_some());
}
