//! Shared account notification configuration and administrator policy.
use chrono::Timelike;
use mw_store::settings_policy::{SettingScope, SettingSubject, SettingValueRow};
use mw_store::{Store, StoreError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotificationRuleDto {
    pub id: String,
    pub label: String,
    /// `match` is a Rust keyword; the wire/JSON field is `match`.
    pub r#match: String,
    /// "notify" | "mute" (opaque here).
    pub action: String,
}

/// A quiet-hours window (local 24h HH:MM).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuietHoursDto {
    pub enabled: bool,
    pub start: String,
    pub end: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

impl Default for QuietHoursDto {
    fn default() -> Self {
        QuietHoursDto {
            enabled: false,
            start: "22:00".to_string(),
            end: "07:00".to_string(),
            timezone: None,
        }
    }
}

/// The account's notification configuration (`GET`/`PUT /api/account/notifications`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotificationConfigDto {
    pub enabled: bool,
    #[serde(default)]
    pub rules: Vec<NotificationRuleDto>,
    #[serde(default)]
    pub quiet_hours: QuietHoursDto,
}

pub(crate) fn validate(config: &NotificationConfigDto) -> Result<(), &'static str> {
    if config.rules.len() > 64 {
        return Err("at most 64 notification rules are allowed");
    }
    let mut ids = std::collections::BTreeSet::new();
    for rule in &config.rules {
        if rule.id.is_empty()
            || rule.id.len() > 64
            || !ids.insert(&rule.id)
            || rule.label.len() > 128
            || rule.r#match.trim().is_empty()
            || rule.r#match.len() > 256
            || !matches!(rule.action.as_str(), "notify" | "mute")
            || [&rule.id, &rule.label, &rule.r#match]
                .iter()
                .any(|text| text.chars().any(char::is_control))
        {
            return Err("invalid, duplicate or oversized notification rule");
        }
    }
    if minutes(&config.quiet_hours.start).is_none() || minutes(&config.quiet_hours.end).is_none() {
        return Err("quiet hours require HH:MM times");
    }
    if config
        .quiet_hours
        .timezone
        .as_deref()
        .unwrap_or("UTC")
        .parse::<chrono_tz::Tz>()
        .is_err()
    {
        return Err("quiet hours require an IANA time zone");
    }
    Ok(())
}

fn minutes(value: &str) -> Option<u32> {
    if value.len() != 5 || !value.is_ascii() || value.as_bytes()[2] != b':' {
        return None;
    }
    let hour = value[..2].parse::<u32>().ok()?;
    let minute = value[3..].parse::<u32>().ok()?;
    (hour < 24 && minute < 60).then_some(hour * 60 + minute)
}

pub(crate) fn quiet_at(quiet: &QuietHoursDto, now: chrono::DateTime<chrono::Utc>) -> bool {
    if !quiet.enabled {
        return false;
    }
    let (Some(start), Some(end)) = (minutes(&quiet.start), minutes(&quiet.end)) else {
        return true;
    };
    let Ok(zone) = quiet
        .timezone
        .as_deref()
        .unwrap_or("UTC")
        .parse::<chrono_tz::Tz>()
    else {
        return true;
    };
    let local = now.with_timezone(&zone);
    let minute = local.hour() * 60 + local.minute();
    if start < end {
        minute >= start && minute < end
    } else if start > end {
        minute >= start || minute < end
    } else {
        false
    }
}

