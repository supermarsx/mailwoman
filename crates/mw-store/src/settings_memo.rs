//! Revision-validated settings LRU. Accounted allocation and entry count are
//! bounded; allocator metadata and in-flight clones are not an RSS ceiling.
use super::{CanonicalSettingSubject, SettingPolicyRow, SettingValueRow};
use hashlink::LinkedHashMap;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const READ_TTL: Duration = Duration::from_secs(60);
const READ_CAPACITY: usize = 32_768;
const BYTE_CAPACITY: usize = 64 * 1024 * 1024;
const MAX_ENTRY_BYTES: usize = 2 * 1024 * 1024;

pub(super) trait MemoFootprint {
    fn heap_bytes(&self) -> usize;
}
impl MemoFootprint for bool {
    fn heap_bytes(&self) -> usize { 0 }
}
impl MemoFootprint for Vec<String> {
    fn heap_bytes(&self) -> usize {
        self.capacity() * std::mem::size_of::<String>() + self.iter().map(String::capacity).sum::<usize>()
    }
}
impl MemoFootprint for CanonicalSettingSubject {
    fn heap_bytes(&self) -> usize {
        self.personal_owner.as_ref().map_or(0, String::capacity)
            + self.domain.capacity()
            + self.account_id.capacity()
            + self.address.capacity()
    }
}
fn policy_heap(row: &SettingPolicyRow) -> usize {
    row.key.capacity()
        + row.updated_by.capacity()
        + row.global_value.as_ref().map_or(0, String::capacity)
}
impl MemoFootprint for Option<SettingPolicyRow> {
    fn heap_bytes(&self) -> usize {
        self.as_ref().map_or(0, policy_heap)
    }
}
impl MemoFootprint for Vec<SettingPolicyRow> {
    fn heap_bytes(&self) -> usize {
        self.capacity() * std::mem::size_of::<SettingPolicyRow>()
            + self.iter().map(policy_heap).sum::<usize>()
    }
}
impl MemoFootprint for Vec<SettingValueRow> {
    fn heap_bytes(&self) -> usize {
        self.capacity() * std::mem::size_of::<SettingValueRow>()
            + self
                .iter()
                .map(|row| {
                    row.scope_id.capacity()
                        + row.key.capacity()
                        + row.value.capacity()
                        + row.updated_by.capacity()
                })
                .sum::<usize>()
    }
}

