//! t55: a panic inside the in-process memory layer (moka) must not take the
//! cache down: it fails only the operation that hit it, the poisoned moka
//! instance is replaced, and the next operation works.
use super::*;
use std::sync::atomic::Ordering as AtomicOrdering;

fn small_cache(capacity: u64) -> Cache {
    Cache::build(ScopeMatrix::spec_defaults(), None, false, None, capacity)
}

/// Run `fut` on its own task and report whether it panicked.
async fn panics<F>(fut: F) -> bool
where
    F: std::future::Future + Send + 'static,
    F::Output: Send,
{
    tokio::spawn(fut).await.is_err_and(|e| e.is_panic())
}

// Why the containment exists (moka 0.12.15 `future/housekeeper.rs:162-200`):
// moka keeps its running maintenance task in `current_task` as a
// `futures::Shared` and clears it only after the task completes. A panic inside
// the task (the frequency-sketch overflow) leaves the poisoned Shared stored,
// and every later maintenance run awaits it again and panics with "inner
// future panicked during poll", for the life of the process. Public moka API
// offers no way to inject a panic there (evicted values are dropped later by
// crossbeam-epoch, listener panics are caught by moka), so the test below
// injects the panic inside the operation `Cache` runs, which is where moka's
// maintenance runs.

/// The same panic through `Cache`: the operation that hit it answers (as a
/// miss), the panic is counted, the layer is replaced, and later reads and
/// writes work.
#[tokio::test]
async fn a_memory_layer_panic_fails_only_that_operation_and_the_next_one_works() {
    let cache = small_cache(4);
    for i in 0..16 {
        cache.mem_put(&namespaced(CacheClass::Sessions, &format!("k{i}")), vec![1], 0).await;
    }
    let before = memory_layer_panics();
    cache.inner.panic_once.store(true, AtomicOrdering::SeqCst);
    // Maintenance runs inside these calls; none of them may panic.
    let c = cache.clone();
    assert!(!panics(async move { c.run_pending_memory_tasks().await }).await);
    assert!(memory_layer_panics() > before, "the contained panic is counted");
    assert!(
        cache.inner.layer(Layer::General).0 >= 1 || cache.inner.layer(Layer::Mail).0 >= 1,
        "the panicked instance was replaced"
    );
    for round in 0..3 {
        let c = cache.clone();
        let ok = tokio::spawn(async move {
            let key = namespaced(CacheClass::Sessions, &format!("after{round}"));
            c.mem_put(&key, b"value".to_vec(), 0).await;
            c.run_pending_memory_tasks().await;
            c.inner.memory_get(Layer::General, &key).await.is_some()
        })
        .await;
        assert_eq!(ok.ok(), Some(true), "round {round}: the memory layer did not recover");
    }
}

/// `Cache::get` keeps answering from its loader while and after the memory
/// layer fails: a request is never failed by the advisory tier.
#[tokio::test]
async fn cache_get_answers_from_the_loader_through_a_memory_layer_panic() {
    let cache = small_cache(4);
    for i in 0..16 {
        cache.mem_put(&namespaced(CacheClass::Sessions, &format!("k{i}")), vec![1], 0).await;
    }
    cache.inner.panic_once.store(true, AtomicOrdering::SeqCst);
    for round in 0..20 {
        let c = cache.clone();
        let got = tokio::spawn(async move {
            c.get(CacheClass::Sessions, &format!("q{round}"), || async move { Ok::<_, CacheError>(round) })
                .await
                .ok()
        })
        .await;
        assert_eq!(got.ok().flatten(), Some(round), "round {round}");
        cache.run_pending_memory_tasks().await;
    }
}

/// Drive the memory layer the way a large, mostly-unique working set does:
/// inserts, reads of recent keys, and the maintenance runs that apply them to
/// moka's frequency sketch. The cache keeps answering throughout.
#[tokio::test]
async fn a_mostly_unique_working_set_keeps_the_memory_layer_working() {
    let cache = small_cache(129);
    for i in 0..50_000usize {
        let key = namespaced(CacheClass::Sessions, &format!("k{i}"));
        cache.mem_put(&key, vec![1u8; 8], 0).await;
        let _ = cache.inner.memory_get(Layer::General, &namespaced(CacheClass::Sessions, &format!("k{}", i / 2))).await;
        if i % 64 == 0 {
            cache.run_pending_memory_tasks().await;
        }
    }
    let key = namespaced(CacheClass::Sessions, "after");
    cache.mem_put(&key, b"value".to_vec(), 0).await;
    assert!(cache.inner.memory_get(Layer::General, &key).await.is_some());
}
