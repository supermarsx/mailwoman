//! TZID-aware, DST-correct recurrence expansion over the `rrule` crate.
//!
//! The event's `RRULE`/`RDATE`/`EXDATE` and TZID are reassembled into an RFC
//! 5545 recurrence grammar and handed to `rrule::RRuleSet`, which expands in the
//! `DTSTART` time zone — so a weekly 09:00 event keeps wall-clock 09:00 across a
//! DST transition and its UTC offset shifts accordingly (plan §1.12, the
//! DST-boundary gate). Occurrences are returned as UTC instant bounds, the
//! `event_instances` materialization index (plan §2.4).

use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use rrule::{RRuleSet, Tz};
use serde_json::Value;

use crate::dt;
use crate::{IcsError, Result};

/// One expanded recurrence instance: start/end as RFC3339 UTC instants.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Instance {
    pub start_utc: String,
    pub end_utc: String,
}

/// Cap on generated occurrences (guards against runaway/infinite rules).
const EXPAND_LIMIT: u16 = 10_000;

fn date_line(name: &str, local: &str, tz: Option<&str>) -> String {
    let compact = dt::to_compact(local);
    let is_date = !local.contains('T');
    if is_date {
        return format!("{name};VALUE=DATE:{compact}");
    }
    match tz {
        Some("UTC") | Some("Etc/UTC") => format!("{name}:{compact}Z"),
        Some(z) => format!("{name};TZID={z}:{compact}"),
        None => format!("{name}:{compact}"),
    }
}

fn parse_bound(rfc3339: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(rfc3339)
        .map_err(|e| IcsError::Rrule(format!("bad window bound {rfc3339:?}: {e}")))?
        .with_timezone(&Utc))
}

/// Resolve a projection `LocalDateTime` + TZID into a UTC instant.
fn local_to_utc(local: &str, tz: Option<&str>) -> Result<DateTime<Utc>> {
    let naive =
        dt::local_to_naive(local).ok_or_else(|| IcsError::Rrule(format!("bad start {local:?}")))?;
    match tz {
        Some("UTC") | Some("Etc/UTC") | None => Ok(Utc.from_utc_datetime(&naive)),
        Some(z) => {
            let zone: chrono_tz::Tz = z
                .parse()
                .map_err(|_| IcsError::Rrule(format!("unknown time zone {z:?}")))?;
            zone.from_local_datetime(&naive)
                .earliest()
                .map(|dt| dt.with_timezone(&Utc))
                .ok_or_else(|| IcsError::Rrule(format!("invalid local time {local:?} in {z}")))
        }
    }
}

/// Expand a master event's recurrence set within `[window_start, window_end)`
/// (RFC3339 UTC). A non-recurring event yields its single instance when it
/// falls in the window.
pub fn expand_recurrence(
    event_json: &Value,
    window_start: &str,
    window_end: &str,
) -> Result<Vec<Instance>> {
    let start = event_json
        .get("start")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| IcsError::Rrule("event has no start".into()))?;
    let tz = event_json.get("timeZone").and_then(Value::as_str);
    let duration_secs = event_json
        .get("duration")
        .and_then(Value::as_str)
        .and_then(dt::iso_duration_to_secs)
        .unwrap_or(0);

    let win_start = parse_bound(window_start)?;
    let win_end = parse_bound(window_end)?;

    let rules: Vec<&str> = event_json
        .get("recurrenceRules")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|r| r.get("rrule").and_then(Value::as_str))
        .collect();

    // Recurrence overrides split into: RDATE additions (empty patch) and
    // per-instance overrides that reschedule/re-length the matching instance.
    // Every override recurrence-id is injected as an RDATE so it materializes;
    // non-empty ones then remap the occurrence's start/duration by base instant.
    let mut override_rids: Vec<String> = vec![];
    let mut remaps: Vec<(DateTime<Utc>, DateTime<Utc>, i64)> = vec![];
    if let Some(overrides) = event_json
        .get("recurrenceOverrides")
        .and_then(Value::as_object)
    {
        for (rid, patch) in overrides {
            override_rids.push(rid.clone());
            let Some(patch_obj) = patch.as_object() else {
                continue;
            };
            if patch_obj.is_empty() {
                continue; // pure RDATE addition, no remap.
            }
            let base = local_to_utc(rid, tz)?;
            let new_local = patch_obj
                .get("start")
                .and_then(Value::as_str)
                .unwrap_or(rid);
            let new_start = local_to_utc(new_local, tz)?;
            let dur = patch_obj
                .get("duration")
                .and_then(Value::as_str)
                .and_then(dt::iso_duration_to_secs)
                .unwrap_or(duration_secs);
            remaps.push((base, new_start, dur));
        }
    }

    // No RRULE and no RDATE → single occurrence; `rrule` requires an
    // RRULE/RDATE, so resolve the lone DTSTART instant directly.
    let occurrences: Vec<DateTime<Utc>> = if rules.is_empty() && override_rids.is_empty() {
        vec![local_to_utc(start, tz)?]
    } else {
        let mut lines = vec![date_line("DTSTART", start, tz)];
        for r in &rules {
            lines.push(format!("RRULE:{r}"));
        }
        // With no RRULE the DTSTART is not implicitly an occurrence; add it as
        // an RDATE so a pure-RDATE event still yields its base instance.
        if rules.is_empty() {
            lines.push(date_line("RDATE", start, tz));
        }
        for rid in &override_rids {
            lines.push(date_line("RDATE", rid, tz));
        }
        for ex in event_json
            .get("excludedRecurrenceDates")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(d) = ex.as_str() {
                lines.push(date_line("EXDATE", d, tz));
            }
        }
        let set: RRuleSet = lines
            .join("\n")
            .parse()
            .map_err(|e| IcsError::Rrule(format!("{e}")))?;
        let before = win_end.with_timezone(&Tz::UTC);
        let mut dates: Vec<DateTime<Utc>> = set
            .before(before)
            .all(EXPAND_LIMIT)
            .dates
            .into_iter()
            .map(|d| d.with_timezone(&Utc))
            .collect();
        dates.sort_unstable();
        dates.dedup();
        dates
    };

    let mut out = vec![];
    for occ_utc in occurrences {
        // A per-instance override remaps this occurrence's start/duration.
        let (start_utc, dur) = match remaps.iter().find(|(base, _, _)| *base == occ_utc) {
            Some((_, new_start, dur)) => (*new_start, *dur),
            None => (occ_utc, duration_secs),
        };
        if start_utc < win_start || start_utc >= win_end {
            continue;
        }
        let end_utc = start_utc + chrono::Duration::seconds(dur);
        out.push(Instance {
            start_utc: start_utc.to_rfc3339_opts(SecondsFormat::Secs, true),
            end_utc: end_utc.to_rfc3339_opts(SecondsFormat::Secs, true),
        });
    }
    Ok(out)
}
