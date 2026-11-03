//! Stable account/message gates with a replica-wide revision floor. Evicting an
//! idle gate must not let a delayed request resurrect an older, larger ceiling.
use super::{adaptive::Runtime, capacity::Capacity};
use std::{collections::HashMap, sync::Arc};
#[derive(Default)]
pub(crate) struct Slots {
    entries: HashMap<[u8; 32], Slot>,
    runtime: Option<Runtime>,
}
struct Slot {
    requested: usize,
    limit: usize,
    account: bool,
    revision: u64,
    capacity: Arc<Capacity>,
}
impl Slots {
    pub(crate) fn get(
        &mut self,
        key: [u8; 32],
        requested: usize,
        account: bool,
        revision: u64,
    ) -> Arc<Capacity> {
        if self.entries.len() >= 1024 {
            self.entries.retain(|_, slot| {
                Arc::strong_count(&slot.capacity) > 1
                    || slot.capacity.available_permits() != slot.limit
            });
        }
        let ceiling = self
            .runtime
            .map_or(if account { 256 } else { 128 }, |runtime| {
                if account {
                    runtime.effective.per_account
                } else {
                    runtime.effective.per_message
                }
            });
        let current_revision = revision.max(self.runtime.map_or(0, |runtime| runtime.revision));
        let requested = requested.max(1);
        let slot = self.entries.entry(key).or_insert_with(|| Slot {
            requested,
            limit: requested.min(ceiling).max(1),
            account,
            revision: current_revision,
            capacity: Capacity::new(requested.min(ceiling).max(1)),
        });
        if revision >= slot.revision {
            slot.requested = requested;
            slot.revision = current_revision;
        }
        let limit = slot.requested.min(ceiling).max(1);
        if limit != slot.limit {
            slot.capacity.configure(limit);
            slot.limit = limit;
        }
        slot.capacity.clone()
    }
    pub(crate) fn advance(&mut self, runtime: Runtime) {
        if self
            .runtime
            .is_some_and(|previous| previous.revision >= runtime.revision)
        {
            return;
        }
        self.runtime = Some(runtime);
        for slot in self.entries.values_mut() {
            let ceiling = if slot.account {
                runtime.effective.per_account
            } else {
                runtime.effective.per_message
            };
            let limit = slot.requested.min(ceiling).max(1);
            if limit != slot.limit {
                slot.capacity.configure(limit);
                slot.limit = limit;
            }
            slot.revision = runtime.revision;
        }
    }
    pub(crate) fn waiting(&self) -> usize {
        self.entries
            .values()
            .map(|slot| slot.capacity.waiting())
            .sum()
    }
}
#[cfg(test)]
mod tests {
    use super::super::{adaptive::Limits, resources::ImageResources};
    use super::*;
    fn runtime(revision: u64, message: usize) -> Runtime {
        Runtime {
            revision,
            resources: ImageResources {
                cpu_millis: 16000,
                memory_limit_bytes: None,
                memory_available_bytes: None,
            },
            effective: Limits {
                per_message: message,
                per_account: message * 2,
                global: 128,
                pipeline: 128,
                decoders: 10,
            },
        }
    }
    #[tokio::test]
    async fn stale_requests_and_pruned_keys_cannot_restore_pre_reduction_capacity() {
        let mut slots = Slots::default();
        slots.advance(runtime(1, 30));
        let first = slots.get([1; 32], 30, false, 1);
        let held = first.acquire().await.unwrap();
        slots.advance(runtime(2, 2));
        let stale = slots.get([1; 32], 30, false, 1);
        assert!(Arc::ptr_eq(&first, &stale));
        let last = stale.acquire().await.unwrap();
        assert!(stale.try_acquire().is_err());
        drop(last);
        drop(held);
        slots.entries.clear();
        let recreated = slots.get([1; 32], 30, false, 1);
        assert_eq!(recreated.available_permits(), 2);
        let account = slots.get([2; 32], 128, true, 1);
        assert_eq!(account.available_permits(), 4);
    }
    #[tokio::test]
    async fn resource_recovery_wakes_existing_batch_without_a_new_request() {
        let mut slots = Slots::default();
        slots.advance(runtime(1, 2));
        let capacity = slots.get([1; 32], 96, false, 1);
        let first = capacity.acquire().await.unwrap();
        let second = capacity.acquire().await.unwrap();
        let waiting = {
            let capacity = capacity.clone();
            tokio::spawn(async move { capacity.acquire().await.unwrap() })
        };
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while slots.waiting() != 1 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(slots.waiting(), 1);
        slots.advance(runtime(2, 30));
        let third = tokio::time::timeout(std::time::Duration::from_secs(1), waiting)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(slots.waiting(), 0);
        drop(first);
        drop(second);
        drop(third);
        assert_eq!(capacity.available_permits(), 30);
    }
}
