//! Administrator limits on the undo-send hold and on scheduled sends.
//!
//! The web client picks the hold (`mail.send.recallDelaySeconds`, the user's
//! own choice) and a `sendAt`; these keys are the ceiling the engine applies at
//! `EmailSubmission/set` create time, so a modified client cannot exceed them:
//!
//! * `mail.send.allowRecall` (default `true`): `false` turns the hold off, a
//!   submission is sent at once (a future `sendAt` still schedules).
//! * `mail.send.recallDelayMaxSeconds` (default 1800, max 1800): the hold is
//!   clamped to it, never refused.
//! * `mail.send.maxDelayedSeconds` (default 90 days, max 1 year): how far ahead
//!   `sendAt` may be; further is refused. 0 turns scheduling off.
//! * `mail.send.maxQueued` (default 200, max 5000): pending submissions per
//!   account; a create that would be queued past it is refused, never dropped.
//!
//! Only administrator layers are read (a user cannot store these keys), so the
//! values resolve without a user row.

use chrono::{DateTime, Duration, Utc};
use mw_store::settings_policy::{
    ResolveInput, SettingScope, SettingSubject, SettingValueRow, resolve,
};
use serde_json::{Value, json};

use crate::engine::Engine;

/// Longest hold the feature supports (30 minutes).
pub const MAX_HOLD_SECONDS: u32 = 1800;
/// Default scheduling horizon (90 days).
pub const DEFAULT_HORIZON_SECONDS: u64 = 90 * 24 * 3600;
/// Largest horizon an administrator may set (1 year).
pub const MAX_HORIZON_SECONDS: u64 = 365 * 24 * 3600;
/// Default pending submissions per account.
pub const DEFAULT_MAX_QUEUED: usize = 200;

/// The effective limits for one account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendLimits {
    pub allow_recall: bool,
    pub recall_max_seconds: u32,
    pub horizon_seconds: u64,
    pub max_queued: usize,
}

impl Default for SendLimits {
    fn default() -> Self {
        Self {
            allow_recall: true,
            recall_max_seconds: MAX_HOLD_SECONDS,
            horizon_seconds: DEFAULT_HORIZON_SECONDS,
            max_queued: DEFAULT_MAX_QUEUED,
        }
    }
}

impl SendLimits {
    /// The hold actually applied for a requested one.
    pub fn clamp_hold(&self, requested: u32) -> u32 {
        if !self.allow_recall {
            0
        } else {
            requested.min(self.recall_max_seconds).min(MAX_HOLD_SECONDS)
        }
    }

    /// Refuse a `sendAt` beyond the horizon. A time in the past is the
    /// caller's own check.
    pub fn check_send_at(&self, at: DateTime<Utc>, now: DateTime<Utc>) -> Result<(), String> {
        if at <= now {
            return Ok(());
        }
        let horizon = self.horizon_seconds.min(MAX_HORIZON_SECONDS);
        if horizon == 0 {
            return Err("scheduled sending is turned off by the administrator".into());
        }
        let limit = now + Duration::seconds(i64::try_from(horizon).unwrap_or(i64::MAX / 4));
        if at > limit {
            let days = horizon / 86_400;
            return Err(format!(
                "cannot be more than {} {} ahead",
                if days > 0 { days } else { horizon / 3600 },
                if days > 0 { "days" } else { "hours" },
            ));
        }
        Ok(())
    }

    /// Refuse a create that would be queued while `queued` are already pending.
    pub fn check_queue(&self, queued: usize) -> Result<(), String> {
        if queued >= self.max_queued {
            return Err(format!(
                "too many queued or scheduled messages ({} allowed)",
                self.max_queued
            ));
        }
        Ok(())
    }
}

fn parse<T: std::str::FromStr>(text: &str) -> Option<T> {
    text.trim().parse().ok()
}

