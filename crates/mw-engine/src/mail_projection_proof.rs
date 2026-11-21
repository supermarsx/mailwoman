//! Account-local observation clocks for prepared mail DATA, never authority.
//!
//! The caller must additionally check Publisher poison/repair/erasure state and
//! the fresh Store witness. These clocks do not assert a SQL/Tantivy lease.
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

const MAX_ACCOUNTS: usize = 10_000;

#[derive(Default)]
struct Version {
    revision: u64,
    focus: u64,
    outstanding: usize,
    exhausted: bool,
    messages: HashMap<String, Weak<Mutex<MessageVersion>>>,
}
#[derive(Default)]
struct MessageVersion { revision:u64, exhausted:bool }

#[derive(Default)]
struct Account {
    version: Mutex<Version>,
}

#[derive(Default)]
struct Inner {
    lifecycle: u64,
    exhausted: bool,
    accounts: HashMap<String, Weak<Account>>,
}

#[derive(Clone, Default)]
pub(crate) struct ProjectionClock(Arc<Mutex<Inner>>);

/// Owned by the actual admission/job, not by its response waiter.
pub(crate) struct ProjectionWrite {
    account: Arc<Account>,
    touched:Vec<Arc<Mutex<MessageVersion>>>,
}

/// Keeps the account identity alive; an evicted/recreated weak entry cannot ABA.
#[derive(Clone)]
pub(crate) struct ProjectionProof {
    clock: ProjectionClock,
    account: Arc<Account>,
    lifecycle: u64,
    revision: u64,
    focus: u64,
    selected: Option<Vec<(Arc<Mutex<MessageVersion>>,u64)>>,
}

impl ProjectionClock {
    fn account(&self, account: &str) -> Option<(Arc<Account>, u64)> {
        if account.is_empty() || account.len() > 1024 { return None; }
        let mut inner = self.0.lock().ok()?;
        if inner.exhausted { return None; }
        if let Some(value) = inner.accounts.get(account).and_then(Weak::upgrade) {
            return Some((value, inner.lifecycle));
        }
        inner.accounts.retain(|_, entry| entry.strong_count() != 0);
        if inner.accounts.len() >= MAX_ACCOUNTS { return None; }
        let value = Arc::new(Account::default());
        inner.accounts.insert(account.to_owned(), Arc::downgrade(&value));
        Some((value, inner.lifecycle))
    }

    /// Called before a reserved ticket can reach upstream or source mutation.
    pub(crate) fn begin(&self, account: &str) -> Option<ProjectionWrite> {
        let (account, _) = self.account(account)?;
        {
            let mut state = account.version.lock().ok()?;
            if state.exhausted { return None; }
            state.outstanding = state.outstanding.checked_add(1)?;
        }
        Some(ProjectionWrite { account,touched:Vec::new() })
    }

    /// Only after the owner's independent clean-state checks. A write racing
    /// this capture either makes it unavailable or invalidates its final check.
    #[cfg(test)]
    pub(crate) fn observe(&self, account: &str) -> Option<ProjectionProof> {
        self.observe_inner(account,None)
    }
    pub(crate) fn observe_selected(&self, account:&str,ids:&[String])->Option<ProjectionProof> {
        if ids.len()>50 || ids.iter().any(|id|id.is_empty() || id.len()>1024){return None}
        self.observe_inner(account,Some(ids))
    }
    fn observe_inner(&self,account:&str,ids:Option<&[String]>)->Option<ProjectionProof> {
        let (account, lifecycle) = self.account(account)?;
        let (revision,focus,selected) = {
            let mut state = account.version.lock().ok()?;
            if state.exhausted || state.outstanding != 0 { return None; }
            let selected=if let Some(ids)=ids {
                if ids.iter().any(|id|state.messages.get(id).and_then(Weak::upgrade).is_none()) {
                    state.messages.retain(|_,entry|entry.strong_count()!=0);
                }
                let mut selected=Vec::with_capacity(ids.len());
                for id in ids {
                    let message=match state.messages.get(id).and_then(Weak::upgrade) {
                        Some(message)=>message,
                        None=>{
                            if state.messages.len()>=16384{return None}
                            let message=Arc::new(Mutex::new(MessageVersion::default()));
                            state.messages.insert(id.clone(),Arc::downgrade(&message));message
                        }
                    };
                    let version={let version=message.lock().ok()?;if version.exhausted{return None}version.revision};
                    selected.push((message,version));
                }
                Some(selected)
            }else{None};
            (state.revision,state.focus,selected)
        };
        let proof = ProjectionProof { clock: self.clone(), account, lifecycle, revision,focus,selected };
        proof.is_current().then_some(proof)
    }
    /// Conservative account Focus invalidation when index To/Cc/mailbox or
    /// liveness may change. SQL contacts/Sent epochs are separate dependencies.
    pub(crate) fn focus_changed(&self,account:&str) {
        let Some((account,_))=self.account(account) else{self.invalidate_all();return};
        if let Ok(mut state)=account.version.lock(){match state.focus.checked_add(1){Some(next)=>state.focus=next,None=>state.exhausted=true}};
    }

