use super::*;

fn admission(requests: usize, wait_ms: u64, queue_max: usize) -> Arc<Admission> {
    Admission::new(&HttpLimits {
        requests,
        queue_wait: Duration::from_millis(wait_ms),
        queue_max,
        ..HttpLimits::default()
    })
}

fn count(admission: &Admission, key: &str) -> u64 {
    admission.snapshot()[key].as_u64().unwrap()
}

#[tokio::test]
async fn waiters_are_served_in_arrival_order() {
    let admission = admission(1, 2000, 16);
    let first = admission.admit(Pool::Requests).await.unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    for i in 0..5 {
        let waiter = admission.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let lease = waiter.admit(Pool::Requests).await.unwrap();
            tx.send(i).unwrap();
            tokio::time::sleep(Duration::from_millis(5)).await;
            drop(lease);
        });
        // Each waiter is queued before the next one arrives.
        while admission.waiting() < i + 1 {
            tokio::task::yield_now().await;
        }
    }
    drop(first);
    let mut order = Vec::new();
    for _ in 0..5 {
        order.push(rx.recv().await.unwrap());
    }
    assert_eq!(order, vec![0, 1, 2, 3, 4]);
    assert_eq!(count(&admission, "admittedAfterWait"), 5);
    assert_eq!(count(&admission, "admittedImmediately"), 1);
}

#[tokio::test]
async fn a_newcomer_cannot_take_a_permit_freed_for_a_waiter() {
    let admission = admission(1, 2000, 16);
    let held = admission.admit(Pool::Requests).await.unwrap();
    let waiter = {
        let admission = admission.clone();
        tokio::spawn(async move { admission.admit(Pool::Requests).await.map(drop) })
    };
    while admission.waiting() == 0 {
        tokio::task::yield_now().await;
    }
    drop(held);
    // The freed permit belongs to the waiter; a try at this instant finds none.
    assert!(admission.requests.clone().try_acquire_owned().is_err());
    waiter.await.unwrap().unwrap();
}

#[tokio::test]
async fn the_deadline_refuses_and_zero_wait_refuses_at_once() {
    let admission_ = admission(1, 50, 16);
    let _held = admission_.admit(Pool::Requests).await.unwrap();
    let started = Instant::now();
    assert_eq!(
        admission_.admit(Pool::Requests).await.err(),
        Some(Refusal::Deadline)
    );
    assert!(started.elapsed() >= Duration::from_millis(45));
    assert_eq!(admission_.waiting(), 0);
    assert_eq!(count(&admission_, "refusedOnDeadline"), 1);

    let immediate = admission(1, 0, 16);
    let _held = immediate.admit(Pool::Requests).await.unwrap();
    let started = Instant::now();
    assert_eq!(
        immediate.admit(Pool::Requests).await.err(),
        Some(Refusal::Deadline)
    );
    assert!(started.elapsed() < Duration::from_millis(40));
}

#[tokio::test]
async fn a_full_queue_refuses_at_once() {
    let admission = admission(1, 2000, 2);
    let _held = admission.admit(Pool::Requests).await.unwrap();
    let mut waiters = Vec::new();
    for _ in 0..2 {
        let admission = admission.clone();
        waiters.push(tokio::spawn(async move {
            admission.admit(Pool::Requests).await.map(drop)
        }));
    }
    while admission.waiting() < 2 {
        tokio::task::yield_now().await;
    }
    let started = Instant::now();
    assert_eq!(
        admission.admit(Pool::Requests).await.err(),
        Some(Refusal::QueueFull)
    );
    assert!(started.elapsed() < Duration::from_millis(40));
    assert_eq!(admission.waiting(), 2, "the refused request took no slot");
    assert_eq!(count(&admission, "refusedOnFullQueue"), 1);
    for waiter in waiters {
        waiter.abort();
    }
}

#[tokio::test]
async fn a_cancelled_wait_frees_its_slot_and_its_place() {
    let admission = admission(1, 2000, 1);
    let held = admission.admit(Pool::Requests).await.unwrap();
    let cancelled = {
        let admission = admission.clone();
        tokio::spawn(async move { admission.admit(Pool::Requests).await.map(drop) })
    };
    while admission.waiting() == 0 {
        tokio::task::yield_now().await;
    }
    cancelled.abort();
    let _ = cancelled.await;
    assert_eq!(admission.waiting(), 0);
    // The slot is free again: a new waiter is accepted and gets the permit.
    let next = {
        let admission = admission.clone();
        tokio::spawn(async move { admission.admit(Pool::Requests).await.map(drop) })
    };
    while admission.waiting() == 0 {
        tokio::task::yield_now().await;
    }
    drop(held);
    next.await.unwrap().unwrap();
    assert_eq!(count(&admission, "refusedOnFullQueue"), 0);
}

#[tokio::test]
async fn probes_and_streams_are_not_starved_by_a_full_request_pool() {
    let admission = admission(1, 2000, 1);
    let _held = admission.admit(Pool::Requests).await.unwrap();
    let _queued = {
        let admission = admission.clone();
        tokio::spawn(async move { admission.admit(Pool::Requests).await.map(drop) })
    };
    while admission.waiting() == 0 {
        tokio::task::yield_now().await;
    }
    let started = Instant::now();
    let probes: Vec<_> =
        futures_util::future::join_all((0..PROBE_PERMITS).map(|_| admission.admit(Pool::Probes)))
            .await;
    assert!(probes.iter().all(Result::is_ok));
    assert!(admission.admit(Pool::Streams).await.is_ok());
    assert!(started.elapsed() < Duration::from_millis(40));
    // The probe pool itself is bounded.
    assert_eq!(
        admission.admit(Pool::Probes).await.err(),
        Some(Refusal::Probes)
    );
}

#[test]
fn classification() {
    assert_eq!(classify("/jmap/eventsource"), Pool::Streams);
    assert_eq!(classify("/healthz"), Pool::Probes);
    assert_eq!(classify("/readyz"), Pool::Probes);
    assert_eq!(classify("/x/healthz"), Pool::Requests);
    assert_eq!(classify("/jmap/api"), Pool::Requests);
}
