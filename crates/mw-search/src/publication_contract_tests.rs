//! Real Tantivy publication contracts. Faults are private cfg(test) seams;
//! none of these cases substitutes SQL flags for query-visible index state.
use super::*;
use crate::publication::PublicationFault;
use std::sync::Arc;

fn document(id: &str, account: &str) -> IndexDoc {
    IndexDoc {
        stable_id: id.into(), account_id: account.into(), mailbox_id: "inbox".into(),
        subject: "Publication contract".into(), body: "uniquepublicationtoken".into(),
        size: 64, ..Default::default()
    }
}

fn upsert(doc: IndexDoc) -> PublicationChange { PublicationChange::Upsert(Arc::new(doc)) }

fn ids(index: &Index, query: &str) -> Vec<String> {
    let mut result = index.search(&parse_query(query).unwrap(), 100).unwrap();
    result.sort();
    result
}

#[test]
fn receipt_acknowledges_fulltext_flags_move_and_last_duplicate_visibility() {
    let index = Index::open_in_ram().unwrap();
    let first = document("one", "owner");
    let receipt = index.publish_batch(&[upsert(first.clone())]).unwrap();
    assert_eq!(receipt.changes, 1);
    assert_eq!(receipt.searcher_generation, index.reader.searcher().generation().generation_id());
    assert_eq!(ids(&index, "body:uniquepublicationtoken in:inbox is:unread"), ["one"]);

    let mut changed = first.clone();
    changed.mailbox_id = "archive".into();
    changed.keywords = vec!["$seen".into()];
    changed.pinned = true;
    let next = index.publish_batch(&[upsert(first), upsert(changed.clone())]).unwrap();
    assert_eq!(next.changes, 2);
    assert!(next.opstamp > receipt.opstamp);
    assert_eq!(next.searcher_generation, index.reader.searcher().generation().generation_id());
    assert_eq!(index.fetch_doc("one").unwrap(), Some(changed));
    assert_eq!(index.num_docs(), 1, "duplicate IDs must not create duplicate documents");
    assert_eq!(ids(&index, "body:uniquepublicationtoken in:archive is:read pinned:true"), ["one"]);
    assert!(ids(&index, "in:inbox").is_empty());
    assert!(ids(&index, "is:unread").is_empty());
}

#[test]
fn owned_delete_cannot_remove_a_reassigned_binding_or_another_account() {
    let index = Index::open_in_ram().unwrap();
    index.publish_batch(&[upsert(document("same", "old-owner")), upsert(document("foreign", "other"))]).unwrap();
    index.publish_batch(&[upsert(document("same", "new-owner"))]).unwrap();
    index.publish_batch(&[
        PublicationChange::Delete { account_id: "old-owner".into(), stable_id: "same".into() },
        PublicationChange::Delete { account_id: "old-owner".into(), stable_id: "foreign".into() },
    ]).unwrap();
    assert_eq!(ids(&index, "body:uniquepublicationtoken"), ["foreign", "same"]);
    assert_eq!(index.fetch_doc("same").unwrap().unwrap().account_id, "new-owner");
    index.publish_batch(&[PublicationChange::Delete { account_id: "new-owner".into(), stable_id: "same".into() }]).unwrap();
    assert_eq!(ids(&index, "body:uniquepublicationtoken"), ["foreign"]);
}

#[test]
fn apply_failure_rolls_back_entire_group_before_a_later_publication() {
    let index = Index::open_in_ram().unwrap();
    let original = document("one", "owner");
    index.publish_batch(&[upsert(original.clone())]).unwrap();
    let mut changed = original.clone(); changed.mailbox_id = "archive".into();
    index.set_publication_fault(PublicationFault::ApplyAfter(1));
    assert!(index.publish_batch(&[upsert(changed), upsert(document("never", "owner"))]).is_err());
    assert!(!index.publication_is_poisoned(), "successful rollback permits a later job");
    assert_eq!(index.fetch_doc("one").unwrap(), Some(original.clone()));
    index.publish_batch(&[upsert(document("later", "owner"))]).unwrap();
    assert_eq!(index.fetch_doc("one").unwrap(), Some(original));
    assert_eq!(ids(&index, "in:inbox"), ["later", "one"]);
    assert!(index.fetch_doc("never").unwrap().is_none());
    assert!(ids(&index, "in:archive").is_empty());
}

#[test]
fn uncertain_failures_poison_all_mutations_until_explicit_recovery() {
    for fault in [PublicationFault::Commit, PublicationFault::Reload, PublicationFault::Rollback] {
        let index = Index::open_in_ram().unwrap();
        index.publish_batch(&[upsert(document("original", "owner"))]).unwrap();
        index.set_publication_fault(fault);
        assert!(index.publish_batch(&[upsert(document("failed", "owner"))]).is_err());
        assert!(index.publication_is_poisoned());
        assert!(index.publish_batch(&[upsert(document("later", "owner"))]).is_err());
        assert!(index.upsert(&document("legacy", "owner")).is_err());
        assert!(index.delete("original").is_err());
        assert!(index.relocate("original", "archive").is_err());
        index.recover_publication().unwrap();
        assert!(!index.publication_is_poisoned());
        assert_eq!(index.fetch_doc("failed").unwrap().is_some(), fault == PublicationFault::Reload,
            "reload failure follows a durable commit; other injected failures leave no committed document");
        index.publish_batch(&[upsert(document("later", "owner"))]).unwrap();
        assert!(index.fetch_doc("legacy").unwrap().is_none());
        assert_eq!(ids(&index, "in:archive"), Vec::<String>::new());
        assert!(index.fetch_doc("original").unwrap().is_some());
    }
}

