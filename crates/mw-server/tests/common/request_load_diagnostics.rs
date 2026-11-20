//! Fixture-only adapter. The baseline overlay substitutes a compatible adapter
//! without changing any production backend source.
pub async fn snapshot(store: &mw_store::Store, directory: &std::path::Path) -> serde_json::Value {
    serde_json::json!({
        "pool":store.database_pool_snapshot().await,
        "resources":mw_observability::resource_snapshot::snapshot(directory),
        "requestProfile":mw_server::fixture_profile::snapshot(),
        "profileEnabled":mw_server::fixture_profile::enabled(),
        "adapter":"current"
    })
}
pub fn reset() {
    mw_server::fixture_profile::reset();
}
