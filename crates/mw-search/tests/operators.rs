//! Operator + lifecycle coverage over the shared fixture corpus
//! (`fixtures/search/corpus.json`). Every operator in plan §0.1 parses and
//! returns the correct stable ids; sorts, boolean, and phrase are exercised;
//! and the index→search→delete→relocate lifecycle is asserted.

use mw_search::{Index, IndexDoc, SearchQuery, Sort, SortField, parse_query};

/// Load the fixture corpus (four hand-built messages).
fn corpus() -> Vec<IndexDoc> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/search/corpus.json"
    );
    let raw = std::fs::read_to_string(path).expect("read corpus fixture");
    serde_json::from_str(&raw).expect("parse corpus fixture")
}

/// Build an in-RAM index loaded with the whole corpus.
fn indexed() -> Index {
    let idx = Index::open_in_ram().expect("open ram index");
    idx.upsert_batch(&corpus()).expect("index corpus");
    idx
}

/// Run operator text and return the matching ids, sorted for set comparison.
fn find(idx: &Index, q: &str) -> Vec<String> {
    let query = parse_query(q).expect("parse query");
    let mut ids = idx.search(&query, 0).expect("search");
    ids.sort();
    ids
}

/// Ordered ids (sort preserved) for a match-all query under `sort`.
fn all_sorted(idx: &Index, sort: Sort) -> Vec<String> {
    let q = SearchQuery::all().with_sort(sort);
    idx.search(&q, 0).expect("search")
}

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn field_operators_return_correct_ids() {
    let idx = indexed();
    assert_eq!(find(&idx, "from:alice"), ids(&["m1"]));
    assert_eq!(find(&idx, "to:alice"), ids(&["m2", "m3", "m4"]));
    assert_eq!(find(&idx, "cc:bob"), ids(&["m2"]));
    assert_eq!(find(&idx, "subject:invoice"), ids(&["m3"]));
    assert_eq!(find(&idx, "body:lunch"), ids(&["m2"]));
    assert_eq!(find(&idx, "quarterly"), ids(&["m1"]));
    assert_eq!(find(&idx, "text:quarterly"), ids(&["m1"]));
    assert_eq!(find(&idx, "filename:invoice"), ids(&["m3"]));
}

#[test]
fn attachment_mailbox_keyword_pinned_operators() {
    let idx = indexed();
    assert_eq!(find(&idx, "has:attachment"), ids(&["m1", "m3"]));
    assert_eq!(find(&idx, "in:INBOX"), ids(&["m1", "m2", "m4"]));
    assert_eq!(find(&idx, "in:Archive"), ids(&["m3"]));
    assert_eq!(find(&idx, "tag:Work"), ids(&["m1"]));
    assert_eq!(find(&idx, "tag:Finance"), ids(&["m3"]));
    assert_eq!(find(&idx, "is:unread"), ids(&["m2", "m4"]));
    assert_eq!(find(&idx, "is:read"), ids(&["m1", "m3"]));
    assert_eq!(find(&idx, "pinned:true"), ids(&["m1"]));
    assert_eq!(find(&idx, "pinned:false"), ids(&["m2", "m3", "m4"]));
}

#[test]
fn date_and_size_range_operators() {
    let idx = indexed();
    assert_eq!(find(&idx, "after:2020-01-01"), ids(&["m1", "m2", "m4"]));
    assert_eq!(find(&idx, "before:2020-01-03"), ids(&["m1", "m3"]));
    assert_eq!(find(&idx, "larger:10000"), ids(&["m1", "m3"]));
    assert_eq!(find(&idx, "smaller:10000"), ids(&["m2", "m4"]));
    assert_eq!(find(&idx, "larger:1k"), ids(&["m1", "m2", "m3", "m4"]));
}