#[test]
fn owner_claim_is_exclusive_and_released_without_retaining_the_index() {
    let index = Arc::new(Index::open_in_ram().unwrap());
    let first = index.claim_publication_owner().unwrap();
    assert!(index.claim_publication_owner().is_err());
    drop(first);
    let second = index.claim_publication_owner().unwrap();
    let weak = Arc::downgrade(&index);
    drop(index);
    assert!(weak.upgrade().is_none(), "claim must not create an owner/index strong cycle");
    drop(second);
}

#[test]
fn bounded_document_reads_require_owner_and_reject_both_size_limits() {
    let index = Index::open_in_ram().unwrap();
    let doc = document("one", "owner");
    index.publish_batch(&[upsert(doc.clone())]).unwrap();
    assert!(index.publication_document("foreign", "one", 1024, 8192).unwrap().is_none());
    assert!(index.publication_document("owner", "one", 63, 8192).is_err());
    assert!(index.publication_document("owner", "one", 64, 1).is_err());
    assert_eq!(index.publication_document("owner", "one", 64, 8192).unwrap(), Some(doc));
}

#[test]
fn waiter_that_passed_initial_check_cannot_mutate_after_poison() {
    let index = Arc::new(Index::open_in_ram().unwrap());
    index.publish_batch(&[upsert(document("original", "owner"))]).unwrap();
    let writer = index.writer.lock().unwrap();
    let reached = Arc::new(std::sync::Barrier::new(2));
    index.set_writer_lock_hook_for_test(reached.clone());
    let waiter_index = index.clone();
    let waiter = std::thread::spawn(move || waiter_index.publish_batch(&[upsert(document("late", "owner"))]));
    reached.wait(); // waiter has already passed the first poison check
    index.poison_publication();
    drop(writer);
    assert!(waiter.join().unwrap().is_err());
    index.recover_publication().unwrap();
    index.publish_batch(&[upsert(document("later", "owner"))]).unwrap();
    assert!(index.fetch_doc("late").unwrap().is_none());
    assert_eq!(ids(&index, "body:uniquepublicationtoken"), ["later", "original"]);
}

fn sealed_root() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("mw-publication-erasure-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&root).unwrap(); root
}
fn segment_paths(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(root).unwrap().map(|entry| entry.unwrap().path()).filter(|path| {
        path.file_name().and_then(|name| name.to_str()).is_some_and(|name| {
            let stem = name.split('.').next().unwrap_or("");
            stem.len() == 32 && stem.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
    }).collect()
}

#[test]
fn physical_erasure_retries_cleanup_with_zero_live_target_and_other_account_segment() {
    let root = sealed_root();
    {
        let index = Index::open_sealed(&root, mw_store::ServerKey::generate()).unwrap();
        index.publish_batch(&[upsert(document("target", "erased")), upsert(document("keep", "other"))]).unwrap();
        let retired = segment_paths(&root); assert!(!retired.is_empty());
        index.publish_batch(&[PublicationChange::Delete { account_id: "erased".into(), stable_id: "target".into() }]).unwrap();
        assert_eq!(index.account_doc_count("erased").unwrap(), 0);
        // A previous tombstone is not evidence that physical cleanup completed.
        if index.delete_account_physical("erased").is_err() {
            assert!(index.publication_is_poisoned(), "incomplete cleanup must explicitly pause publication");
            index.recover_publication().unwrap();
            index.delete_account_physical("erased").expect("cleanup converges without retained external readers");
        }
        assert_eq!(ids(&index, "body:uniquepublicationtoken"), ["keep"]);
        assert!(retired.iter().all(|path| !path.exists()), "retired segment files must actually be absent before physical ack");
        for path in segment_paths(&root) {
            assert!(std::fs::read(path).unwrap().starts_with(b"MWB1"), "real disk fixture retains authenticated encryption");
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn retained_searcher_requires_disk_removal_or_pending_cleanup_not_ram_revocation() {
    let root = sealed_root();
    {
        let index = Index::open_sealed(&root, mw_store::ServerKey::generate()).unwrap();
        index.publish_batch(&[upsert(document("target", "erased")), upsert(document("keep", "other"))]).unwrap();
        let retired = segment_paths(&root); assert!(!retired.is_empty());
        let held = index.reader.searcher();
        index.publish_batch(&[PublicationChange::Delete { account_id: "erased".into(), stable_id: "target".into() }]).unwrap();
        assert_eq!(index.account_doc_count("erased").unwrap(), 0);
        let cleanup = index.delete_account_physical("erased");
        if cleanup.is_ok() {
            assert!(retired.iter().all(|path| !path.exists()), "physical ack requires actual retired ciphertext-file absence even with an old reader");
            assert!(!index.publication_is_poisoned());
            eprintln!("retained-reader outcome: physical ciphertext files removed; raw old RAM snapshot remains outside disk-erasure contract");
        } else {
            assert!(index.publication_is_poisoned(), "incomplete physical cleanup must remain explicitly pending");
            eprintln!("retained-reader outcome: physical cleanup explicitly pending; release/recover/retry required");
        }
        // SealedDirectory readers own decrypted bytes. A caller retaining a raw
        // old Searcher can keep that old RAM snapshot after ciphertext files
        // are removed; physical disk erasure does not promise RAM revocation.
        assert_eq!(held.search(&index.account_query("erased"), &Count).unwrap(), 1);
        drop(held);
        if cleanup.is_err() {
            index.recover_publication().unwrap();
            index.delete_account_physical("erased").unwrap();
        }
        assert!(retired.iter().all(|path| !path.exists()));
        assert_eq!(ids(&index, "body:uniquepublicationtoken"), ["keep"]);
    }
    std::fs::remove_dir_all(root).unwrap();
}
