//! Scheduler-only global-window evaluation; the HTTP maintenance gate can reuse
//! this contract. No window is inferred from invalid or unsupported JSON.
use chrono::{DateTime, Datelike, NaiveTime, Timelike, Utc};
use mw_store::jobs::SchedulerWindow;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Recurrence {
    weekly: Option<Weekly>,
    monthly: Option<Monthly>,
    every_days: Option<u32>,
    duration_secs: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Weekly { days: Vec<String>, at: Option<String> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Monthly { day: u32, at: Option<String> }

pub(super) fn active(row: &SchedulerWindow, now: i64) -> bool {
    let Some(anchor) = DateTime::<Utc>::from_timestamp(row.starts_at, 0) else { return false };
    let Some(now_time) = DateTime::<Utc>::from_timestamp(now, 0) else { return false };
    let Some(tz) = super::parse_tz(&row.tz) else { return false };
    if now < row.starts_at { return false; }
    let Some(raw) = &row.recurrence_json else {
        return row.ends_at.is_none_or(|end| end > row.starts_at && now < end)
            && row.occurrence_ended_at.is_none_or(|end| end < row.starts_at || now < end);
    };
    let Ok(rule) = serde_json::from_str::<Recurrence>(raw) else { return false };
    if rule.duration_secs == 0 || rule.duration_secs > 31 * 86_400
        || usize::from(rule.weekly.is_some()) + usize::from(rule.monthly.is_some())
            + usize::from(rule.every_days.is_some()) != 1 { return false; }
    let anchor_local = anchor.with_timezone(&tz);
    let anchor_date = anchor_local.date_naive();
    let default_at = NaiveTime::from_hms_opt(anchor_local.hour(), anchor_local.minute(), 0).unwrap();
    let mut at = default_at;
    let mut weekdays = Vec::new();
    if let Some(weekly) = &rule.weekly {
        if weekly.days.is_empty() || weekly.days.len() > 7 { return false; }
        for day in &weekly.days {
            let Some(day) = super::parse_weekday(day) else { return false };
            if weekdays.contains(&day) { return false; }
            weekdays.push(day);
        }
        if let Some(text) = &weekly.at {
            let Some(parsed) = super::parse_hhmm(text) else { return false }; at = parsed;
        }
    }
    if let Some(monthly) = &rule.monthly {
        if !(1..=31).contains(&monthly.day) { return false; }
        if let Some(text) = &monthly.at {
            let Some(parsed) = super::parse_hhmm(text) else { return false }; at = parsed;
        }
    }
    if rule.every_days.is_some_and(|days| !(1..=3660).contains(&days)) { return false; }
    // A maximum31-day duration plus skipped local dates needs only this bounded
    // lookback. Check newest first so end-current-occurrence never cancels a
    // later recurrence. Duration is elapsed seconds; start is local wall time.
    let today = now_time.with_timezone(&tz).date_naive();
    for offset in 0..=33 {
        let Some(date) = today.checked_sub_days(chrono::Days::new(offset)) else { continue };
        if date < anchor_date { continue; }
        let matches = if let Some(days) = rule.every_days {
            (date - anchor_date).num_days() % i64::from(days) == 0
        } else if let Some(monthly) = &rule.monthly {
            super::month_day_clamped(date.year(), date.month(), monthly.day) == Some(date)
        } else { weekdays.contains(&date.weekday()) };
        if !matches { continue; }
        let Some(start) = super::local_at(tz, date, at).map(|t| t.timestamp()) else { continue };
        if start < row.starts_at || start > now { continue; }
        let Some(end) = start.checked_add(i64::from(rule.duration_secs)) else { continue };
        if now < end && row.occurrence_ended_at.is_none_or(|ended| ended < start || now < ended) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ts(s: &str) -> i64 { DateTime::parse_from_rfc3339(s).unwrap().timestamp() }
    fn row() -> SchedulerWindow {
        SchedulerWindow { starts_at:ts("2026-03-22T01:30:00Z"),ends_at:None,
            tz:"Europe/Berlin".into(),recurrence_json:Some(r#"{"weekly":{"days":["sun"],"at":"02:30"},"durationSecs":3600}"#.into()),occurrence_ended_at:None }
    }
    #[test]
    fn recurrence_gap_fold_and_ending_only_current_occurrence() {
        let mut r=row();
        assert!(!active(&r,ts("2026-03-29T00:59:59Z")));
        assert!(active(&r,ts("2026-03-29T01:00:00Z")));
        assert!(!active(&r,ts("2026-03-29T02:00:00Z")));
        r.occurrence_ended_at=Some(ts("2026-03-29T01:20:00Z"));
        assert!(!active(&r,ts("2026-03-29T01:21:00Z")));
        assert!(active(&r,ts("2026-04-05T00:30:00Z")));
        assert!(active(&r,ts("2026-10-25T00:30:00Z")));
        assert!(!active(&r,ts("2026-10-25T01:30:00Z")),"fold occurs only once");
    }
    #[test]
    fn monthly_every_days_and_malformed_are_bounded() {
        let mut r=row();r.starts_at=ts("2026-01-01T00:00:00Z");r.tz="UTC".into();
        r.recurrence_json=Some(r#"{"monthly":{"day":31,"at":"03:00"},"durationSecs":60}"#.into());
        assert!(active(&r,ts("2026-02-28T03:00:00Z")));
        r.recurrence_json=Some(r#"{"everyDays":2,"durationSecs":60}"#.into());
        assert!(active(&r,ts("2026-01-03T00:00:00Z")));
        assert!(!active(&r,ts("2026-01-02T00:00:00Z")));
        for raw in ["null","{}",r#"{"everyDays":0,"durationSecs":60}"#,r#"{"everyDays":1,"durationSecs":0}"#,r#"{"everyDays":1,"weekly":{"days":["sun"]},"durationSecs":60}"#,r#"{"everyDays":1,"durationSecs":60,"unknown":true}"#] {
            r.recurrence_json=Some(raw.into());assert!(!active(&r,ts("2026-01-03T00:00:00Z")),"{raw}");
        }
    }
    #[test]
    fn one_off_boundaries_and_whole_skipped_date() {
        let mut r=row();r.recurrence_json=None;r.ends_at=Some(r.starts_at+60);
        assert!(active(&r,r.starts_at));assert!(!active(&r,r.starts_at-1));assert!(!active(&r,r.starts_at+60));
        r.occurrence_ended_at=Some(r.starts_at+20);assert!(!active(&r,r.starts_at+20));
        let date=chrono::NaiveDate::from_ymd_opt(2011,12,30).unwrap();
        let at=super::super::local_at("Pacific/Apia".parse().unwrap(),date,NaiveTime::from_hms_opt(0,0,0).unwrap()).unwrap();
        assert_eq!(at.timestamp(),ts("2011-12-30T10:00:00Z"));
    }
}
