//! Leave-on-server retention policy (plan §0, SPEC §6.3).
//!
//! POP3 traditionally drains the maildrop; Mailwoman defaults to leaving mail
//! on the server. The policy is a pure decision function so the DELE-issuance
//! branches are unit-testable without a socket — the transport layer
//! ([`crate::conn`]) supplies the [`DeleteContext`] and honours the verdict.

/// Retention policy for messages after the client has seen them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LeavePolicy {
    /// Never issue `DELE`; the maildrop is read-only from our side.
    #[default]
    Keep,
    /// `DELE` a message once it is older than `N` days (by its `Date:` header),
    /// but only after it has already been ingested at least once.
    DeleteAfterDays(u32),
    /// `DELE` a message in the same session it is retrieved (`RETR`).
    DeleteOnRetrieval,
}

/// Facts the transport knows about one message when deciding whether to DELE.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DeleteContext {
    /// This message was just `RETR`-ed in the current session.
    pub just_retrieved: bool,
    /// This message was already present in the persisted UIDL cursor
    /// (i.e. we have ingested it on a prior sync).
    pub previously_seen: bool,
    /// Age in whole days derived from the message `Date:` header, if known.
    pub age_days: Option<i64>,
}

impl LeavePolicy {
    /// Whether a `DELE` should be issued for a message in the given context.
    ///
    /// - [`Keep`](LeavePolicy::Keep): always `false`.
    /// - [`DeleteOnRetrieval`](LeavePolicy::DeleteOnRetrieval): `true` exactly
    ///   when the message was just retrieved.
    /// - [`DeleteAfterDays(n)`](LeavePolicy::DeleteAfterDays): `true` when the
    ///   message has been seen before and its known age is `>= n` days. An
    ///   unknown age is treated as "not old enough" (never delete blind).
    pub fn should_delete(&self, ctx: DeleteContext) -> bool {
        match *self {
            LeavePolicy::Keep => false,
            LeavePolicy::DeleteOnRetrieval => ctx.just_retrieved,
            LeavePolicy::DeleteAfterDays(n) => {
                ctx.previously_seen && ctx.age_days.is_some_and(|d| d >= i64::from(n))
            }
        }
    }

    /// Whether the policy needs a `Date:` header (via `TOP`) during sync to
    /// evaluate age-based reaping. Lets the sync path skip `TOP` when useless.
    pub fn needs_age(&self) -> bool {
        matches!(self, LeavePolicy::DeleteAfterDays(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(retrieved: bool, seen: bool, age: Option<i64>) -> DeleteContext {
        DeleteContext {
            just_retrieved: retrieved,
            previously_seen: seen,
            age_days: age,
        }
    }

    #[test]
    fn keep_never_deletes() {
        let p = LeavePolicy::Keep;
        assert!(!p.should_delete(ctx(true, true, Some(9999))));
        assert!(!p.needs_age());
    }

    #[test]
    fn delete_on_retrieval_only_on_retrieval() {
        let p = LeavePolicy::DeleteOnRetrieval;
        assert!(p.should_delete(ctx(true, false, None)));
        assert!(!p.should_delete(ctx(false, true, Some(100))));
        assert!(!p.needs_age());
    }

    #[test]
    fn delete_after_days_thresholds() {
        let p = LeavePolicy::DeleteAfterDays(7);
        assert!(p.needs_age());
        // Old enough and previously ingested -> delete.
        assert!(p.should_delete(ctx(false, true, Some(7))));
        assert!(p.should_delete(ctx(false, true, Some(30))));
        // Too new -> keep.
        assert!(!p.should_delete(ctx(false, true, Some(6))));
        // Age unknown -> never delete blind.
        assert!(!p.should_delete(ctx(false, true, None)));
        // Never ingested before -> don't delete on first contact.
        assert!(!p.should_delete(ctx(false, false, Some(90))));
    }
}
