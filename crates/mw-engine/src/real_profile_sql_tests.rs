// Exact benchmark property arrays, interpreted only from checked-in test source.
fn real_profile_properties(name: &str) -> Vec<String> {
    let source = include_str!("../../../scripts/perf/fixture-properties.mjs");
    let prefix = format!("export const {name}=Object.freeze(");
    let body = source.lines().find_map(|line|line.strip_prefix(&prefix))
        .and_then(|line|line.strip_suffix(");")).expect("known property declaration");
    let body = body.strip_prefix('[').and_then(|value|value.strip_suffix(']')).expect("array");
    if !body.contains("...") { return serde_json::from_str(&format!("[{body}] ").replace('\'',"\"")).expect("literal property array"); }
    body.split(',').flat_map(|item|real_profile_properties(item.trim().strip_prefix("...").expect("property spread"))).collect()
}
async fn real_profile_fixture(n:usize) -> Fixture {
    let f=Fixture::with(n).await;
    for (index,id) in f.ids.iter().enumerate() {
        let row=f.store().get_message(id).await.unwrap();
        let raw=format!("From: sender@native-load.invalid\r\nTo: owner@native-load.invalid\r\nSubject: Native profile {index}\r\nMessage-ID: <{index}@native-load.invalid>\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nRepresentative body sample.\r\n");
        let parsed=mw_mime::parse(raw.as_bytes()).unwrap();
        let envelope=crate::encode_message_envelope(&parsed.email,raw.as_bytes());
        let blob=f.store().put_body(&f.account,raw.as_bytes()).await.unwrap();
        assert_eq!(f.store().upsert_message(&MessageUpsert {
            account_id:&f.account,mailbox_id:&row.mailbox_id,uid:row.uid,uidvalidity:row.uidvalidity,
            message_id:row.message_id.as_deref(),thread_id:row.thread_id.as_deref(),internaldate:Some("2026-11-01T00:00:00Z"),
            size:raw.len() as u64,flags_json:&row.flags_json,envelope:Some(&envelope),blob_ref:Some(&blob),
        }).await.unwrap(),*id);
    }
    attach_header_cache(&f);
    f
}
#[tokio::test]
async fn exact_default_ui_profile_five_vs_fifty_has_no_hidden_per_message_body_or_full_envelope_reads() {
    let mut totals=Vec::new();
    for count in [5,50] {
        let f=real_profile_fixture(count).await;
        let args=json!({"ids":f.ids,"properties":real_profile_properties("DEFAULT_HEADER_PROPERTIES")});
        assert!(metadata_envelope_only(&args),"real UI properties must take the light path");
        let (response,cold)=counted(&f.threads,f.engine.email_get(&f.account,&args)).await;
        assert_eq!(response["list"].as_array().unwrap().len(),count,"{response}");
        assert_eq!(cold.against("message_list_envelope"),1,"{cold:#?}");
        assert_eq!(cold.against("bodies"),0,"{cold:#?}");
        assert_eq!(cold.against("LEFT JOIN message_meta"),1,"fresh message+meta must share one read: {cold:#?}");
        assert!(cold.matching("envelope_json").is_empty(),"{cold:#?}");
        let (_,warm)=counted(&f.threads,f.engine.email_get(&f.account,&args)).await;
        assert_eq!(warm.against("message_list_envelope"),0,"{warm:#?}");
        assert_eq!(warm.against("bodies"),0,"{warm:#?}");
        eprintln!("real default UI {count} cold={cold:#?} warm={warm:#?}");
        totals.push((cold.len(),warm.len()));
    }
    assert!(totals[0].0>0&&totals[0].1>0,"physical recorder must see reads");
    assert_eq!(totals[0],totals[1],"default UI query count must stay page-independent");
    assert_eq!(totals[0],(6,5),"one fresh join replaces the measured7cold/6warm physical statements");
}
#[tokio::test]
async fn exact_reader_profile_exposes_current_full_cache_miss_and_body_use_query_families() {
    for count in [5,50] {
        let f=real_profile_fixture(count).await;
        let args=json!({"ids":f.ids,"properties":real_profile_properties("READER_PROPERTIES"),"fetchHTMLBodyValues":true,"fetchTextBodyValues":true,"maxBodyValueBytes":1000000,"mailwomanBodyUse":true});
        assert!(!metadata_envelope_only(&args));
        let (response,cold)=counted(&f.threads,f.engine.email_get(&f.account,&args)).await;
        assert_eq!(response["list"].as_array().unwrap().len(),count,"{response}");
        for email in response["list"].as_array().unwrap(){assert!(!email["bodyValues"].as_object().unwrap().is_empty());}
        let (_,warm)=counted(&f.threads,f.engine.email_get(&f.account,&args)).await;
        assert!(cold.len()>=count*2,"reader still hydrates fresh message/envelope per id; instrument must expose it: {cold:#?}");
        eprintln!("real reader {count} cold={cold:#?} warm={warm:#?}");
    }
}
