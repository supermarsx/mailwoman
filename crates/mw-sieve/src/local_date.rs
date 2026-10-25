use super::*;
use chrono::{Datelike, FixedOffset, TimeZone};

fn zone(value: &str) -> Option<FixedOffset> {
    if value.len() != 5 || !value.is_ascii() || !matches!(&value[..1], "+" | "-") {
        return None;
    }
    let hours: i32 = value[1..3].parse().ok()?;
    let minutes: i32 = value[3..].parse().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    FixedOffset::east_opt(
        (hours * 3600 + minutes * 60) * if value.starts_with('-') { -1 } else { 1 },
    )
}
pub(super) fn validate(test: &Test) -> Result<bool, ScriptError> {
    tags(
        &test.arguments,
        &[
            "is",
            "contains",
            "matches",
            "value",
            "count",
            "comparator",
            "zone",
            "originalzone",
            "index",
            "last",
        ],
        test.span,
    )?;
    let current = test.is("currentdate");
    if current
        && (tagged(&test.arguments, "originalzone")
            || tagged(&test.arguments, "index")
            || tagged(&test.arguments, "last"))
    {
        return Ok(false);
    }
    if tagged(&test.arguments, "zone")
        && (tagged(&test.arguments, "originalzone")
            || tagged_values(&test.arguments, "zone")
                .is_none_or(|v| v.len() != 1 || zone(&v[0]).is_none()))
    {
        return Ok(false);
    }
    let values = strings(&test.arguments);
    let part = if current { 0 } else { 1 };
    Ok(values.len() == part + 2
        && values[part].len() == 1
        && (current || values[0].len() == 1)
        && [
            "year", "month", "day", "date", "julian", "hour", "minute", "second", "time",
            "iso8601", "std11", "zone", "weekday",
        ]
        .contains(&values[part][0].as_str()))
}
pub(super) fn matches(test: &Test, message: &Message) -> bool {
    let values = strings(&test.arguments);
    let current = test.is("currentdate");
    let date = if current {
        chrono::Utc
            .timestamp_opt(message.current_time.unwrap_or(0), 0)
            .single()
            .map(|d| d.fixed_offset())
    } else {
        header_values(message, &values[0][0], &test.arguments)
            .first()
            .and_then(|value| {
                let value = if values[0][0].eq_ignore_ascii_case("received") {
                    value.rsplit_once(';').map_or(*value, |(_, date)| date)
                } else {
                    value
                };
                chrono::DateTime::parse_from_rfc2822(value.trim()).ok()
            })
    };
    let Some(date) = date else {
        return false;
    };
    let offset = tagged_values(&test.arguments, "zone")
        .and_then(|v| zone(&v[0]))
        .unwrap_or_else(|| {
            if tagged(&test.arguments, "originalzone") {
                *date.offset()
            } else {
                FixedOffset::east_opt(0).unwrap()
            }
        });
    let date = date.with_timezone(&offset);
    let part = if current { 0 } else { 1 };
    let rendered = match values[part][0].as_str() {
        "year" => date.format("%Y").to_string(),
        "month" => date.format("%m").to_string(),
        "day" => date.format("%d").to_string(),
        "date" => date.format("%Y-%m-%d").to_string(),
        "hour" => date.format("%H").to_string(),
        "minute" => date.format("%M").to_string(),
        "second" => date.format("%S").to_string(),
        "time" => date.format("%H:%M:%S").to_string(),
        "iso8601" => date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "std11" => date.to_rfc2822(),
        "zone" => date.format("%z").to_string(),
        "weekday" => date.weekday().num_days_from_sunday().to_string(),
        "julian" => date
            .date_naive()
            .signed_duration_since(chrono::NaiveDate::from_ymd_opt(1858, 11, 17).unwrap())
            .num_days()
            .to_string(),
        _ => return false,
    };
    values[part + 1].iter().any(|key| {
        matches_text(
            &test.arguments,
            key,
            if tagged(&test.arguments, "count") {
                "1"
            } else {
                &rendered
            },
        )
    })
}

#[test]
fn date_parts_zones_index_and_stable_clock() {
    let message = Message {
        current_time: Some(0),
        headers: vec![
            ("Date".into(), "Thu, 01 Jan 1970 01:00:00 +0100".into()),
            ("Date".into(), "Fri, 02 Jan 1970 00:00:00 +0000".into()),
        ],
        ..Default::default()
    };
    for test in [
        "currentdate :zone \"+0000\" :is \"iso8601\" \"1970-01-01T00:00:00Z\"",
        "currentdate :value \"ge\" \"date\" \"1970-01-01\"",
        "date :originalzone :is \"Date\" \"hour\" \"01\"",
        "date :index 1 :last :is \"Date\" \"day\" \"02\"",
        "date :is \"Date\" \"julian\" \"40587\"",
    ] {
        let source = format!(
            "require [\"date\",\"relational\",\"index\"]; if {test} {{keep;}} else {{discard;}}"
        );
        assert_eq!(
            evaluate(&source, &message).unwrap(),
            vec![Action::Keep],
            "{test}"
        );
    }
}
