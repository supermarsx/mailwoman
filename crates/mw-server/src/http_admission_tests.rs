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
    let first = admission.admit(Pool::Requests, true, "/x").await.unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    for i in 0..5 {
        let waiter = admission.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let lease = waiter.admit(Pool::Requests, true, "/x").await.unwrap();
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
    let held = admission.admit(Pool::Requests, true, "/x").await.unwrap();
    let waiter = {
        let admission = admission.clone();
        tokio::spawn(async move { admission.admit(Pool::Requests, true, "/x").await.map(drop) })
    };
    while admission.waiting() == 0 {
        tokio::task::yield_now().await;
    }
    drop(held);
    // The freed permit belongs to the waiter; a try at this instant finds none.
    assert!(admission.request_pool().try_acquire().is_none());
    waiter.await.unwrap().unwrap();
}

#[tokio::test]
async fn the_deadline_refuses_and_zero_wait_refuses_at_once() {
    let admission_ = admission(1, 50, 16);
    let _held = admission_.admit(Pool::Requests, true, "/x").await.unwrap();
    let started = Instant::now();
    assert_eq!(
        admission_.admit(Pool::Requests, true, "/x").await.err(),
        Some(Refusal::Deadline)
    );
    assert!(started.elapsed() >= Duration::from_millis(45));
    assert_eq!(admission_.waiting(), 0);
    assert_eq!(count(&admission_, "refusedOnDeadline"), 1);

    let immediate = admission(1, 0, 16);
    let _held = immediate.admit(Pool::Requests, true, "/x").await.unwrap();
    let started = Instant::now();
    assert_eq!(
        immediate.admit(Pool::Requests, true, "/x").await.err(),
        Some(Refusal::Deadline)
    );
    assert!(started.elapsed() < Duration::from_millis(40));
}