    /// Rare lifecycle transitions only: rebuild/recovery/unknown index source.
    /// Ordinary other-account publications must NEVER call this operation.
    pub(crate) fn invalidate_all(&self) {
        if let Ok(mut inner) = self.0.lock() {
            match inner.lifecycle.checked_add(1) {
                Some(next) => inner.lifecycle = next,
                None => inner.exhausted = true,
            }
        }
    }
}

impl ProjectionProof {
    pub(crate) fn matches(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.clock.0, &other.clock.0)
            && Arc::ptr_eq(&self.account, &other.account)
            && self.lifecycle == other.lifecycle
            && self.focus == other.focus
            && (match (&self.selected,&other.selected) {
                (None,None)=>self.revision==other.revision,
                (Some(a),Some(b))=>a.len()==b.len() && a.iter().zip(b).all(|((a,av),(b,bv))|Arc::ptr_eq(a,b) && av==bv),
                _=>false,
            })
            && self.is_current()
            && other.is_current()
    }
    /// Synchronous final DATA check. Caller's Store/auth/posture proof remains
    /// mandatory and no async operation is authorized by this return value.
    pub(crate) fn is_current(&self) -> bool {
        let Ok(inner) = self.clock.0.lock() else { return false; };
        if inner.exhausted || inner.lifecycle != self.lifecycle { return false; }
        let Ok(state) = self.account.version.lock() else { return false; };
        if state.exhausted || state.outstanding!=0 || state.focus!=self.focus{return false}
        match &self.selected {
            None=>state.revision==self.revision,
            Some(selected)=>selected.iter().all(|(message,revision)|message.lock().is_ok_and(|current|!current.exhausted && current.revision==*revision)),
        }
    }
}

impl ProjectionWrite {
    /// Called on track BEFORE mutation and again on actual owned handoff.
    /// No new message state is created for unobserved IDs; a pending ticket
    /// already prevents a new reader from certifying those IDs mid-mutation.
    pub(crate) fn touched(&mut self,ids:&[String]) {
        let Ok(state)=self.account.version.lock() else{return};
        self.touched=ids.iter().collect::<std::collections::BTreeSet<_>>().into_iter()
            .filter_map(|id|state.messages.get(id).and_then(Weak::upgrade)).collect();
    }
}

impl Drop for ProjectionWrite {
    fn drop(&mut self) {
        if let Ok(mut state) = self.account.version.lock() {
            for message in &self.touched {
                if let Ok(mut message)=message.lock(){match message.revision.checked_add(1){Some(next)=>message.revision=next,None=>message.exhausted=true}}
            }
            // Even disarmed/no-op/canceled tickets conservatively invalidate
            // earlier observations. This is NOT a successful-commit counter.
            match state.revision.checked_add(1) {
                Some(next) => state.revision = next,
                None => state.exhausted = true,
            }
            match state.outstanding.checked_sub(1) {
                Some(next) => state.outstanding = next,
                None => state.exhausted = true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn another_accounts_real_boundary_does_not_invalidate_this_account() {
        let clock = ProjectionClock::default();
        let a = clock.observe("a").unwrap();
        let b = clock.observe("b").unwrap();
        let pending = clock.begin("b").unwrap();
        assert!(a.is_current());
        assert!(!b.is_current());
        assert!(clock.observe("b").is_none());
        drop(pending);
        assert!(a.is_current());
        assert!(!b.is_current());
        assert!(clock.observe("b").unwrap().is_current());
    }

    #[test]
    fn accepted_owner_retains_pending_through_response_waiter_drop() {
        let clock = ProjectionClock::default();
        let before = clock.observe("a").unwrap();
        let accepted_job = Arc::new(clock.begin("a").unwrap());
        let waiter = Arc::clone(&accepted_job);
        drop(waiter);
        assert!(!before.is_current());
        assert!(clock.observe("a").is_none());
        drop(accepted_job);
        assert!(!before.is_current());
        assert!(clock.observe("a").unwrap().is_current());
    }

    #[test]
    fn lifecycle_and_overlapping_tickets_never_revalidate_old_observations() {
        let clock = ProjectionClock::default();
        let old = clock.observe("a").unwrap();
        let first = clock.begin("a").unwrap();
        let second = clock.begin("a").unwrap();
        drop(first);
        assert!(clock.observe("a").is_none());
        drop(second);
        let next = clock.observe("a").unwrap();
        clock.invalidate_all();
        assert!(!old.is_current());
        assert!(!next.is_current());
        assert!(clock.observe("a").unwrap().is_current());
        assert!(clock.observe("").is_none());
    }

    #[test]
    fn unchanged_selected_ids_recertify_after_unselected_metadata_but_not_focus_or_selected_change(){
        let clock=ProjectionClock::default();let selected=vec!["shown".to_owned()];
        let before=clock.observe_selected("a",&selected).unwrap();
        let mut other=clock.begin("a").unwrap();other.touched(&["other".into()]);
        assert!(!before.is_current());drop(other);
        assert!(before.matches(&clock.observe_selected("a",&selected).unwrap()));
        let mut same=clock.begin("a").unwrap();same.touched(&selected);drop(same);
        assert!(!before.is_current());
        let ready=clock.observe_selected("a",&selected).unwrap();clock.focus_changed("a");
        assert!(!ready.is_current());
    }
}
