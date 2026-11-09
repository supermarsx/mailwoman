use mw_search::{Index, IndexDoc, SearchQuery, parse_query, KW_FOLLOWUP_ACTIVE, KW_FOLLOWUP_COMPLETED};
fn find(index:&Index, text:&str)->Vec<String>{let mut ids=index.search(&parse_query(text).unwrap(),0).unwrap();ids.sort();ids}
#[test]
fn followup_and_finished_are_indexed_before_paging_and_remain_distinct_from_plain_flags() {
    let index=Index::open_in_ram().unwrap();
    let docs:Vec<_>=[("plain",vec![]),("flagged",vec!["$flagged"]),("reminder",vec![KW_FOLLOWUP_ACTIVE]),("done",vec![KW_FOLLOWUP_COMPLETED]),("old-done",vec!["$flagged",KW_FOLLOWUP_ACTIVE,KW_FOLLOWUP_COMPLETED])].into_iter().map(|(id,keywords)|IndexDoc{stable_id:id.into(),account_id:"owner".into(),mailbox_id:"inbox".into(),keywords:keywords.into_iter().map(str::to_owned).collect(),..IndexDoc::default()}).collect();
    index.upsert_batch(&docs).unwrap();
    assert_eq!(find(&index,"is:followup"),vec!["flagged","reminder"]);
    assert_eq!(find(&index,"is:follow-up"),vec!["flagged","reminder"]);
    assert_eq!(find(&index,"is:finished"),vec!["done","old-done"]);
    assert_eq!(find(&index,"-is:followup"),vec!["done","old-done","plain"]);
    assert!(find(&index,"is:followup is:finished").is_empty());
    let page=index.search_hits(&parse_query("is:followup").unwrap(),1).unwrap();
    assert!(page.truncated);assert_eq!(page.ids.len(),1);
    index.upsert(&IndexDoc{stable_id:"reminder".into(),keywords:vec![KW_FOLLOWUP_COMPLETED.into()],..docs[2].clone()}).unwrap();
    assert_eq!(find(&index,"is:followup"),vec!["flagged"]);
    assert_eq!(find(&index,"is:finished"),vec!["done","old-done","reminder"]);
}
#[test]
fn mention_queries_match_exact_canonical_addresses_and_combine_with_other_filters() {
    let index=Index::open_in_ram().unwrap();
    index.upsert_batch(&[
        IndexDoc{stable_id:"mentioned".into(),keywords:vec!["$mw-mention:alice@example.org".into(),"$flagged".into()],..IndexDoc::default()},
        IndexDoc{stable_id:"other".into(),keywords:vec!["$mw-mention:notalice@example.org".into()],..IndexDoc::default()},
    ]).unwrap();
    assert_eq!(find(&index,"mentions:ALICE@EXAMPLE.ORG"),vec!["mentioned"]);
    assert_eq!(find(&index,"mentions:alice@example.org is:followup"),vec!["mentioned"]);
    assert_eq!(find(&index,"-mentions:alice@example.org"),vec!["other"]);
    assert_eq!(index.search(&SearchQuery::all(),0).unwrap().len(),2);
}
