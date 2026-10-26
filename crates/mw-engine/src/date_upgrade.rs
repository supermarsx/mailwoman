//! One successful cache-date upgrade per account and engine lifetime.
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};
use tokio::sync::OnceCell;
#[derive(Default)]
pub(super) struct DateUpgrade {
    accounts: Mutex<HashMap<String, Arc<OnceCell<()>>>>,
}
impl DateUpgrade {
    pub(super) async fn ensure<F, Fut, E>(&self, account: &str, upgrade: F) -> Result<(), E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(), E>>,
    {
        let cell = self
            .accounts
            .lock()
            .expect("date upgrade lock")
            .entry(account.to_owned())
            .or_default()
            .clone();
        // Errors and cancelled initialization do not poison the cell. Other
        // callers wait for the same account's upgrade instead of rescanning.
        cell.get_or_try_init(upgrade).await.map(|_| ())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[tokio::test]
    async fn concurrent_queries_upgrade_once_and_accounts_remain_separate() {
        let guard = Arc::new(DateUpgrade::default());
        let count = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();
        for _ in 0..16 {
            let guard = guard.clone();
            let count = count.clone();
            tasks.push(tokio::spawn(async move {
                guard
                    .ensure("owner", || async {
                        count.fetch_add(1, Ordering::SeqCst);
                        tokio::task::yield_now().await;
                        Ok::<(), ()>(())
                    })
                    .await
                    .unwrap()
            }));
        }
        for task in tasks {
            task.await.unwrap();
        }
        assert_eq!(count.load(Ordering::SeqCst), 1);
        guard
            .ensure("owner", || async {
                count.fetch_add(1, Ordering::SeqCst);
                Ok::<(), ()>(())
            })
            .await
            .unwrap();
        guard
            .ensure("other", || async {
                count.fetch_add(1, Ordering::SeqCst);
                Ok::<(), ()>(())
            })
            .await
            .unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }
    #[tokio::test]
    async fn failed_upgrade_is_retryable() {
        let guard = DateUpgrade::default();
        assert_eq!(
            guard
                .ensure("owner", || async { Err::<(), _>("database unavailable") })
                .await,
            Err("database unavailable")
        );
        guard
            .ensure("owner", || async { Ok::<(), ()>(()) })
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn cancelled_upgrade_does_not_block_retry() {
        let guard = Arc::new(DateUpgrade::default());
        let entered = Arc::new(tokio::sync::Notify::new());
        let task = {
            let guard = guard.clone();
            let entered = entered.clone();
            tokio::spawn(async move {
                guard
                    .ensure("owner", || async {
                        entered.notify_one();
                        std::future::pending::<Result<(), ()>>().await
                    })
                    .await
            })
        };
        entered.notified().await;
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            guard.ensure("owner", || async { Ok::<(), ()>(()) }),
        )
        .await
        .unwrap()
        .unwrap();
    }
}