pub(crate) async fn for_subject(
    store: &Store,
    subject: &SettingSubject<'_>,
    owner: &str,
) -> Result<NotificationConfigDto, StoreError> {
    let row = store.get_personal_notification_rules(owner).await?;
    let mut config = match &row {
        Some(row) => NotificationConfigDto {
            enabled: row.enabled,
            rules: serde_json::from_str(&row.rule_json)
                .map_err(|_| StoreError::Corrupt("notification rules".into()))?,
            quiet_hours: serde_json::from_str(&row.quiet_hours_json)
                .map_err(|_| StoreError::Corrupt("notification quiet hours".into()))?,
        },
        None => NotificationConfigDto::default(),
    };
    validate(&config).map_err(|_| StoreError::Corrupt("notification configuration".into()))?;
    let extra = row
        .map(|row| SettingValueRow {
            scope: SettingScope::User,
            scope_id: subject.account_id.into(),
            key: "notifications.enabled".into(),
            value: row.enabled.to_string(),
            locked: false,
            updated_by: "notification-rules".into(),
            updated_at: 0,
        })
        .into_iter()
        .collect::<Vec<_>>();
    config.enabled = crate::settings_registry::effective_for_with(
        store,
        "notifications.enabled",
        subject,
        &extra,
    )
    .await?
    .is_some_and(|value| value.value.as_bool() == Some(true));
    Ok(config)
}

pub(crate) async fn account_subject(
    store: &Store,
    account: &str,
) -> Result<Option<crate::settings_routes::UserSubject>, StoreError> {
    let username = match store.get_account(account).await {
        Ok(row) => row.username,
        Err(StoreError::NotFound) => return Ok(None),
        Err(error) => return Err(error),
    };
    let subject = SettingSubject {
        account_id: account,
        address: &username,
        domain: username
            .rsplit_once('@')
            .map(|(_, domain)| domain)
            .unwrap_or(""),
    };
    let canonical = store.canonical_setting_subject(&subject).await?;
    Ok(Some(crate::settings_routes::UserSubject {
        personal_owner: None,
        record_owner: store.native_personal_subject(account).await?,
        account_id: canonical.account_id,
        address: canonical.address,
        domain: canonical.domain,
    }))
}

pub(crate) async fn allowed(
    store: &Store,
    subject: &SettingSubject<'_>,
    key: &str,
) -> Result<bool, StoreError> {
    Ok(crate::settings_registry::effective_for(store, key, subject)
        .await?
        .is_some_and(|value| value.value.as_bool() == Some(true)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quiet_hours_use_saved_zone_and_reject_malformed_unicode() {
        let quiet = QuietHoursDto {
            enabled: true,
            start: "22:00".into(),
            end: "07:00".into(),
            timezone: Some("Europe/London".into()),
        };
        assert!(quiet_at(&quiet, "2026-07-01T21:00:00Z".parse().unwrap()));
        assert!(!quiet_at(&quiet, "2026-07-02T06:00:00Z".parse().unwrap()));
        assert!(!quiet_at(&quiet, "2026-01-01T21:00:00Z".parse().unwrap()));
        assert_eq!(minutes("é:00"), None);
        let invalid = NotificationConfigDto {
            quiet_hours: QuietHoursDto {
                start: "é:00".into(),
                ..quiet
            },
            ..Default::default()
        };
        assert!(validate(&invalid).is_err());
    }
    #[tokio::test]
    async fn effective_defaults_and_admin_ceiling_are_real() {
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let subject = SettingSubject {
            account_id: "a",
            address: "a@example.org",
            domain: "example.org",
        };
        assert!(!for_subject(&store, &subject,"a").await.unwrap().enabled);
        assert!(
            allowed(&store, &subject, "notifications.pushAllowed")
                .await
                .unwrap()
        );
        assert!(
            !allowed(&store, &subject, "offline.persistentStorageAllowed")
                .await
                .unwrap()
        );
        store
            .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                key: "notifications.enabled".into(),
                global_value: Some("true".into()),
                allow_user_override: true,
                global_locked: false,
                updated_by: "test".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        assert!(for_subject(&store, &subject,"a").await.unwrap().enabled);
        store
            .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                key: "notifications.pushAllowed".into(),
                global_value: Some("false".into()),
                allow_user_override: false,
                global_locked: true,
                updated_by: "test".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        assert!(
            !allowed(&store, &subject, "notifications.pushAllowed")
                .await
                .unwrap()
        );
    }
}
