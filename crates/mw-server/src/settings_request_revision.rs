//! A fresh, Store-bound settings revision shared only by this HTTP request's
//! middleware. Handlers still resolve their own settings after any mutation.
use axum::extract::Request;
use mw_store::{Store, StoreError, settings_policy::SettingsReadRevision};
pub(crate) async fn get(
    store: &Store,
    request: &mut Request,
) -> Result<SettingsReadRevision, StoreError> {
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
mod tests {
    use super::*;
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