#[tokio::test]
async fn a_full_queue_refuses_at_once() {
    let admission = admission(1, 2000, 2);
    let _held = admission.admit(Pool::Requests, true, "/x").await.unwrap();
    let mut waiters = Vec::new();
    for _ in 0..2 {
        let admission = admission.clone();
        waiters.push(tokio::spawn(async move {
            admission.admit(Pool::Requests, true, "/x").await.map(drop)
        }));
    }
    while admission.waiting() < 2 {
        tokio::task::yield_now().await;
    }
    let started = Instant::now();
    assert_eq!(
        admission.admit(Pool::Requests, true, "/x").await.err(),
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
    let held = admission.admit(Pool::Requests, true, "/x").await.unwrap();
    let cancelled = {
        let admission = admission.clone();
        tokio::spawn(async move { admission.admit(Pool::Requests, true, "/x").await.map(drop) })
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
        tokio::spawn(async move { admission.admit(Pool::Requests, true, "/x").await.map(drop) })
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
    let _held = admission.admit(Pool::Requests, true, "/x").await.unwrap();
    let _queued = {
        let admission = admission.clone();
        tokio::spawn(async move { admission.admit(Pool::Requests, true, "/x").await.map(drop) })
    };
    while admission.waiting() == 0 {
        tokio::task::yield_now().await;
    }
    let started = Instant::now();
    let probes: Vec<_> = futures_util::future::join_all(
        (0..PROBE_PERMITS).map(|_| admission.admit(Pool::Probes, false, "/x")),
    )
    .await;
    assert!(probes.iter().all(Result::is_ok));
    assert!(admission.admit(Pool::Streams, false, "/x").await.is_ok());
    assert!(started.elapsed() < Duration::from_millis(40));
    // The probe pool itself is bounded.
    assert_eq!(
        admission.admit(Pool::Probes, false, "/x").await.err(),
        Some(Refusal::Probes)
    );
}

#[test]
fn classification() {
    assert_eq!(classify("/jmap/eventsource"), (Pool::Streams, false));
    assert_eq!(classify("/healthz"), (Pool::Probes, false));
    assert_eq!(classify("/readyz"), (Pool::Probes, false));
    assert_eq!(classify("/x/healthz"), (Pool::Requests, true));
    assert_eq!(classify("/jmap/api"), (Pool::Requests, true));
    for slow in [
        "/jmap/download/a/b/c",
        "/jmap/upload/a",
        "/jmap/ws",
        "/jmap/actions",
        "/api/export/1",
        "/api/import/mbox",
        "/api/attachments/preview",
        "/api/image-proxy",
        "/api/mail/refresh",
        "/dav/calendars/a/b",
        "/api/privacy/requests/7/download",
    ] {
        assert_eq!(classify(slow), (Pool::Requests, false), "{slow}");
    }
}

#[test]
fn admin_paths_match_the_path_or_below_it_never_a_shared_prefix() {
    assert!(is_admin_path("/ops-7f", "/ops-7f"));
    assert!(is_admin_path("/ops-7f/api/settings", "/ops-7f"));
    assert!(!is_admin_path("/ops-7fx", "/ops-7f"));
    assert!(!is_admin_path("/ops-7f.css", "/ops-7f"));
    assert!(!is_admin_path("/mail/ops-7f", "/ops-7f"));
    assert!(!is_admin_path("/anything", ""));
}

#[tokio::test]
async fn only_admin_paths_reach_the_reserve_when_the_pool_is_full() {
    let admission = Admission::with_admin_path(
        &HttpLimits {
            requests: 1,
            queue_wait: Duration::ZERO,
            queue_max: 1,
            ..HttpLimits::default()
        },
        "/ops-7f".into(),
    );
    let _held = admission.admit(Pool::Requests, true, "/x").await.unwrap();
    for outsider in ["/x", "/ops-7fx", "/ops-7f.css", "/mail/ops-7f"] {
        assert_eq!(
            admission.admit(Pool::Requests, true, outsider).await.err(),
            Some(Refusal::Deadline),
            "{outsider}"
        );
    }
    let admins: Vec<_> = futures_util::future::join_all(
        (0..ADMIN_PERMITS).map(|_| admission.admit(Pool::Requests, true, "/ops-7f/api/settings")),
    )
    .await;
    assert!(admins.iter().all(Result::is_ok));
    assert_eq!(
        count(&admission, "admittedFromAdminReserve"),
        ADMIN_PERMITS as u64
    );
    // The reserve is bounded too.
    assert!(
        admission
            .admit(Pool::Requests, true, "/ops-7f")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn console_values_resize_the_pool_without_cutting_off_requests() {
    let admission = admission(4, 0, 4);
    let held: Vec<_> = futures_util::future::join_all(
        (0..4).map(|_| admission.admit(Pool::Requests, false, "/x")),
    )
    .await
    .into_iter()
    .map(Result::unwrap)
    .collect();
    admission.apply(Console {
        requests: Some(2),
        ..Console::default()
    });
    assert_eq!(admission.live().0.requests, 2);
    assert_eq!(admission.request_pool().size(), 2);
    // The four in progress keep their slots; two come back as permits.
    drop(held);
    assert_eq!(admission.request_pool().available(), 2);
    admission.apply(Console {
        requests: Some(6),
        queue_wait_ms: Some(0),
        ..Console::default()
    });
    assert_eq!(admission.request_pool().available(), 6);
    assert_eq!(
        admission.live().1.requests,
        super::super::limits::Source::Console
    );
}

#[tokio::test]
async fn the_adaptive_mode_switches_at_runtime_and_stays_under_the_request_limit() {
    let admission = admission(10, 0, 10);
    assert!(admission.adaptive_limit().is_none());
    admission.apply(Console {
        adaptive: Some(true),
        adaptive_floor: Some(3),
        ..Console::default()
    });
    assert_eq!(admission.adaptive_limit(), Some(10));
    assert_eq!(admission.snapshot()["adaptive"]["floor"], 3);
    // A lower request limit lowers the adaptive ceiling with it.
    admission.apply(Console {
        adaptive: Some(true),
        adaptive_floor: Some(3),
        requests: Some(5),
        ..Console::default()
    });
    assert_eq!(admission.adaptive_limit(), Some(5));
    assert_eq!(admission.request_pool().size(), 5);
    admission.apply(Console::default());
    assert!(admission.adaptive_limit().is_none());
    assert_eq!(admission.request_pool().size(), 10);
}

#[tokio::test]
async fn adaptive_off_has_no_state_and_samples_nothing() {
    let admission = admission(4, 100, 4);
    let mut lease = admission.admit(Pool::Requests, true, "/x").await.unwrap();
    lease.headers_ready();
    assert!(admission.adaptive_limit().is_none());
    assert!(admission.snapshot()["adaptive"].is_null());
}
