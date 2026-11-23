//! Request-local list fields. Large stored body strings are validated, not copied.
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::atomic::Ordering;

use serde::{
    Deserialize, Deserializer,
    de::{SeqAccess, Visitor},
};
use tantivy::{
    Term,
    collector::TopDocs,
    query::{BooleanQuery, Occur, Query, TermQuery, TermSetQuery},
    schema::{IndexRecordOption, TantivyDocument, Value},
};

use crate::{CalendarHint, Index, Result, SearchError};

/// Matches the mail API's maximum objects per get; larger internal callers are
/// chunked while retaining one searcher generation for the complete operation.
pub const MAX_LIST_PROJECTION_IDS: usize = 5000;

// Small exact-ID sets avoid constructing an FST for every reader/list request.
// Larger sets retain the compact automaton rather than thousands of clauses.
const SMALL_LIST_PROJECTION_IDS: usize = 32;
fn projection_id_query(field: tantivy::schema::Field, ids: &[&str]) -> Box<dyn Query> {
    if ids.len() <= SMALL_LIST_PROJECTION_IDS {
        Box::new(BooleanQuery::new(ids.iter().map(|id| {
            (Occur::Should, Box::new(TermQuery::new(
                Term::from_field_text(field, id), IndexRecordOption::Basic,
            )) as Box<dyn Query>)
        }).collect()))
    } else {
        Box::new(TermSetQuery::new(ids.iter().map(|id| Term::from_field_text(field, id))))
    }
}

#[derive(Debug, Default)]
struct SkipString;
impl<'de> Deserialize<'de> for SkipString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct StringVisitor;
        impl<'de> Visitor<'de> for StringVisitor {
            type Value = SkipString;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a string")
            }
            fn visit_str<E: serde::de::Error>(
                self,
                _: &str,
            ) -> std::result::Result<Self::Value, E> {
                Ok(SkipString)
            }
        }
        deserializer.deserialize_str(StringVisitor)
    }
}
#[derive(Debug, Default)]
struct SkipStrings;
impl<'de> Deserialize<'de> for SkipStrings {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct StringsVisitor;
        impl<'de> Visitor<'de> for StringsVisitor {
            type Value = SkipStrings;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a string array")
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                while seq.next_element::<SkipString>()?.is_some() {}
                Ok(SkipStrings)
            }
        }
        deserializer.deserialize_seq(StringsVisitor)
    }
}

/// Only fields consumed by list projections are retained. The skipped fields
/// still have IndexDoc's required/type/default rules, so malformed legacy JSON
/// remains absent exactly as with fetch_doc. Escaped strings may use serde's
/// parsing scratch buffer; no owned copy of body/attachment text is retained.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProjection {
    pub stable_id: String,
    pub account_id: String,
    pub keywords: Vec<String>,
    #[serde(default)]
    pub calendar: Option<CalendarHint>,
    #[serde(default)]
    pub list_signal: Option<String>,
    #[serde(rename = "mailboxId")]
    _mailbox_id: SkipString,
    #[serde(rename = "from")]
    _from: SkipString,
    #[serde(default, rename = "fromDomains")]
    _from_domains: SkipStrings,
    #[serde(rename = "to")]
    _to: SkipString,
    #[serde(rename = "cc")]
    _cc: SkipString,
    #[serde(rename = "subject")]
    _subject: SkipString,
    #[serde(rename = "body")]
    _body: SkipString,
    #[serde(rename = "date")]
    _date: i64,
    #[serde(default, rename = "sentAt")]
    _sent_at: Option<i64>,
    #[serde(default, rename = "fromAddress")]
    _from_address: Option<SkipString>,
    #[serde(rename = "hasAttachment")]
    _has_attachment: bool,
    #[serde(rename = "size")]
    _size: u64,
    #[serde(rename = "filenames")]
    _filenames: SkipStrings,
    #[serde(rename = "pinned")]
    _pinned: bool,
    #[serde(default, rename = "attachmentText")]
    _attachment_text: SkipString,
}