struct Entry<V> {
    value: Arc<V>,
    revision: i64,
    inserted: Instant,
    weight: usize,
}
struct Entries<V> {
    map: LinkedHashMap<(uuid::Uuid, String), Entry<V>>,
    bytes: usize,
}
impl<V> Default for Entries<V> {
    fn default() -> Self {
        Self {
            map: LinkedHashMap::new(),
            bytes: 0,
        }
    }
}
pub(super) struct ReadMemo<V>(Mutex<Option<Entries<V>>>);
impl<V: Clone + MemoFootprint> ReadMemo<V> {
    pub(super) const fn new() -> Self {
        Self(Mutex::new(None))
    }
    pub(super) fn get(&self, scope: uuid::Uuid, key: &str, revision: i64) -> Option<V> {
        let probe = (scope, key.to_owned());
        let mut retired = None;
        let value = {
            let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
            let entries = guard.get_or_insert_with(Default::default);
            match entries.map.get(&probe) {
                Some(entry)
                    if entry.revision == revision && entry.inserted.elapsed() < READ_TTL =>
                {
                    let value = entry.value.clone();
                    entries.map.to_back(&probe);
                    Some(value)
                }
                // Older requests must neither evict nor refresh newer snapshots.
                Some(entry) if entry.revision > revision && entry.inserted.elapsed() < READ_TTL => {
                    None
                }
                Some(_) => {
                    retired = entries.map.remove_entry(&probe);
                    if let Some((_, entry)) = &retired {
                        entries.bytes -= entry.weight;
                    }
                    None
                }
                None => None,
            }
        };
        // Both destructing an expired string tree and cloning a live one must
        // happen outside the global memo mutex.
        drop(retired);
        value.map(|value| value.as_ref().clone())
    }
    pub(super) fn put(&self, scope: uuid::Uuid, key: String, revision: i64, value: V) {
        self.put_bounded(scope, key, revision, value, READ_CAPACITY);
    }
    pub(super) fn put_bounded(
        &self,
        scope: uuid::Uuid,
        key: String,
        revision: i64,
        value: V,
        capacity: usize,
    ) {
        self.put_with_limits(scope, key, revision, value, capacity, BYTE_CAPACITY);
    }
    fn put_with_limits(
        &self,
        scope: uuid::Uuid,
        key: String,
        revision: i64,
        value: V,
        capacity: usize,
        budget: usize,
    ) {
        let capacity = capacity.min(READ_CAPACITY);
        let budget = budget.min(BYTE_CAPACITY);
        let weight = key
            .capacity()
            .saturating_add(value.heap_bytes())
            .saturating_add(std::mem::size_of::<V>() + std::mem::size_of::<Entry<V>>() + 144);
        if capacity == 0 || weight > budget || weight > MAX_ENTRY_BYTES {
            return;
        }
        // Allocate the Arc outside the mutex. Retired entries retain their
        // keys and values until after unlock. The usual no-eviction insertion
        // needs no retirement-buffer allocation.
        let value = Arc::new(value);
        let mut retired_one = None;
        let mut retired_many = Vec::new();
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let entries = guard.get_or_insert_with(Default::default);
        let probe = (scope, key);
        if entries
            .map
            .get(&probe)
            .is_some_and(|entry| entry.revision > revision && entry.inserted.elapsed() < READ_TTL)
        {
            return;
        }
        if let Some(old) = entries.map.remove_entry(&probe) {
            entries.bytes -= old.1.weight;
            retired_one = Some(old);
        }
        while entries.map.len() >= capacity || entries.bytes.saturating_add(weight) > budget {
            let Some(old) = entries.map.pop_front() else {
                break;
            };
            entries.bytes -= old.1.weight;
            if retired_one.is_none() {
                retired_one = Some(old);
            } else {
                retired_many.push(old);
            }
        }
        entries.map.insert(
            probe,
            Entry {
                value,
                revision,
                inserted: Instant::now(),
                weight,
            },
        );
        entries.bytes += weight;
        drop(guard);
        drop(retired_one);
        drop(retired_many);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    impl MemoFootprint for String {
        fn heap_bytes(&self) -> usize {
            self.capacity()
        }
    }
    #[test]
    fn ten_thousand_subjects_remain_hot_without_global_flushes() {
        let memo = ReadMemo::<String>::new();
        let scope = uuid::Uuid::new_v4();
        for user in 0..10_000 {
            memo.put(scope, user.to_string(), 7, format!("value-{user}"));
        }
        for user in 0..10_000 {
            assert_eq!(
                memo.get(scope, &user.to_string(), 7),
                Some(format!("value-{user}"))
            );
        }
        assert_eq!(memo.0.lock().unwrap().as_ref().unwrap().map.len(), 10_000);
    }
    #[test]
    fn pressure_evicts_lru_and_updates_do_not_flush() {
        let memo = ReadMemo::<String>::new();
        let scope = uuid::Uuid::new_v4();
        for key in ["a", "b", "c"] {
            memo.put_bounded(scope, key.into(), 1, key.into(), 3);
        }
        assert_eq!(memo.get(scope, "a", 1).as_deref(), Some("a"));
        memo.put_bounded(scope, "d".into(), 1, "d".into(), 3);
        assert_eq!(memo.get(scope, "b", 1), None);
        memo.put_bounded(scope, "a".into(), 1, "updated".into(), 3);
        for key in ["a", "c", "d"] {
            assert!(memo.get(scope, key, 1).is_some());
        }
    }
    #[test]
    fn byte_budget_and_large_values_bound_admission() {
        let memo = ReadMemo::<String>::new();
        let scope = uuid::Uuid::new_v4();
        for key in 0..100 {
            memo.put_with_limits(scope, key.to_string(), 1, "x".repeat(1024), 100, 16 * 1024);
        }
        let guard = memo.0.lock().unwrap();
        let entries = guard.as_ref().unwrap();
        assert!(entries.bytes <= 16 * 1024 && entries.map.len() > 1 && entries.map.len() < 100);
        assert_eq!(
            entries.bytes,
            entries
                .map
                .values()
                .map(|entry| entry.weight)
                .sum::<usize>()
        );
        drop(guard);
        memo.put(scope, "oversized".into(), 1, "x".repeat(MAX_ENTRY_BYTES));
        assert_eq!(memo.get(scope, "oversized", 1), None);
    }
    #[test]
    fn revisions_scope_and_expiry_preserve_freshness_and_accounting() {
        let memo = ReadMemo::<String>::new();
        let scope = uuid::Uuid::new_v4();
        memo.put(scope, "key".into(), 2, "new".into());
        assert!(memo.get(scope, "key", 1).is_none());
        memo.put(scope, "key".into(), 1, "old".into());
        assert_eq!(memo.get(scope, "key", 2).as_deref(), Some("new"));
        assert!(memo.get(uuid::Uuid::new_v4(), "key", 2).is_none());
        memo.0
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .map
            .get_mut(&(scope, "key".into()))
            .unwrap()
            .inserted = Instant::now() - READ_TTL;
        assert!(memo.get(scope, "key", 2).is_none());
        assert_eq!(memo.0.lock().unwrap().as_ref().unwrap().bytes, 0);
    }
    fn assert_accounting<V: Clone + MemoFootprint>(memo: &ReadMemo<V>) {
        let guard = memo.0.lock().unwrap();
        let entries = guard.as_ref().unwrap();
        assert_eq!(
            entries.bytes,
            entries
                .map
                .values()
                .map(|entry| entry.weight)
                .sum::<usize>()
        );
    }
    #[test]
    fn stale_proofs_do_not_refresh_lru_ttl_or_accounted_weight() {
        let memo = ReadMemo::<String>::new();
        let scope = uuid::Uuid::new_v4();
        memo.put_bounded(scope, "a".into(), 2, "fresh-a".into(), 2);
        memo.put_bounded(scope, "b".into(), 2, "fresh-b".into(), 2);
        let inserted = memo
            .0
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .map
            .get(&(scope, "a".into()))
            .unwrap()
            .inserted;
        assert!(memo.get(scope, "a", 1).is_none());
        memo.put_bounded(scope, "a".into(), 1, "stale".repeat(100), 2);
        assert_eq!(
            memo.0
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .map
                .get(&(scope, "a".into()))
                .unwrap()
                .inserted,
            inserted
        );
        memo.put_bounded(scope, "c".into(), 2, "fresh-c".into(), 2);
        assert!(
            memo.get(scope, "a", 2).is_none(),
            "old requests must not make a newer entry recent"
        );
        assert_eq!(memo.get(scope, "b", 2).as_deref(), Some("fresh-b"));
        assert_eq!(memo.get(scope, "c", 2).as_deref(), Some("fresh-c"));
        assert_accounting(&memo);
    }
    #[test]
    fn replacement_expiry_and_multi_eviction_keep_exact_accounting() {
        let memo = ReadMemo::<String>::new();
        let scope = uuid::Uuid::new_v4();
        for id in 0..8 {
            memo.put_with_limits(scope, id.to_string(), 1, "x".repeat(700), 16, 8192);
            assert_accounting(&memo);
        }
        memo.put_with_limits(scope, "replacement".into(), 1, "x".repeat(6000), 16, 8192);
        assert_accounting(&memo);
        assert!(memo.0.lock().unwrap().as_ref().unwrap().map.len() < 8);
        memo.put_with_limits(scope, "replacement".into(), 2, "small".into(), 16, 8192);
        assert_accounting(&memo);
        memo.0
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .map
            .get_mut(&(scope, "replacement".into()))
            .unwrap()
            .inserted = Instant::now() - READ_TTL - Duration::from_secs(1);
        assert!(memo.get(scope, "replacement", 2).is_none());
        assert_accounting(&memo);
    }
    #[test]
    fn caller_supplied_limits_cannot_raise_hard_entry_or_byte_ceilings() {
        let memo = ReadMemo::<String>::new();
        let scope = uuid::Uuid::new_v4();
        for id in 0..READ_CAPACITY + 2 {
            memo.put_with_limits(scope, id.to_string(), 1, "x".into(), usize::MAX, usize::MAX);
        }
        assert_eq!(
            memo.0.lock().unwrap().as_ref().unwrap().map.len(),
            READ_CAPACITY
        );
        assert!(memo.get(scope, "0", 1).is_none());
        assert_accounting(&memo);
        let bytes = ReadMemo::<String>::new();
        for id in 0..70 {
            bytes.put_with_limits(
                scope,
                id.to_string(),
                1,
                "x".repeat(1024 * 1024),
                usize::MAX,
                usize::MAX,
            );
        }
        let guard = bytes.0.lock().unwrap();
        let entries = guard.as_ref().unwrap();
        assert!(entries.bytes <= BYTE_CAPACITY && entries.map.len() < 70);
        assert_eq!(
            entries.bytes,
            entries
                .map
                .values()
                .map(|entry| entry.weight)
                .sum::<usize>()
        );
    }
    struct CallbackValue {
        cloned: Arc<dyn Fn() + Send + Sync>,
        dropped: Arc<dyn Fn() + Send + Sync>,
    }
    impl Clone for CallbackValue {
        fn clone(&self) -> Self {
            (self.cloned)();
            Self {
                cloned: self.cloned.clone(),
                dropped: self.dropped.clone(),
            }
        }
    }
    impl Drop for CallbackValue {
        fn drop(&mut self) {
            (self.dropped)();
        }
    }
    impl MemoFootprint for CallbackValue {
        fn heap_bytes(&self) -> usize {
            0
        }
    }
    #[test]
    fn clones_and_retired_value_destructors_can_reenter_the_unlocked_memo() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let memo = Arc::new(ReadMemo::<CallbackValue>::new());
        let scope = uuid::Uuid::new_v4();
        let clones = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let value = || {
            let weak = Arc::downgrade(&memo);
            let clone_count = clones.clone();
            let cloned: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                if let Some(memo) = weak.upgrade() {
                    assert!(memo.0.try_lock().is_ok(), "deep clone held memo mutex");
                }
                clone_count.fetch_add(1, Ordering::SeqCst);
            });
            let weak = Arc::downgrade(&memo);
            let drop_count = drops.clone();
            let dropped: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                if let Some(memo) = weak.upgrade() {
                    assert!(
                        memo.0.try_lock().is_ok(),
                        "retired value dropped while memo mutex held"
                    );
                }
                drop_count.fetch_add(1, Ordering::SeqCst);
            });
            CallbackValue { cloned, dropped }
        };
        memo.put_bounded(scope, "a".into(), 1, value(), 1);
        drop(memo.get(scope, "a", 1));
        assert_eq!(clones.load(Ordering::SeqCst), 1);
        memo.put_bounded(scope, "a".into(), 2, value(), 1); // replace
        memo.put_bounded(scope, "b".into(), 2, value(), 1); // evict
        memo.0
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .map
            .get_mut(&(scope, "b".into()))
            .unwrap()
            .inserted = Instant::now() - READ_TTL - Duration::from_secs(1);
        assert!(memo.get(scope, "b", 2).is_none()); // expire
        assert!(drops.load(Ordering::SeqCst) >= 4);
        assert_accounting(&memo);
    }
}