impl Engine {
    /// The administrator value of `key` for this account, or `floor` when none.
    async fn admin_setting_for(
        &self,
        account_id: &str,
        key: &str,
        floor: &str,
    ) -> crate::backend::Result<String> {
        let address = self
            .store()
            .get_account(account_id)
            .await?
            .username
            .to_ascii_lowercase();
        let domain = address
            .rsplit_once('@')
            .map(|(_, d)| d.to_string())
            .unwrap_or_default();
        let subject = SettingSubject {
            personal_owner: None,
            domain: &domain,
            account_id,
            address: &address,
        };
        let policy = self.store().get_setting_policy(key).await?;
        let rows = self.store().setting_values_for_subject(&subject).await?;
        let mine: Vec<&SettingValueRow> = rows
            .iter()
            .filter(|r| r.key == key && r.scope != SettingScope::User)
            .collect();
        Ok(resolve(&ResolveInput {
            floor,
            floor_is_env: false,
            pin: None,
            policy: policy.as_ref(),
            values: &mine,
        })
        .value)
    }

    /// The account's effective limits. A key that cannot be read falls back to
    /// its default: these are ceilings, and the hold is bounded in any case.
    pub(crate) async fn send_limits(&self, account_id: &str) -> SendLimits {
        let d = SendLimits::default();
        SendLimits {
            allow_recall: self
                .admin_setting_for(account_id, "mail.send.allowRecall", "true")
                .await
                .ok()
                .and_then(|v| parse::<bool>(&v))
                .unwrap_or(d.allow_recall),
            recall_max_seconds: self
                .admin_setting_for(account_id, "mail.send.recallDelayMaxSeconds", "1800")
                .await
                .ok()
                .and_then(|v| parse::<u32>(&v))
                .unwrap_or(d.recall_max_seconds),
            horizon_seconds: self
                .admin_setting_for(account_id, "mail.send.maxDelayedSeconds", "7776000")
                .await
                .ok()
                .and_then(|v| parse::<u64>(&v))
                .unwrap_or(d.horizon_seconds),
            max_queued: self
                .admin_setting_for(account_id, "mail.send.maxQueued", "200")
                .await
                .ok()
                .and_then(|v| parse::<usize>(&v))
                .unwrap_or(d.max_queued),
        }
    }

    /// Apply the limits to a create about to be enqueued. Returns the hold to
    /// use, or the JMAP `notCreated` error.
    pub(crate) async fn apply_send_limits(
        &self,
        account_id: &str,
        hold_seconds: u32,
        send_at: Option<&str>,
        manual_hold: bool,
    ) -> std::result::Result<u32, Value> {
        let limits = self.send_limits(account_id).await;
        let now = Utc::now();
        let at = send_at
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc));
        if let Some(at) = at
            && let Err(why) = limits.check_send_at(at, now)
        {
            return Err(json!({
                "type": "invalidProperties",
                "properties": ["sendAt"],
                "description": format!("sendAt: {why}"),
            }));
        }
        let hold = limits.clamp_hold(hold_seconds);
        let queued = hold > 0 || manual_hold || at.is_some_and(|a| a > now);
        if queued {
            let pending = self
                .store()
                .list_submissions(account_id)
                .await
                .map(|rows| rows.iter().filter(|r| r.undo_status == "pending").count())
                .map_err(|_| {
                    json!({"type":"serverFail","description":"queue size could not be checked"})
                })?;
            if let Err(why) = limits.check_queue(pending) {
                return Err(json!({"type":"overQuota","description":why}));
            }
        }
        Ok(hold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn hold_is_clamped_and_switched_off() {
        let mut l = SendLimits::default();
        assert_eq!(l.clamp_hold(10), 10);
        assert_eq!(l.clamp_hold(99_999), 1800);
        l.recall_max_seconds = 60;
        assert_eq!(l.clamp_hold(300), 60);
        l.allow_recall = false;
        assert_eq!(l.clamp_hold(30), 0);
    }

    #[test]
    fn horizon_bounds_the_send_time() {
        let now = at("2026-01-01T00:00:00Z");
        let l = SendLimits::default();
        assert!(l.check_send_at(at("2026-03-01T00:00:00Z"), now).is_ok());
        assert!(l.check_send_at(at("2026-06-01T00:00:00Z"), now).is_err());
        let off = SendLimits {
            horizon_seconds: 0,
            ..l
        };
        assert!(off.check_send_at(at("2026-01-02T00:00:00Z"), now).is_err());
        assert!(off.check_send_at(now, now).is_ok());
    }

    #[test]
    fn queue_cap_refuses_at_the_limit() {
        let l = SendLimits {
            max_queued: 2,
            ..SendLimits::default()
        };
        assert!(l.check_queue(1).is_ok());
        assert!(l.check_queue(2).is_err());
    }
}
