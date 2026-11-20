use super::*;

#[test]
fn default_writer_resolution_matches_locked_tantivy_memory_clamp() {
    for (available, expected) in [(1, 1), (2, 2), (3, 3), (4, 3), (8, 3), (80, 3)] {
        assert_eq!(
            WriterOptions::default().resolve(available).unwrap(),
            WriterConfiguration {
                indexing_workers: expected,
                memory_budget_bytes: 50_000_000,
            }
        );
    }
    assert_eq!(
        WriterOptions {
            memory_budget_bytes: 15_000_000,
            ..Default::default()
        }
        .resolve(80)
        .unwrap()
        .indexing_workers,
        1
    );
    assert_eq!(
        WriterOptions {
            memory_budget_bytes: 128_000_000,
            ..Default::default()
        }
        .resolve(80)
        .unwrap()
        .indexing_workers,
        8
    );
}

#[test]
fn invalid_worker_options_fail_before_tantivy_allocation() {
    for (indexing_workers, memory_budget_bytes) in [
        (Some(0), 50_000_000),
        (Some(9), 128_000_000),
        (Some(4), 50_000_000),
        (Some(8), 119_999_999),
        (Some(1), 14_999_999),
        (None, 128_000_001),
    ] {
        let options = WriterOptions {
            indexing_workers,
            memory_budget_bytes,
        };
        assert!(options.validate().is_err());
        assert!(Index::open_in_ram_with_writer_options(options).is_err());
    }
    assert!(
        WriterOptions {
            indexing_workers: Some(8),
            memory_budget_bytes: 120_000_000
        }
        .validate()
        .is_ok()
    );
}

#[test]
fn explicit_workers_preserve_commit_publication_and_full_document_values() {
    for workers in [1, 3] {
        let index = Index::open_in_ram_with_writer_options(WriterOptions {
            indexing_workers: Some(workers),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            index.writer_configuration(),
            WriterConfiguration {
                indexing_workers: workers,
                memory_budget_bytes: 50_000_000,
            }
        );
        let mut original = IndexDoc {
            stable_id: "owned".into(),
            account_id: "account".into(),
            mailbox_id: "inbox".into(),
            body: "A complete indexed body remains visible after every commit".into(),
            ..Default::default()
        };
        let foreign = IndexDoc {
            stable_id: "foreign".into(),
            account_id: "other".into(),
            ..Default::default()
        };
        let before = commit_count();
        index
            .upsert_batch(&[original.clone(), foreign.clone()])
            .unwrap();
        assert_eq!(commit_count() - before, 1);
        assert_eq!(index.fetch_doc("owned").unwrap(), Some(original.clone()));
        original.keywords = vec!["$seen".into()];
        original.pinned = true;
        let before = commit_count();
        index.upsert(&original).unwrap();
        assert_eq!(commit_count() - before, 1);
        assert_eq!(index.fetch_doc("owned").unwrap(), Some(original));
        assert_eq!(index.fetch_doc("foreign").unwrap(), Some(foreign));
    }
}
