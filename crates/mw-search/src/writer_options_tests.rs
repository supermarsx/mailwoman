use super::*;

#[test]
fn sealed_options_preserve_encryption_reopen_and_schema_recovery() {
    let root=std::env::temp_dir().join(format!("mw-sealed-options-{}",std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let key=mw_store::ServerKey::generate();
    let options=|workers|WriterOptions{indexing_workers:Some(workers),..Default::default()};
    let invalid=root.join("invalid");
    assert!(Index::open_sealed_with_writer_options(&invalid,key.clone(),options(4)).is_err());
    assert!(!invalid.exists(),"invalid options must not create files");
    std::fs::create_dir_all(&root).unwrap();
    // A genuine legacy stale schema must take the same encrypted recovery path.
    {
        let mut schema=Schema::builder();schema.add_text_field("old",TEXT);
        let old=TantivyIndex::create_in_dir(&root,schema.build()).unwrap();
        let mut writer=old.writer_with_num_threads::<TantivyDocument>(1,15_000_000).unwrap();
        writer.commit().unwrap();
    }
    let expected=IndexDoc{stable_id:"private".into(),account_id:"owned".into(),body:"uniquesealedoptionssentinel".into(),..Default::default()};
    {
        let index=Index::open_sealed_with_writer_options(&root,key.clone(),options(1)).unwrap();
        assert_eq!(index.writer_configuration().indexing_workers,1);
        index.upsert(&expected).unwrap();
    }
    let meta=std::fs::read(root.join("meta.json")).unwrap();
    assert!(meta.starts_with(b"MWB1"));
    for entry in std::fs::read_dir(&root).unwrap(){
        let entry=entry.unwrap();if !entry.file_type().unwrap().is_file(){continue;}
        let bytes=std::fs::read(entry.path()).unwrap();
        assert!(!bytes.windows(b"uniquesealedoptionssentinel".len()).any(|window|window==b"uniquesealedoptionssentinel"));
    }
    assert!(Index::open_sealed_with_writer_options(&root,mw_store::ServerKey::generate(),options(3)).is_err());
    assert_eq!(std::fs::read(root.join("meta.json")).unwrap(),meta,"wrong key must not rewrite metadata");
    {
        let index=Index::open_sealed_with_writer_options(&root,key.clone(),options(3)).unwrap();
        assert_eq!(index.writer_configuration().indexing_workers,3);
        assert_eq!(index.fetch_doc("private").unwrap(),Some(expected));
    }
    let mut corrupt=std::fs::read(root.join("meta.json")).unwrap();
    let last=corrupt.len()-1;corrupt[last]^=1;
    std::fs::write(root.join("meta.json"),&corrupt).unwrap();
    assert!(Index::open_sealed_with_writer_options(&root,key,options(1)).is_err());
    assert_eq!(std::fs::read(root.join("meta.json")).unwrap(),corrupt);
    std::fs::remove_dir_all(root).unwrap();
}

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
