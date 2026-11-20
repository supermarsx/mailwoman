//! Bounded CPU-only jobs. Accepted jobs own admission until real completion.
//! Timeouts stop waiting, not an in-process parser; no I/O belongs on workers.
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex, OnceLock, mpsc},
    time::Duration,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, oneshot};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CpuError {
    Unavailable,
    Budget,
    Timeout,
    Panicked,
}
#[derive(Clone)]
pub(crate) struct CpuExecutor(Arc<Producer>);
struct Producer {
    sender: mpsc::SyncSender<Job>,
    jobs: Arc<Semaphore>,
    bytes: Arc<Semaphore>,
    limit: usize,
    finish: Duration,
}
struct Admission {
    _jobs: OwnedSemaphorePermit,
    _bytes: OwnedSemaphorePermit,
}
struct Job {
    run: Box<dyn FnOnce() + Send>,
    _admission: Admission,
}
impl CpuExecutor {
    fn new(workers: usize, jobs: usize, bytes: usize, finish: Duration) -> Result<Self, CpuError> {
        if workers == 0
            || workers > 2
            || jobs < workers
            || jobs > 64
            || bytes == 0
            || bytes > 512 * 1024 * 1024
        {
            return Err(CpuError::Budget);
        }
        let (sender, receiver) = mpsc::sync_channel::<Job>(jobs);
        let receiver = Arc::new(Mutex::new(receiver));
        for index in 0..workers {
            let receiver = receiver.clone();
            std::thread::Builder::new()
                .name(format!("mw-dlp-cpu-{index}"))
                .spawn(move || {
                    loop {
                        let job = match receiver.lock() {
                            Ok(receiver) => receiver.recv(),
                            Err(_) => return,
                        };
                        let Ok(job) = job else { return };
                        // A client timeout/cancellation cannot drop this admission while
                        // the CPU operation is still running.
                        (job.run)();
                    }
                })
                .map_err(|_| CpuError::Unavailable)?;
        }
        Ok(Self(Arc::new(Producer {
            sender,
            jobs: Arc::new(Semaphore::new(jobs)),
            bytes: Arc::new(Semaphore::new(bytes)),
            limit: bytes,
            finish,
        })))
    }
    pub(crate) async fn run<T: Send + 'static>(
        &self,
        bytes: usize,
        operation: impl FnOnce() -> T + Send + 'static,
    ) -> Result<T, CpuError> {
        if bytes == 0 || bytes > self.0.limit {
            return Err(CpuError::Budget);
        }
        // The operation already owns its input. Never enqueue an unbounded
        // collection of waiting futures retaining large raw buffers.
        let jobs = self
            .0
            .jobs
            .clone()
            .try_acquire_owned()
            .map_err(|_| CpuError::Unavailable)?;
        let bytes = self
            .0
            .bytes
            .clone()
            .try_acquire_many_owned(bytes as u32)
            .map_err(|_| CpuError::Unavailable)?;
        let (tx, rx) = oneshot::channel();
        let job = Job {
            run: Box::new(move || {
                let result =
                    catch_unwind(AssertUnwindSafe(operation)).map_err(|_| CpuError::Panicked);
                let _ = tx.send(result);
            }),
            _admission: Admission {
                _jobs: jobs,
                _bytes: bytes,
            },
        };
        self.0
            .sender
            .try_send(job)
            .map_err(|_| CpuError::Unavailable)?;
        tokio::time::timeout(self.0.finish, rx)
            .await
            .map_err(|_| CpuError::Timeout)?
            .map_err(|_| CpuError::Unavailable)?
    }
}
/// Bound owned DLP inputs/policy working space across asynchronous policy/Store
/// waits too; not just buffers already executing on the CPU workers.
#[derive(Clone)]
pub(crate) struct InputBudget {
    _permit: Arc<OwnedSemaphorePermit>,
}
pub(crate) fn input_budget(bytes: usize) -> Result<InputBudget, CpuError> {
    static BYTES: OnceLock<Arc<Semaphore>> = OnceLock::new();
    if bytes == 0 || bytes > 256 * 1024 * 1024 {
        return Err(CpuError::Budget);
    }
    BYTES
        .get_or_init(|| Arc::new(Semaphore::new(256 * 1024 * 1024)))
        .clone()
        .try_acquire_many_owned(bytes as u32)
        .map(|permit| InputBudget {
            _permit: Arc::new(permit),
        })
        .map_err(|_| CpuError::Unavailable)
}
pub(crate) fn shared() -> Result<&'static CpuExecutor, CpuError> {
    static EXECUTOR: OnceLock<Result<CpuExecutor, CpuError>> = OnceLock::new();
    EXECUTOR
        .get_or_init(|| {
            CpuExecutor::new(
                std::thread::available_parallelism()
                    .map_or(1, usize::from)
                    .clamp(1, 2),
                16,
                256 * 1024 * 1024,
                Duration::from_secs(30),
            )
        })
        .as_ref()
        .map_err(|error| *error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    #[tokio::test]
    async fn panic_is_failure_and_next_job_still_executes() {
        let pool = CpuExecutor::new(1, 2, 128, Duration::from_secs(2)).unwrap();
        assert_eq!(
            pool.run(1, || panic!("fixture panic")).await,
            Err(CpuError::Panicked)
        );
        assert_eq!(pool.run(1, || 42).await, Ok(42));
    }
    #[tokio::test]
    async fn cancellation_retains_real_job_and_byte_admission_without_blocking_async_heartbeat() {
        let pool = CpuExecutor::new(1, 2, 64, Duration::from_secs(2)).unwrap();
        let budget = input_budget(64).unwrap();
        let weak_budget = Arc::downgrade(&budget._permit);
        let worker_budget = budget.clone();
        let (entered_tx, entered_rx) = oneshot::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let task = {
            let pool = pool.clone();
            tokio::spawn(async move {
                pool.run(64, move || {
                    let _ = entered_tx.send(());
                    release_rx.recv().unwrap();
                    worker_budget
                })
                .await
            })
        };
        entered_rx.await.unwrap();
        task.abort();
        let _ = task.await;
        drop(budget);
        assert!(
            weak_budget.upgrade().is_some(),
            "shared input token remains owned by running canceled job"
        );
        assert_eq!(pool.0.bytes.available_permits(), 0);
        let ran = Arc::new(AtomicBool::new(false));
        let flag = ran.clone();
        assert_eq!(
            pool.run(1, move || flag.store(true, Ordering::SeqCst))
                .await,
            Err(CpuError::Unavailable)
        );
        assert!(!ran.load(Ordering::SeqCst));
        // On a single-thread async runtime this timer progresses even though
        // the dedicated worker remains blocked at the deterministic gate.
        tokio::time::timeout(
            Duration::from_secs(1),
            tokio::time::sleep(Duration::from_millis(1)),
        )
        .await
        .unwrap();
        release_tx.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while pool.0.bytes.available_permits() != 64 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(
            weak_budget.upgrade().is_none(),
            "shared input token releases after the real job/result is dropped"
        );
        assert_eq!(pool.run(65, || 0).await, Err(CpuError::Budget));
        assert_eq!(pool.run(64, || 7).await, Ok(7));
    }
    #[tokio::test]
    async fn timeout_and_job_limit_keep_execution_admission_until_real_finish() {
        let pool = CpuExecutor::new(1, 1, 128, Duration::from_millis(20)).unwrap();
        let (entered_tx, entered_rx) = oneshot::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let task = {
            let pool = pool.clone();
            tokio::spawn(async move {
                pool.run(64, move || {
                    let _ = entered_tx.send(());
                    release_rx.recv().unwrap();
                })
                .await
            })
        };
        entered_rx.await.unwrap();
        assert_eq!(task.await.unwrap(), Err(CpuError::Timeout));
        assert_eq!(pool.0.jobs.available_permits(), 0);
        assert_eq!(pool.0.bytes.available_permits(), 64);
        assert_eq!(pool.run(1, || 0).await, Err(CpuError::Unavailable));
        release_tx.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while pool.0.jobs.available_permits() != 1 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(pool.run(1, || 1).await, Ok(1));
    }
}