#[test]
fn boolean_and_or_not() {
    let idx = indexed();
    assert_eq!(find(&idx, "from:alice AND has:attachment"), ids(&["m1"]));
    assert_eq!(find(&idx, "tag:Work OR tag:Finance"), ids(&["m1", "m3"]));
    assert_eq!(find(&idx, "in:INBOX NOT tag:Personal"), ids(&["m1", "m4"]));
    assert_eq!(find(&idx, "in:INBOX -pinned:true"), ids(&["m2", "m4"]));
    // Implicit AND between adjacent atoms.
    assert_eq!(find(&idx, "in:INBOX has:attachment"), ids(&["m1"]));
}

#[test]
fn phrase_queries_respect_adjacency() {
    let idx = indexed();
    assert_eq!(find(&idx, "subject:\"quarterly report\""), ids(&["m1"]));
    // Reversed word order is not an adjacent phrase.
    assert!(find(&idx, "subject:\"report quarterly\"").is_empty());
}

#[test]
fn sort_orders() {
    let idx = indexed();
    // receivedAt desc (default): newest first.
    assert_eq!(
        all_sorted(&idx, Sort::for_field(SortField::ReceivedAt)),
        ids(&["m4", "m2", "m1", "m3"])
    );
    // size ascending / descending.
    assert_eq!(
        all_sorted(
            &idx,
            Sort {
                field: SortField::Size,
                ascending: true
            }
        ),
        ids(&["m2", "m4", "m1", "m3"])
    );
    assert_eq!(
        all_sorted(
            &idx,
            Sort {
                field: SortField::Size,
                ascending: false
            }
        ),
        ids(&["m3", "m1", "m4", "m2"])
    );
    // from ascending (alphabetical by lowercased from address).
    assert_eq!(
        all_sorted(&idx, Sort::for_field(SortField::From)),
        ids(&["m1", "m2", "m3", "m4"])
    );
    // subject ascending.
    assert_eq!(
        all_sorted(&idx, Sort::for_field(SortField::Subject)),
        ids(&["m3", "m2", "m1", "m4"])
    );
}

#[test]
fn limit_is_honored() {
    let idx = indexed();
    let q = SearchQuery::all().with_sort(Sort::for_field(SortField::ReceivedAt));
    let top2 = idx.search(&q, 2).expect("search");
    assert_eq!(top2, ids(&["m4", "m2"]));
}

#[test]
fn lifecycle_index_search_delete_relocate() {
    let idx = indexed();
    assert_eq!(idx.num_docs(), 4);

    // delete removes it from results.
    idx.delete("m2").expect("delete");
    assert_eq!(idx.num_docs(), 3);
    assert!(find(&idx, "body:lunch").is_empty());

    // relocate preserves the stable id + every other field, only moving mailbox.
    assert_eq!(find(&idx, "in:Archive"), ids(&["m3"]));
    idx.relocate("m3", "Trash").expect("relocate");
    assert_eq!(idx.num_docs(), 3); // still one doc, re-keyed in place.
    assert!(find(&idx, "in:Archive").is_empty());
    assert_eq!(find(&idx, "in:Trash"), ids(&["m3"]));
    // Non-mailbox fields survive the move.
    assert_eq!(find(&idx, "subject:invoice"), ids(&["m3"]));
    assert_eq!(find(&idx, "tag:Finance"), ids(&["m3"]));
}

#[test]
fn upsert_replaces_in_place() {
    let idx = indexed();
    let mut doc = corpus().into_iter().find(|d| d.stable_id == "m1").unwrap();
    doc.subject = "Completely different heading".to_string();
    idx.upsert(&doc).expect("upsert");
    assert_eq!(idx.num_docs(), 4); // replaced, not duplicated.
    assert!(find(&idx, "subject:quarterly").is_empty());
    assert_eq!(find(&idx, "subject:heading"), ids(&["m1"]));
}

#[test]
fn empty_query_matches_all() {
    let idx = indexed();
    let mut got = idx.search(&SearchQuery::all(), 0).expect("search");
    got.sort();
    assert_eq!(got, ids(&["m1", "m2", "m3", "m4"]));
}

#[test]
fn relocate_missing_id_is_noop() {
    let idx = indexed();
    idx.relocate("does-not-exist", "INBOX").expect("noop");
    assert_eq!(idx.num_docs(), 4);
}
