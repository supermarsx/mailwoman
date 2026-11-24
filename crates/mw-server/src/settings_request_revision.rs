//! A fresh, Store-bound settings revision shared only by this HTTP request's
//! middleware. Handlers still resolve their own settings after any mutation.
use axum::extract::Request;
use mw_store::{Store, StoreError, settings_policy::SettingsReadRevision};
/// Establish a new policy-check boundary after asynchronous request work such
/// as body parsing. An outer middleware's earlier proof must not hide a policy
/// that was committed while this request was waiting for its body.
pub(crate) async fn refresh(
    store: &Store,
    request: &mut Request,
) -> Result<SettingsReadRevision, StoreError> {
    let revision = match crate::request_inputs::revision(store, true).await {
        Some(revision) => revision,
        None => store.settings_read_revision().await?,
    };
    request.extensions_mut().insert(revision.clone());
    Ok(revision)
}
pub(crate) async fn get(
    store: &Store,
    request: &mut Request,
) -> Result<SettingsReadRevision, StoreError> {
    if let Some(revision) = crate::request_inputs::revision(store, false).await {
        request.extensions_mut().insert(revision.clone());
        return Ok(revision);
    }
    if let Some(revision) = request
        .extensions()
        .get::<SettingsReadRevision>()
        .filter(|revision| revision.scope() == store.cache_scope())
    {
        return Ok(revision.clone());
    }
    let revision = store.settings_read_revision().await?;
    request.extensions_mut().insert(revision.clone());
    Ok(revision)
}
#[cfg(test)]
#[path = "guard_read_budget_tests.rs"]
mod guard_read_budget_tests;
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn refresh_observes_policy_committed_after_an_outer_proof() {
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let mut request = Request::new(axum::body::Body::empty());
        let outer = get(&store, &mut request).await.unwrap();
        assert!(
            !store
                .administrator_setting_present_at("access.protocols", &outer)
                .await
                .unwrap()
        );
        store
            .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                key: "access.protocols".into(),
                global_value: Some("[]".into()),
                allow_user_override: false,
                global_locked: true,
                updated_by: "revision-boundary-test".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        let feature = refresh(&store, &mut request).await.unwrap();
        assert_ne!(outer.number(), feature.number());
        assert!(
            store
                .administrator_setting_present_at("access.protocols", &feature)
                .await
                .unwrap()
        );
        assert_eq!(
            get(&store, &mut request).await.unwrap().number(),
            feature.number()
        );
        store.close().await;
        assert!(
            refresh(&store, &mut request).await.is_err(),
            "a failed fresh read must not fall back to the old proof"
        );
    }
    #[tokio::test]
    async fn proof_is_request_local_and_never_crosses_store_identity() {
        let first = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let second = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let mut request = Request::new(axum::body::Body::empty());
        let original = get(&first, &mut request).await.unwrap();
        assert_eq!(original.scope(), first.cache_scope());
        first.close().await;
        // A captured boundary proof remains usable by middleware within this request.
        assert_eq!(
            get(&first, &mut request).await.unwrap().number(),
            original.number()
        );
        let mut next = Request::new(axum::body::Body::empty());
        assert!(get(&first, &mut next).await.is_err());
        let other = get(&second, &mut request).await.unwrap();
        assert_eq!(other.scope(), second.cache_scope());
        assert_ne!(other.scope(), original.scope());
    }
}
