//! Request-scoped sharing of JMAP response state tokens (t34-e12, D3).
//!
//! Every read method in a JMAP request used to read its own state counter
//! (`type_state`, one statement per method) and the request then read the joint
//! `sessionState` frame once more. Within one request whose methods only read,
//! those values are the same snapshot of the same three mail counters, so the
//! request now reads the joint frame **once**, at the first point a state token
//! is needed, and every later read method and the closing `sessionState` reuse
//! it.
//!
//! Boundaries, each enforced here rather than by callers:
//! - The frame lives in a task-local installed by
//!   [`crate::jmap`]'s request loop. It ends with that request; nothing is kept
//!   on the engine, so it cannot outlive the request or reach another one.
//! - It is bound to the request's account. A lookup for any other account reads
//!   fresh (see [`RequestFrame::mail_counters`]).
//! - Only the read methods in [`SHARES_STATE`] fill or consume it. Every other
//!   method (any write, Sieve, MDN, PIM, security, unknown) clears it **before it
//!   runs** and runs with it closed, so it neither reuses nor fills it. A
//!   mutation followed by a read in the same request is therefore always
//!   followed by a fresh frame read, and the closing `sessionState` is fresh
//!   after any such method. `Email/set` checks `ifInState` against a fresh
//!   `type_state` in any case.
//! - A failed joint read is not cached; the caller falls back to the original
//!   per-type reader and its error handling.
//!
//! The token a read method returns is therefore read no later than the data it
//! accompanies, never after it: a concurrent writer can make the token older
//! than the data, which only makes a client re-fetch changes it already has. It
//! can no longer make the token newer than the data (which would hide a change).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Read methods that may fill and reuse the request's state frame. Anything
/// else invalidates it before it runs. The `*/changes` methods served by the
/// shared mail handler and `Email/queryChanges` read their current counter
/// from the frame (t50hp-e7): the client's refresh pair, `Email/queryChanges`
/// and `Email/changes`, then reads the three mail counters once, and the
/// closing `sessionState` reuses that read.
pub(crate) const SHARES_STATE: [&str; 7] = [
    "Mailbox/get",
    "Email/get",
    "Email/query",
    "Email/changes",
    "Email/queryChanges",
    "Mailbox/changes",
    "EmailSubmission/changes",
];

/// The joint counters read once for this request, or nothing yet.
pub(crate) struct RequestFrame {
    account: String,
    counters: Mutex<Option<Arc<mw_store::SessionStateCounters>>>,
    /// Whether the code running now may fill or reuse the frame: true while a
    /// [`SHARES_STATE`] method runs and for the closing `sessionState`, false
    /// while any other method runs (it then reads fresh, as before).
    open: AtomicBool,
}

tokio::task_local! {
    static FRAME: Arc<RequestFrame>;
}

impl RequestFrame {
    pub(crate) fn new(account: &str) -> Arc<Self> {
        Arc::new(Self {
            account: account.to_owned(),
            counters: Mutex::new(None),
            open: AtomicBool::new(false),
        })
    }

    /// Run `fut` with `frame` as the current request's frame. A nested request
    /// (an engine method that issues its own JMAP request) installs its own.
    pub(crate) async fn scope<F: std::future::Future>(frame: Arc<Self>, fut: F) -> F::Output {
        FRAME.scope(frame, fut).await
    }

    /// Called before each method of the request runs. A method outside
    /// [`SHARES_STATE`] drops whatever was read and runs with the frame closed,
    /// so it neither reuses nor fills it.
    pub(crate) fn begin_method(&self, name: &str) {
        if SHARES_STATE.contains(&name) {
            self.open.store(true, Ordering::Release);
        } else {
            self.open.store(false, Ordering::Release);
            *self.counters.lock().expect("request frame lock") = None;
        }
    }

    /// Called after the last method, before the closing `sessionState`.
    pub(crate) fn end_methods(&self) {
        self.open.store(true, Ordering::Release);
    }

    fn get(&self) -> Option<Arc<mw_store::SessionStateCounters>> {
        self.counters.lock().expect("request frame lock").clone()
    }

    fn put(&self, counters: Arc<mw_store::SessionStateCounters>) {
        *self.counters.lock().expect("request frame lock") = Some(counters);
    }

    /// The current request's frame for `account`, if this task runs inside a
    /// request for exactly that account and the frame is open.
    pub(crate) fn current(account: &str) -> Option<Arc<Self>> {
        FRAME
            .try_with(|frame| {
                (frame.account == account && frame.open.load(Ordering::Acquire))
                    .then(|| Arc::clone(frame))
            })
            .ok()
            .flatten()
    }

    /// The shared counters for this request: the frame's, or one fresh joint
    /// read stored into it. `None` when the joint read failed; the caller then
    /// uses its original reader.
    pub(crate) async fn mail_counters(
        &self,
        store: &mw_store::Store,
    ) -> Option<Arc<mw_store::SessionStateCounters>> {
        if let Some(found) = self.get() {
            return Some(found);
        }
        let read = Arc::new(store.try_session_state_counters(&self.account).await.ok()?);
        self.put(Arc::clone(&read));
        Some(read)
    }
}
