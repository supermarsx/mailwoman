//! Host adapter for shared latency observations and durable metric aggregation.
pub(crate) use mw_observability::latency::*;
pub(crate) fn bind_metrics_persistence() {
    install_persistence(Persistence {
        active: crate::metrics_persist::active,
        value: |now, metric, value| {
            crate::metrics_persist::value(
                now,
                metric,
                value,
                &mw_store::metrics_bucket::Dims::default(),
            );
        },
    });
}

pub(crate) async fn track(
    fut: impl std::future::Future<Output = axum::response::Response>,
) -> axum::response::Response {
    mw_observability::latency::track(fut, |response| response.status().as_u16()).await
}