/// The recipients of one indexed message, for the Focus "written to" check
/// (t50hp-e7). Every other field keeps [`crate::IndexDoc`]'s required/type
/// rules, as in [`ListProjection`], so a document `fetch_doc` would reject is
/// absent here too.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipientProjection {
    pub stable_id: String,
    pub account_id: String,
    pub to: String,
    pub cc: String,
    #[serde(rename = "keywords")]
    _keywords: SkipStrings,
    #[serde(default, rename = "calendar")]
    _calendar: Option<CalendarHint>,
    #[serde(default, rename = "listSignal")]
    _list_signal: Option<SkipString>,
    #[serde(rename = "mailboxId")]
    _mailbox_id: SkipString,
    #[serde(rename = "from")]
    _from: SkipString,
    #[serde(default, rename = "fromDomains")]
    _from_domains: SkipStrings,
    #[serde(rename = "subject")]
    _subject: SkipString,
    #[serde(rename = "body")]
    _body: SkipString,
    #[serde(rename = "date")]
    _date: i64,
    #[serde(default, rename = "sentAt")]
    _sent_at: Option<i64>,
    #[serde(default, rename = "fromAddress")]
    _from_address: Option<SkipString>,
    #[serde(rename = "hasAttachment")]
    _has_attachment: bool,
    #[serde(rename = "size")]
    _size: u64,
    #[serde(rename = "filenames")]
    _filenames: SkipStrings,
    #[serde(rename = "pinned")]
    _pinned: bool,
    #[serde(default, rename = "attachmentText")]
    _attachment_text: SkipString,
}

/// A thin projection bound to the account and id it was read for.
trait Bound {
    fn binding(&self) -> (&str, &str);
}
impl Bound for ListProjection {
    fn binding(&self) -> (&str, &str) {
        (&self.account_id, &self.stable_id)
    }
}
impl Bound for RecipientProjection {
    fn binding(&self) -> (&str, &str) {
        (&self.account_id, &self.stable_id)
    }
}

impl Index {
    /// One account-filtered index search per bounded chunk. Callers must supply
    /// only IDs from fresh account-owned SQL rows: the index is never an
    /// authorization source. Both indexed and decoded account/ID bindings are
    /// checked. Missing/malformed documents are absent; ambiguous duplicate live
    /// documents are also absent rather than selecting an arbitrary version.
    pub fn fetch_list_projections(
        &self,
        account_id: &str,
        stable_ids: &[String],
    ) -> Result<HashMap<String, ListProjection>> {
        self.fetch_bound(account_id, stable_ids)
    }

    /// [`Index::fetch_list_projections`]'s batched, account-bound read, for
    /// the `to`/`cc` of documents (t50hp-e7): one search per chunk instead of
    /// one `fetch_doc` search and full document decode per id. The same
    /// ownership rule applies: pass only ids from account-owned SQL rows.
    pub fn fetch_recipient_projections(
        &self,
        account_id: &str,
        stable_ids: &[String],
    ) -> Result<HashMap<String, RecipientProjection>> {
        self.fetch_bound(account_id, stable_ids)
    }

