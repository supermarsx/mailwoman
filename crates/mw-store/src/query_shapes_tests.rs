use super::*;
use crate::{ServerKey, Store, q};
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
#[tokio::test]
async fn actual_select_with_failure_and_write_rows_reconcile_without_bind_values() {
    let _serial = SERIAL.lock().await;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    q("CREATE TABLE trace_calibration(id INTEGER)")
        .execute(store.backend())
        .await
        .unwrap();
    let (_, report) = measure(async {
        assert_eq!(
            q("SELECT ?1")
                .bind("private-bind-never-report")
                .fetch_one(store.backend())
                .await
                .unwrap()
                .get_string_idx(0),
            "private-bind-never-report"
        );
        assert_eq!(
            q("WITH probe(n) AS(SELECT 7 UNION ALL SELECT 8) SELECT n FROM probe")
                .fetch_all(store.backend())
                .await
                .unwrap()
                .len(),
            2
        );
        assert!(
            q("SELECT * FROM missing_trace_calibration_table")
                .fetch_all(store.backend())
                .await
                .is_err()
        );
        assert_eq!(
            q("INSERT INTO trace_calibration VALUES(1)")
                .execute(store.backend())
                .await
                .unwrap(),
            1
        );
    })
    .await;
    report.require_valid().unwrap();
    assert_eq!(report.operations_completed, 4);
    assert_eq!(report.query_counts.read.calls, 2);
    assert_eq!(report.query_counts.read.failures, 1);
    assert_eq!(report.query_counts.other.calls, 1);
    assert_eq!(report.query_counts.write.calls, 1);
    let with = report
        .shapes
        .iter()
        .find(|shape| shape.class == "other")
        .unwrap();
    assert_eq!(with.returned_rows, 2);
    assert_eq!(with.returned_rows_known_calls, 1);
    assert_eq!(
        report.shapes.iter().map(|s| s.affected_rows).sum::<u64>(),
        1
    );
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("private-bind"));
    assert!(!json.contains("missing_trace_calibration_table"));
    assert!(!json.contains("SELECT"));
    store.close().await;
}
#[tokio::test]
async fn concurrent_task_scopes_are_separate_and_fresh() {
    let _serial = SERIAL.lock().await;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let a = measure(async {
        for _ in 0..3 {
            q("SELECT 1").fetch_one(store.backend()).await.unwrap();
            tokio::task::yield_now().await;
        }
    });
    let b = measure(async {
        for _ in 0..7 {
            q("WITH probe AS(SELECT 2) SELECT * FROM probe")
                .fetch_one(store.backend())
                .await
                .unwrap();
            tokio::task::yield_now().await;
        }
    });
    let ((_, a), (_, b)) = tokio::join!(a, b);
    a.require_valid().unwrap();
    b.require_valid().unwrap();
    assert_eq!(a.operations_completed, 3);
    assert_eq!(b.operations_completed, 7);
    assert_eq!(a.shapes.len(), 1);
    assert_eq!(b.shapes.len(), 1);
    assert_eq!(
        a.shapes[0].template_sha256,
        "e004ebd5b5532a4b85984a62f8ad48a81aa3460c1ca07701f386135d72cdecf5"
    );
    assert_ne!(a.shapes[0].template_sha256, b.shapes[0].template_sha256);
    let (_, fresh) = measure(async {}).await;
    fresh.require_valid().unwrap();
    assert_eq!(fresh.operations_completed, 0);
    assert!(fresh.shapes.is_empty());
    store.close().await;
}
#[tokio::test]
async fn operation_template_and_row_counter_mismatch_never_qualify() {
    let _serial = SERIAL.lock().await;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, overflow) = measure(async {
        for _ in 0..=MAX_OPERATIONS {
            q("SELECT 1").fetch_one(store.backend()).await.unwrap();
        }
    })
    .await;
    assert!(!overflow.valid);
    assert!(overflow.require_valid().is_err());
    // Each static test literal is leaked only within this finite calibration.
    let statements = (0..=MAX_TEMPLATES)
        .map(|n| Box::leak(format!("SELECT {n}").into_boxed_str()) as &'static str)
        .collect::<Vec<_>>();
    let (_, templates) = measure(async {
        for sql in statements {
            q(sql).fetch_one(store.backend()).await.unwrap();
        }
    })
    .await;
    assert!(!templates.valid);
    assert!(templates.shapes.len() <= MAX_TEMPLATES);
    // An inner counter scope hiding work from the outer counter is rejected,
    // rather than inventing agreement from this trace's own counters.
    let (_, mismatch) = measure(super::super::measure_queries(
        q("SELECT 1").fetch_one(store.backend()),
    ))
    .await;
    assert!(!mismatch.valid);
    assert_eq!(mismatch.invalid_reason, Some("query count mismatch"));
    store.close().await;
}
#[tokio::test]
async fn fifth_live_scope_fails_closed_and_releases_admission_after_completion() {
    let _serial = SERIAL.lock().await;
    let entered = Arc::new(tokio::sync::Barrier::new(5));
    let release = Arc::new(tokio::sync::Barrier::new(5));
    let mut jobs = Vec::new();
    for _ in 0..4 {
        let entered = entered.clone();
        let release = release.clone();
        jobs.push(tokio::spawn(measure(async move {
            entered.wait().await;
            release.wait().await;
        })));
    }
    entered.wait().await;
    let (_, full) = measure(async {}).await;
    assert!(!full.valid);
    assert_eq!(full.invalid_reason, Some("scope budget exceeded"));
    release.wait().await;
    for job in jobs {
        job.await.unwrap().1.require_valid().unwrap();
    }
    measure(async {}).await.1.require_valid().unwrap();
}

