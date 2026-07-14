//! Send-gating (SPEC §20.3, §7.1 over-privileged automation) — the
//! safety-critical decision for `mail.send`.
//!
//! By the time this runs, the caller's scope has already been authorized to hold
//! `send` + the `mail.send` tool grant. This gate decides *how* the send happens:
//!
//! | `unattended_send` | admin countersign | decision            |
//! |-------------------|-------------------|---------------------|
//! | false             | (ignored)         | [`SendDecision::Queue`]   (→ Outbox) |
//! | true              | false             | [`SendDecision::Deny`]    (→ 403)    |
//! | true              | true              | [`SendDecision::SendNow`] (transmit) |
//!
//! The default (`unattended_send=false`) is always the human-in-the-loop Outbox.

use mw_oauth::Scope;

/// What to do with an authorized `mail.send`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendDecision {
    /// Place in the V2 Outbox for in-app human confirmation (the default).
    Queue,
    /// Transmit immediately — only for `unattended_send` + admin countersign.
    SendNow,
    /// Refuse: `unattended_send` requested without the required admin
    /// countersignature (→ 403).
    Deny,
}

/// Decide the fate of an authorized `mail.send` from the *granted* scope and the
/// key's admin-countersign flag. Pure + total — the single source of truth for the
/// three send paths.
pub fn gate_send(granted: &Scope, admin_countersigned: bool) -> SendDecision {
    if !granted.unattended_send {
        // Default & only safe automation path: human confirms in the Outbox.
        SendDecision::Queue
    } else if admin_countersigned {
        // Explicitly opted in AND countersigned by an admin: may transmit.
        SendDecision::SendNow
    } else {
        // Unattended requested but not countersigned — refuse.
        SendDecision::Deny
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_oauth::Scope;

    fn scope(unattended: bool) -> Scope {
        let mut s = Scope::read_only("acct");
        s.send = true;
        s.unattended_send = unattended;
        s
    }

    #[test]
    fn default_key_queues_to_outbox() {
        assert_eq!(gate_send(&scope(false), false), SendDecision::Queue);
        // Countersign is irrelevant without unattended_send.
        assert_eq!(gate_send(&scope(false), true), SendDecision::Queue);
    }

    #[test]
    fn unattended_without_countersign_denies() {
        assert_eq!(gate_send(&scope(true), false), SendDecision::Deny);
    }

    #[test]
    fn unattended_with_countersign_sends() {
        assert_eq!(gate_send(&scope(true), true), SendDecision::SendNow);
    }
}