    fn fetch_bound<T: serde::de::DeserializeOwned + Bound>(
        &self,
        account_id: &str,
        stable_ids: &[String],
    ) -> Result<HashMap<String, T>> {
        let ids = stable_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let mut found = HashMap::new();
        if ids.is_empty() || account_id.is_empty() {
            return Ok(found);
        }
        let searcher = self.reader.searcher();
        for chunk in ids.chunks(MAX_LIST_PROJECTION_IDS) {
            let query = BooleanQuery::new(vec![
                (
                    Occur::Must,
                    Box::new(TermQuery::new(
                        Term::from_field_text(self.fields.account_id, account_id),
                        IndexRecordOption::Basic,
                    )),
                ),
                (
                    Occur::Must,
                    projection_id_query(self.fields.stable_id, chunk),
                ),
            ]);
            self.list_projection_searches
                .fetch_add(1, Ordering::Relaxed);
            let hits = searcher.search(
                &query,
                &TopDocs::with_limit(chunk.len() + 1).order_by_score(),
            )?;
            if hits.len() > chunk.len() {
                return Err(SearchError::Tantivy(
                    "ambiguous list projection index documents".into(),
                ));
            }
            let mut ambiguous = HashSet::new();
            for (_, address) in hits {
                self.list_projection_doc_reads
                    .fetch_add(1, Ordering::Relaxed);
                let Ok(document) = searcher.doc::<TantivyDocument>(address) else {
                    continue;
                };
                let Some(stored_id) = document
                    .get_first(self.fields.stable_id)
                    .and_then(|value| value.as_str())
                else {
                    continue;
                };
                let Some(json) = document
                    .get_first(self.fields.doc_json)
                    .and_then(|value| value.as_str())
                else {
                    continue;
                };
                let Ok(projection) = serde_json::from_str::<T>(json) else {
                    continue;
                };
                let (bound_account, bound_id) = projection.binding();
                if bound_account != account_id
                    || bound_id != stored_id
                    || chunk.binary_search(&stored_id).is_err()
                {
                    continue;
                }
                if ambiguous.contains(stored_id) {
                    continue;
                }
                if found.contains_key(stored_id) {
                    found.remove(stored_id);
                    ambiguous.insert(stored_id.to_owned());
                } else {
                    found.insert(stored_id.to_owned(), projection);
                }
            }
        }
        Ok(found)
    }
    /// Actual bounded projection searches, distinct from stored-document reads.
    pub fn list_projection_search_count(&self) -> u64 {
        self.list_projection_searches.load(Ordering::Relaxed)
    }
    pub fn list_projection_doc_read_count(&self) -> u64 {
        self.list_projection_doc_reads.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IndexDoc;
    use serde_json::json;

    fn doc(id: &str, account: &str) -> IndexDoc {
        IndexDoc {
            stable_id: id.into(),
            account_id: account.into(),
            mailbox_id: "inbox".into(),
            body: "large escaped body\n".repeat(1000),
            keywords: vec!["$mention:me@example.test".into()],
            list_signal: Some("list-id".into()),
            ..Default::default()
        }
    }
    fn insert_raw(index: &Index, indexed_id: &str, indexed_account: &str, json: Option<&str>) {
        let mut document = TantivyDocument::default();
        document.add_text(index.fields.stable_id, indexed_id);
        document.add_text(index.fields.account_id, indexed_account);
        if let Some(json) = json {
            document.add_text(index.fields.doc_json, json);
        }
        let mut writer = index.lock_writer().unwrap();
        writer.add_document(document).unwrap();
        writer.commit().unwrap();
        drop(writer);
        index.reader.reload().unwrap();
    }
    #[test]
    fn small_query_boundary_preserves_fst_results_across_segments() {
        for segments in [1, 8] {
            let (schema, fields) = crate::build_schema();
            let inner = tantivy::Index::create_in_ram(schema);
            let writer = inner.writer_with_num_threads::<TantivyDocument>(1, 50_000_000).unwrap();
            writer.set_merge_policy(Box::new(tantivy::merge_policy::NoMergePolicy));
            let reader = inner.reader_builder().reload_policy(tantivy::ReloadPolicy::Manual).try_into().unwrap();
            let index = Index {
                inner, reader, writer: std::sync::Mutex::new(writer), fields,
                writer_configuration: crate::WriterConfiguration { indexing_workers: 1, memory_budget_bytes: 50_000_000 },
                doc_fetches: Default::default(), list_projection_searches: Default::default(),
                list_projection_doc_reads: Default::default(),
                publication_poisoned:Default::default(),publication_owner:Default::default(),
                unowned_projection:Default::default(),
                publication_fault:Default::default(),publication_lock_hook:Default::default(),
            };
            for segment in 0..segments {
                let mut writer = index.lock_writer().unwrap();
                for n in (segment..64).step_by(segments) {
                    index.write_one(&writer, &doc(&format!("id-{n:03}"), "owned")).unwrap();
                }
                if segment == 0 {
                    index.write_one(&writer, &doc("foreign", "other")).unwrap();
                }
                writer.commit().unwrap();
            }
            index.reader.reload().unwrap();
            let searcher = index.reader.searcher();
            assert_eq!(searcher.segment_readers().len(), segments);
            for count in [1, 2, 25, 32, 33, 50] {
                let mut ids = (0..count-1).map(|n| format!("id-{n:03}")).collect::<Vec<_>>();
                ids.push("foreign".into());
                let refs = ids.iter().map(String::as_str).collect::<Vec<_>>();
                let selected = projection_id_query(index.fields.stable_id, &refs);
                assert_eq!(selected.downcast_ref::<BooleanQuery>().is_some(), count <= 32);
                assert_eq!(selected.downcast_ref::<TermSetQuery>().is_some(), count > 32);
                let baseline = BooleanQuery::new(vec![
                    (Occur::Must, Box::new(TermQuery::new(Term::from_field_text(index.fields.account_id, "owned"), IndexRecordOption::Basic))),
                    (Occur::Must, Box::new(TermSetQuery::new(refs.iter().map(|id| Term::from_field_text(index.fields.stable_id, id))))),
                ]);
                let baseline_hits = searcher.search(&baseline, &TopDocs::with_limit(count+1).order_by_score()).unwrap();
                let mut expected = baseline_hits.into_iter().map(|(_,address)| {
                    searcher.doc::<TantivyDocument>(address).unwrap().get_first(index.fields.stable_id).unwrap().as_str().unwrap().to_owned()
                }).collect::<Vec<_>>();
                expected.sort();
                // Repeated inputs must not change the distinct-ID branch or results.
                ids.extend(ids.clone());
                let actual = index.fetch_list_projections("owned", &ids).unwrap();
                let mut actual_ids = actual.keys().cloned().collect::<Vec<_>>();
                actual_ids.sort();
                assert_eq!(actual_ids, expected, "segments={segments}, IDs={count}");
                assert!(!actual.contains_key("foreign"));
            }
            drop(searcher);
            let duplicate = serde_json::to_string(&doc("duplicate", "owned")).unwrap();
            insert_raw(&index, "duplicate", "owned", Some(&duplicate));
            insert_raw(&index, "duplicate", "owned", Some(&duplicate));
            for count in [32, 33] {
                let mut ids = (0..count-1).map(|n| format!("missing-{n}")).collect::<Vec<_>>();
                ids.push("duplicate".into());
                assert!(index.fetch_list_projections("owned", &ids).unwrap().is_empty());
            }
        }
    }
    #[test]
    fn thin_projection_keeps_index_doc_required_types_and_legacy_defaults() {
        let value = serde_json::to_value(doc("a", "owned")).unwrap();
        // Also catch future IndexDoc schema additions: missing and mistyped
        // serialized fields must keep the same acceptance rules automatically.
        for field in value.as_object().unwrap().keys() {
            let mut missing = value.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert_eq!(serde_json::from_value::<IndexDoc>(missing.clone()).is_ok(),serde_json::from_value::<ListProjection>(missing).is_ok(),"missing {field}");
            let mut wrong = value.clone();
            wrong[field] = json!({"wrong":true});
            assert_eq!(serde_json::from_value::<IndexDoc>(wrong.clone()).is_ok(),serde_json::from_value::<ListProjection>(wrong).is_ok(),"mistyped {field}");
        }
        for required in [
            "stableId",
            "accountId",
            "mailboxId",
            "from",
            "to",
            "cc",
            "subject",
            "body",
            "date",
            "hasAttachment",
            "keywords",
            "size",
            "filenames",
            "pinned",
        ] {
            let mut missing = value.clone();
            missing.as_object_mut().unwrap().remove(required);
            assert!(
                serde_json::from_value::<IndexDoc>(missing.clone()).is_err(),
                "{required}"
            );
            assert!(
                serde_json::from_value::<ListProjection>(missing).is_err(),
                "{required}"
            );
            let mut wrong = value.clone();
            wrong[required] = json!({"wrong":true});
            assert!(
                serde_json::from_value::<IndexDoc>(wrong.clone()).is_err(),
                "{required}"
            );
            assert!(
                serde_json::from_value::<ListProjection>(wrong).is_err(),
                "{required}"
            );
        }
        let mut legacy = value.clone();
        for optional in [
            "fromDomains",
            "sentAt",
            "fromAddress",
            "attachmentText",
            "calendar",
            "listSignal",
        ] {
            legacy.as_object_mut().unwrap().remove(optional);
        }
        assert!(serde_json::from_value::<IndexDoc>(legacy.clone()).is_ok());
        assert!(serde_json::from_value::<ListProjection>(legacy).is_ok());
        for optional in [
            "fromDomains",
            "sentAt",
            "fromAddress",
            "attachmentText",
            "calendar",
            "listSignal",
        ] {
            let mut wrong = value.clone();
            wrong[optional] = json!(false);
            assert_eq!(
                serde_json::from_value::<IndexDoc>(wrong.clone()).is_ok(),
                serde_json::from_value::<ListProjection>(wrong).is_ok(),
                "{optional}"
            );
        }
    }
    #[test]
    fn projections_batch_owned_ids_once_and_preserve_legacy_field_values() {
        let index = Index::open_in_ram().unwrap();
        index
            .upsert_batch(&[
                doc("a", "owned"),
                doc("b", "owned"),
                doc("foreign", "other"),
            ])
            .unwrap();
        let before = index.doc_fetch_count();
        for id in ["a", "b", "a", "b"] {
            assert!(index.fetch_doc(id).unwrap().is_some());
        }
        assert_eq!(index.doc_fetch_count() - before, 4);
        let before = index.list_projection_search_count();
        let result = index
            .fetch_list_projections(
                "owned",
                &[
                    "a".into(),
                    "b".into(),
                    "a".into(),
                    "foreign".into(),
                    "missing".into(),
                ],
            )
            .unwrap();
        assert_eq!(index.list_projection_search_count() - before, 1);
        assert_eq!(result.len(), 2);
        assert_eq!(index.list_projection_doc_read_count(), 2);
        for id in ["a", "b"] {
            let old = index.fetch_doc(id).unwrap().unwrap();
            assert_eq!(result[id].keywords, old.keywords);
            assert_eq!(result[id].list_signal, old.list_signal);
            assert_eq!(result[id].calendar, old.calendar);
        }
    }
    #[test]
    fn indexed_and_decoded_bindings_missing_and_malformed_rows_are_not_projections() {
        let index = Index::open_in_ram().unwrap();
        insert_raw(
            &index,
            "wrong-account",
            "owned",
            Some(&serde_json::to_string(&doc("wrong-account", "other")).unwrap()),
        );
        insert_raw(
            &index,
            "wrong-id",
            "owned",
            Some(&serde_json::to_string(&doc("different-id", "owned")).unwrap()),
        );
        insert_raw(
            &index,
            "foreign-term",
            "other",
            Some(&serde_json::to_string(&doc("foreign-term", "owned")).unwrap()),
        );
        insert_raw(&index, "no-json", "owned", None);
        insert_raw(&index, "malformed", "owned", Some("{bad"));
        let mut missing = serde_json::to_value(doc("missing-field", "owned")).unwrap();
        missing.as_object_mut().unwrap().remove("body");
        insert_raw(&index, "missing-field", "owned", Some(&missing.to_string()));
        let ids = [
            "wrong-account",
            "wrong-id",
            "foreign-term",
            "no-json",
            "malformed",
            "missing-field",
        ]
        .map(str::to_owned);
        assert!(
            index
                .fetch_list_projections("owned", &ids)
                .unwrap()
                .is_empty()
        );
        assert!(index.fetch_doc("missing-field").unwrap().is_none());
    }
    #[test]
    fn projection_searches_are_chunked_and_empty_input_does_not_search() {
        let index = Index::open_in_ram().unwrap();
        assert!(
            index
                .fetch_list_projections("owned", &[])
                .unwrap()
                .is_empty()
        );
        assert_eq!(index.list_projection_search_count(), 0);
        let ids = (0..=MAX_LIST_PROJECTION_IDS)
            .map(|n| format!("missing-{n}"))
            .collect::<Vec<_>>();
        assert!(
            index
                .fetch_list_projections("owned", &ids)
                .unwrap()
                .is_empty()
        );
        assert_eq!(index.list_projection_search_count(), 2);
    }
}