#[tokio::test]
async fn static_sql_and_encoded_report_byte_caps_fail_without_payload_or_partial_success() {
    let _serial = SERIAL.lock().await;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let oversized =
        Box::leak(format!("SELECT 1 /*{}*/", "x".repeat(MAX_SQL_BYTES)).into_boxed_str())
            as &'static str;
    let (_, sql) = measure(q(oversized).fetch_one(store.backend())).await;
    assert!(!sql.valid);
    assert_eq!(sql.invalid_reason, Some("static SQL budget exceeded"));
    let tokens = TOKENS.join(" ");
    let statements = (0..MAX_TEMPLATES)
        .map(|n| Box::leak(format!("SELECT {n} /*{tokens}*/").into_boxed_str()) as &'static str)
        .collect::<Vec<_>>();
    let (_, report) = measure(async {
        for statement in statements {
            q(statement).fetch_one(store.backend()).await.unwrap();
        }
    })
    .await;
    assert!(!report.valid);
    assert_eq!(report.invalid_reason, Some("report byte budget exceeded"));
    assert!(report.shapes.is_empty());
    assert!(serde_json::to_vec(&report).unwrap().len() <= MAX_REPORT_BYTES);
    store.close().await;
}

#[tokio::test]
async fn nested_and_concurrent_phases_keep_unknown_failures_and_full_query_totals() {
    let _serial = SERIAL.lock().await;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (_, report) = measure(async {
        q("SELECT 1").fetch_one(store.backend()).await.unwrap();
        in_phase(Phase::Features, async {
            q("SELECT 1").fetch_one(store.backend()).await.unwrap();
            in_phase(Phase::Auth, q("SELECT 2").fetch_one(store.backend()))
                .await
                .unwrap();
            q("SELECT 1").fetch_one(store.backend()).await.unwrap();
        })
        .await;
        in_phase(Phase::AccountGate, async {
            assert!(
                q("SELECT absent FROM phase_fixture_missing_table")
                    .fetch_one(store.backend())
                    .await
                    .is_err()
            );
        })
        .await;
        let left = in_phase(Phase::Handler, q("SELECT 3").fetch_one(store.backend()));
        let right = in_phase(Phase::Engine, q("SELECT 3").fetch_one(store.backend()));
        let (left, right) = tokio::join!(left, right);
        left.unwrap();
        right.unwrap();
    })
    .await;
    report.require_valid().unwrap();
    assert_eq!(report.operations_completed, 7);
    let phase = |name| report.phases.iter().find(|row| row.phase == name).unwrap();
    assert_eq!(phase("unknown").completed, 1);
    assert_eq!(phase("guard.features").completed, 2);
    assert_eq!(phase("auth.total").completed, 1);
    assert_eq!(phase("auth.account-gate").failures, 1);
    assert_eq!(phase("jmap.handler").completed, 1);
    assert_eq!(phase("jmap.engine").completed, 1);
    assert_eq!(
        report.phases.iter().map(|row| row.completed).sum::<u64>(),
        7
    );
    assert_eq!(report.query_counts.read.calls, 7);
    for shape in &report.shapes {
        assert_eq!(
            shape
                .phase_calls
                .iter()
                .map(|phase| phase.calls)
                .sum::<u64>(),
            shape.calls
        );
    }
    assert!(
        !serde_json::to_string(&report)
            .unwrap()
            .contains("phase_fixture_missing_table")
    );
    store.close().await;
}

#[tokio::test]
async fn query_phase_is_captured_before_acquisition_await_not_at_completion() {
    let _serial = SERIAL.lock().await;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let crate::backend::Backend::Sqlite(live) = &store.backend else {
        unreachable!()
    };
    let pin = live.pin().await;
    let mut lease = pin.pool().acquire().await.unwrap();
    let (_, report) = measure(async {
        let mut waiting = Box::pin(q("SELECT 42").fetch_one(store.backend()));
        in_phase(
            Phase::Engine,
            std::future::poll_fn(|cx| {
                assert!(
                    waiting.as_mut().poll(cx).is_pending(),
                    "actual sole serving lease is held"
                );
                std::task::Poll::Ready(())
            }),
        )
        .await;
        lease.return_to_pool().await;
        in_phase(Phase::Auth, waiting).await.unwrap();
    })
    .await;
    report.require_valid().unwrap();
    assert_eq!(report.operations_completed, 1);
    assert_eq!(
        report
            .phases
            .iter()
            .find(|row| row.phase == "jmap.engine")
            .unwrap()
            .completed,
        1
    );
    assert_eq!(
        report
            .phases
            .iter()
            .find(|row| row.phase == "auth.total")
            .unwrap()
            .completed,
        0
    );
    assert_eq!(report.shapes[0].phase_calls[0].phase, "jmap.engine");
    drop(pin);
    store.close().await;
}

#[tokio::test]
async fn phase_tags_cannot_create_a_scope_or_attribute_detached_child_queries() {
    let _serial = SERIAL.lock().await;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    assert!(!active());
    in_phase(Phase::Engine, q("SELECT 1").fetch_one(store.backend()))
        .await
        .unwrap();
    assert!(!active());
    let (_, report) = measure(async {
        let child_store = store.clone();
        in_phase(Phase::Auth, async move {
            tokio::spawn(async move {
                assert!(!active());
                q("SELECT 2")
                    .fetch_one(child_store.backend())
                    .await
                    .unwrap();
            })
            .await
            .unwrap();
        })
        .await;
        q("SELECT 3").fetch_one(store.backend()).await.unwrap();
    })
    .await;
    report.require_valid().unwrap();
    assert_eq!(report.operations_completed, 1);
    assert_eq!(report.query_counts.read.calls, 1);
    assert_eq!(
        report
            .phases
            .iter()
            .find(|row| row.phase == "unknown")
            .unwrap()
            .completed,
        1
    );
    assert!(!active());
    store.close().await;
}
