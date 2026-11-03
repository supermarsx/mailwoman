//! A stable, hot-resizable semaphore. Shrinking drains existing permits rather
//! than replacing the semaphore and accidentally admitting a second full pool.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::{AcquireError, OwnedSemaphorePermit, Semaphore, TryAcquireError};

pub(crate) struct Capacity {
    semaphore: Arc<Semaphore>,
    sizing: Mutex<Sizing>,
    waiting: AtomicUsize,
}
struct Sizing {
    limit: usize,
    retiring: usize,
}
pub(crate) struct Permit {
    capacity: Arc<Capacity>,
    permit: Option<OwnedSemaphorePermit>,
}

impl Capacity {
    pub(crate) fn new(limit: usize) -> Arc<Self> {
        let limit = limit.max(1);
        Arc::new(Self {
            semaphore: Arc::new(Semaphore::new(limit)),
            sizing: Mutex::new(Sizing { limit, retiring: 0 }),
            waiting: AtomicUsize::new(0),
        })
    }

    pub(crate) fn configure(&self, limit: usize) {
        let limit = limit.max(1);
        let mut sizing = self.sizing.lock().expect("image capacity lock");
        if limit > sizing.limit {
            let growth = limit - sizing.limit;
            let restored = growth.min(sizing.retiring);
            sizing.retiring -= restored;
            self.semaphore.add_permits(growth - restored);
        } else if limit < sizing.limit {
            let reduction = sizing.limit - limit;
            let removed = self.semaphore.forget_permits(reduction);
            sizing.retiring += reduction - removed;
        }
        sizing.limit = limit;
    }

    pub(crate) fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    pub(crate) fn active(&self) -> usize {
        let sizing = self.sizing.lock().expect("image capacity lock");
        sizing
            .limit
            .saturating_add(sizing.retiring)
            .saturating_sub(self.semaphore.available_permits())
    }
    pub(crate) fn waiting(&self) -> usize {
        self.waiting.load(Ordering::Relaxed)
    }

    pub(crate) fn try_acquire(self: &Arc<Self>) -> Result<Permit, TryAcquireError> {
        loop {
            let permit = self.semaphore.clone().try_acquire_owned()?;
            if let Some(permit) = self.promote(permit) {
                return Ok(permit);
            }
        }
    }

    pub(crate) async fn acquire(self: &Arc<Self>) -> Result<Permit, AcquireError> {
        struct Waiting<'a>(&'a AtomicUsize);
        impl Drop for Waiting<'_> {
            fn drop(&mut self) {
                self.0.fetch_sub(1, Ordering::Relaxed);
            }
        }
        self.waiting.fetch_add(1, Ordering::Relaxed);
        let _waiting = Waiting(&self.waiting);
        loop {
            let permit = self.semaphore.clone().acquire_owned().await?;
            if let Some(permit) = self.promote(permit) {
                return Ok(permit);
            }
        }
    }

    fn promote(self: &Arc<Self>, permit: OwnedSemaphorePermit) -> Option<Permit> {
        let mut sizing = self.sizing.lock().expect("image capacity lock");
        // Tokio may have reserved a permit for a pending acquisition before a
        // shrink. Cancellation can return it without passing through our Drop.
        // Retire that debt before exporting any newly acquired raw permit.
        if sizing.retiring > 0 {
            permit.forget();
            sizing.retiring -= 1;
            return None;
        }
        Some(Permit {
            capacity: self.clone(),
            permit: Some(permit),
        })
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        let mut sizing = self.capacity.sizing.lock().expect("image capacity lock");
        if sizing.retiring > 0 {
            if let Some(permit) = self.permit.take() {
                permit.forget();
                sizing.retiring -= 1;
            }
        }
        // Keep the sizing lock until returning the permit, so a concurrent
        // shrink cannot miss a permit between debt accounting and release.
        drop(self.permit.take());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn growing_wakes_waiters_without_replacing_active_capacity() {
        let capacity = Capacity::new(1);
        let first = capacity.acquire().await.unwrap();
        let waiting = {
            let capacity = capacity.clone();
            tokio::spawn(async move { capacity.acquire().await.unwrap() })
        };
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished());
        capacity.configure(2);
        let second = tokio::time::timeout(Duration::from_secs(1), waiting)
            .await
            .unwrap()
            .unwrap();
        assert!(capacity.try_acquire().is_err());
        drop(first);
        drop(second);
        assert_eq!(capacity.available_permits(), 2);
    }

    #[tokio::test]
    async fn shrinking_drains_old_permits_before_admitting_waiters() {
        let capacity = Capacity::new(4);
        let mut held = Vec::new();
        for _ in 0..4 {
            held.push(capacity.acquire().await.unwrap());
        }
        capacity.configure(1);
        for _ in 0..3 {
            drop(held.pop());
            assert!(capacity.try_acquire().is_err());
        }
        drop(held.pop());
        assert_eq!(capacity.available_permits(), 1);
        let permit = capacity.acquire().await.unwrap();
        assert!(capacity.try_acquire().is_err());
        drop(permit);
    }

    #[tokio::test]
    async fn growing_during_a_shrink_cancels_debt_without_oversubscription() {
        let capacity = Capacity::new(4);
        let mut held = Vec::new();
        for _ in 0..4 {
            held.push(capacity.acquire().await.unwrap());
        }
        capacity.configure(1);
        drop(held.pop());
        capacity.configure(3);
        assert!(capacity.try_acquire().is_err());
        drop(held.pop());
        assert_eq!(capacity.available_permits(), 1);
        held.clear();
        assert_eq!(capacity.available_permits(), 3);
    }

    #[tokio::test]
    async fn cancelled_waiters_do_not_consume_capacity() {
        let capacity = Capacity::new(1);
        let held = capacity.acquire().await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(10), capacity.acquire())
                .await
                .is_err()
        );
        drop(held);
        assert_eq!(capacity.available_permits(), 1);
    }

    #[tokio::test]
    async fn cancelled_reserved_waiters_cannot_restore_capacity_after_a_shrink() {
        use std::{
            future::Future,
            task::{Context, Waker},
        };
        let capacity = Capacity::new(4);
        let mut held = Vec::new();
        for _ in 0..4 {
            held.push(capacity.acquire().await.unwrap());
        }
        let mut pending: Vec<_> = (0..4).map(|_| Box::pin(capacity.acquire())).collect();
        let mut context = Context::from_waker(Waker::noop());
        for future in &mut pending {
            assert!(future.as_mut().poll(&mut context).is_pending());
        }
        held.clear();
        capacity.configure(1);
        pending.clear();
        let last = capacity.try_acquire().unwrap();
        assert!(capacity.try_acquire().is_err());
        drop(last);
        assert_eq!(capacity.available_permits(), 1);
    }
}
