//! Detect unowned mutations without invalidating account-local observations
//! for normal owned publications. This never grants source/owner authority.
use std::sync::Mutex;

#[derive(Default)]
struct State { epoch: u64, pending: usize, unsupported: bool }
#[derive(Default)]
pub(crate) struct Observation(Mutex<State>);
pub(crate) struct Change<'a>(&'a Observation);

impl Observation {
    pub(crate) fn begin(&self) -> Change<'_> {
        if let Ok(mut state) = self.0.lock() {
            if let Some(next) = state.pending.checked_add(1) { state.pending = next; }
            else { state.unsupported = true; }
        }
        Change(self)
    }
    pub(crate) fn stamp(&self) -> Option<u64> {
        let state = self.0.lock().ok()?;
        (!state.unsupported && state.pending == 0).then_some(state.epoch)
    }
    pub(crate) fn unsupported(&self) {
        if let Ok(mut state) = self.0.lock() { state.unsupported = true; }
    }
}
impl Drop for Change<'_> {
    fn drop(&mut self) {
        if let Ok(mut state) = self.0.0.lock() {
            match state.epoch.checked_add(1) {
                Some(next) => state.epoch = next,
                None => state.unsupported = true,
            }
            match state.pending.checked_sub(1) {
                Some(next) => state.pending = next,
                None => state.unsupported = true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Index, IndexDoc, PublicationChange};
    use std::sync::Arc;

    fn doc(id: &str, account: &str) -> IndexDoc {
        IndexDoc { stable_id:id.into(), account_id:account.into(), body:"unique prepared body".into(), ..Default::default() }
    }

    #[test]
    fn actual_owned_publication_preserves_unknown_stamp_but_legacy_write_changes_it() {
        let index = Arc::new(Index::open_in_ram().unwrap());
        let owner = index.claim_publication_owner().unwrap();
        let original = index.unowned_projection_stamp().unwrap();
        index.publish_batch_owned(&owner, &[PublicationChange::Upsert(Arc::new(doc("a", "one")))]).unwrap();
        assert_eq!(index.unowned_projection_stamp(), Some(original));
        assert_eq!(index.fetch_doc("a").unwrap().unwrap().account_id, "one");
        index.upsert(&doc("b", "two")).unwrap();
        assert_ne!(index.unowned_projection_stamp(), Some(original));
        let changed = index.unowned_projection_stamp();
        index.reload().unwrap();
        assert_ne!(index.unowned_projection_stamp(), changed);
    }

    #[test]
    fn foreign_claim_cannot_bypass_observation_and_raw_escape_disables_it() {
        let first = Arc::new(Index::open_in_ram().unwrap());
        let second = Arc::new(Index::open_in_ram().unwrap());
        let owner = first.claim_publication_owner().unwrap();
        assert!(second.publish_batch_owned(&owner, &[PublicationChange::Upsert(Arc::new(doc("foreign", "two")))]).is_err());
        assert!(second.fetch_doc("foreign").unwrap().is_none());
        let _raw = first.tantivy();
        assert!(first.unowned_projection_stamp().is_none());
        assert!(second.unowned_projection_stamp().is_some());
    }

    #[test]
    fn simultaneous_unknown_writers_are_unavailable_until_last_real_owner_drops() {
        let clock = super::Observation::default();
        let before = clock.stamp().unwrap();
        let a = clock.begin();
        let b = clock.begin();
        assert!(clock.stamp().is_none());
        drop(a);
        assert!(clock.stamp().is_none());
        drop(b);
        assert_ne!(clock.stamp(), Some(before));
    }
}
