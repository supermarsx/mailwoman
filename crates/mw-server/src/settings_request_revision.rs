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
/// One parsed guard input per process, valid while the request's settings
/// revision equals the one it was read under (t50hp, after migration 0123:
/// every change of the four guard keys in the key-value `settings` table bumps
/// `settings_revision` in the same transaction, from any process or raw SQL).
/// No TTL: a hit needs the exact revision; a missing revision never hits. Only
/// successful loads are kept. The value read after the revision is at least as
/// new as that revision; an older request never replaces a newer entry.
pub(crate) struct RevisionMemo<T>(std::sync::Mutex<Option<(uuid::Uuid, i64, std::sync::Arc<T>)>>);
impl<T> RevisionMemo<T> {
    pub(crate) const fn new() -> Self {
        Self(std::sync::Mutex::new(None))
    }
    pub(crate) fn hit_at(&self, revision: &SettingsReadRevision) -> Option<std::sync::Arc<T>> {
        let number = revision.number()?;
        let guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .as_ref()
            .filter(|(scope, at, _)| *scope == revision.scope() && *at == number)
            .map(|(_, _, value)| value.clone())
    }
    pub(crate) fn put_at(&self, revision: &SettingsReadRevision, value: std::sync::Arc<T>) {
        let Some(number) = revision.number() else { return };
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if guard
            .as_ref()
            .is_some_and(|(scope, at, _)| *scope == revision.scope() && *at > number)
        {
            return;
        }
        let old = guard.replace((revision.scope(), number, value));
        drop(guard);
        drop(old);
    }
    /// The value at `revision`, loading (and keeping) it on a miss.
    pub(crate) async fn get_or_load<E, F>(
        &self,
        revision: &SettingsReadRevision,
        load: F,
    ) -> Result<std::sync::Arc<T>, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        if let Some(hit) = self.hit_at(revision) {
            return Ok(hit);
        }
        let value = std::sync::Arc::new(load.await?);
        self.put_at(revision, value.clone());
        Ok(value)
    }
}
/// [`RevisionMemo::get_or_load`] at this request's shared revision. A request
/// running on published request inputs, or one whose revision cannot be read,
/// loads directly (today's path), so its fence accounting is unchanged.
pub(crate) async fn memoised<T, E, F>(
    memo: &RevisionMemo<T>,
    store: &Store,
    request: &mut Request,
    load: F,
) -> Result<std::sync::Arc<T>, E>
where
    F: std::future::Future<Output = Result<T, E>>,
{
    match shared(store, request).await {
        Some(revision) => memo.get_or_load(&revision, load).await,
        None => load.await.map(std::sync::Arc::new),
    }
}
/// The shared revision for a revision memo, or `None` when the request runs on
/// published inputs or the revision cannot be read.
pub(crate) async fn shared(store: &Store, request: &mut Request) -> Option<SettingsReadRevision> {
    if crate::request_inputs::active() {
        return None;
    }
    get(store, request).await.ok()
}
#[cfg(test)]
#[path = "guard_read_budget_tests.rs"]
mod guard_read_budget_tests;
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn revision_memo_hits_only_at_the_exact_revision_and_store() {
        let first = Store::open_in_memory(mw_store::ServerKey::generate()).await.unwrap();
        let second = Store::open_in_memory(mw_store::ServerKey::generate()).await.unwrap();
        let memo = RevisionMemo::<u32>::new();
        let at = first.settings_read_revision().await.unwrap();
        let loads = std::sync::atomic::AtomicU32::new(0);
        let load = |value: u32| {
            let loads = &loads;
            async move {
                loads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<_, ()>(value)
            }
        };
        assert_eq!(*memo.get_or_load(&at, load(1)).await.unwrap(), 1);
        assert_eq!(*memo.get_or_load(&at, load(2)).await.unwrap(), 1, "same revision: hit");
        // A failed load is never kept.
        let other = second.settings_read_revision().await.unwrap();
        assert!(memo.get_or_load(&other, async { Err::<u32, ()>(()) }).await.is_err());
        assert_eq!(*memo.get_or_load(&other, load(3)).await.unwrap(), 3, "another store: miss");
        // A settings write moves the revision: miss, then the new value.
        first.put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "access.protocols".into(), global_value: Some("[]".into()),
            allow_user_override: false, global_locked: true,
            updated_by: "revision-memo-test".into(), updated_at: 1,
        }).await.unwrap();
        let newer = first.settings_read_revision().await.unwrap();
        assert_ne!(newer.number(), at.number());
        assert_eq!(*memo.get_or_load(&newer, load(4)).await.unwrap(), 4);
        // An older request neither hits nor replaces the newer entry.
        assert_eq!(*memo.get_or_load(&at, load(5)).await.unwrap(), 5);
        assert_eq!(*memo.get_or_load(&newer, load(6)).await.unwrap(), 4);
        // Loads ran for 1, 3, 4 and 5 only (2 and 6 were hits).
        assert_eq!(loads.load(std::sync::atomic::Ordering::SeqCst), 4);
    }
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
