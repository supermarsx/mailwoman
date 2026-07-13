//! Bundled holiday packs (plan §3 e9, SPEC §11): a region index at
//! `GET /api/holidays` and a subscribable iCalendar feed at
//! `GET /api/holidays/{region}`.
//!
//! The packs are compiled into the binary as structured data and emitted as a
//! valid RFC 5545 `VCALENDAR` on demand — each holiday is an all-day
//! (`VALUE=DATE`) `VEVENT` with `RRULE:FREQ=YEARLY`, so one small event covers
//! every year. Only fixed-date holidays are bundled (correct for every year
//! without an Easter / nth-weekday computation); richer regional packs (`.hol`
//! import via `mw_ics::parse_hol`) are a follow-up (plan §0 cut list). The feed
//! is read-only and cookie-authed like every other endpoint.

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// One fixed-date holiday (recurs yearly).
struct Holiday {
    /// Month (1-12).
    month: u32,
    /// Day of month (1-31).
    day: u32,
    /// The `SUMMARY` shown in the calendar.
    summary: &'static str,
}

/// A bundled, subscribable holiday region.
struct Region {
    /// URL slug (`/api/holidays/{id}`), lowercase ASCII.
    id: &'static str,
    /// Human-readable name for the region index + `X-WR-CALNAME`.
    name: &'static str,
    holidays: &'static [Holiday],
}

/// The bundled packs. Fixed-date holidays only (see module docs).
static REGIONS: &[Region] = &[
    Region {
        id: "us",
        name: "United States",
        holidays: &[
            Holiday {
                month: 1,
                day: 1,
                summary: "New Year's Day",
            },
            Holiday {
                month: 6,
                day: 19,
                summary: "Juneteenth",
            },
            Holiday {
                month: 7,
                day: 4,
                summary: "Independence Day",
            },
            Holiday {
                month: 11,
                day: 11,
                summary: "Veterans Day",
            },
            Holiday {
                month: 12,
                day: 25,
                summary: "Christmas Day",
            },
        ],
    },
    Region {
        id: "uk",
        name: "United Kingdom",
        holidays: &[
            Holiday {
                month: 1,
                day: 1,
                summary: "New Year's Day",
            },
            Holiday {
                month: 12,
                day: 25,
                summary: "Christmas Day",
            },
            Holiday {
                month: 12,
                day: 26,
                summary: "Boxing Day",
            },
        ],
    },
    Region {
        id: "ca",
        name: "Canada",
        holidays: &[
            Holiday {
                month: 1,
                day: 1,
                summary: "New Year's Day",
            },
            Holiday {
                month: 7,
                day: 1,
                summary: "Canada Day",
            },
            Holiday {
                month: 11,
                day: 11,
                summary: "Remembrance Day",
            },
            Holiday {
                month: 12,
                day: 25,
                summary: "Christmas Day",
            },
            Holiday {
                month: 12,
                day: 26,
                summary: "Boxing Day",
            },
        ],
    },
    Region {
        id: "international",
        name: "International Observances",
        holidays: &[
            Holiday {
                month: 1,
                day: 1,
                summary: "New Year's Day",
            },
            Holiday {
                month: 3,
                day: 8,
                summary: "International Women's Day",
            },
            Holiday {
                month: 4,
                day: 22,
                summary: "Earth Day",
            },
            Holiday {
                month: 12,
                day: 10,
                summary: "Human Rights Day",
            },
            Holiday {
                month: 12,
                day: 25,
                summary: "Christmas Day",
            },
        ],
    },
];

/// `GET /api/holidays` — the region index the web client lists for subscription.
pub(crate) fn regions_response() -> Response {
    let regions: Vec<_> = REGIONS
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "name": r.name,
                "count": r.holidays.len(),
                "url": format!("/api/holidays/{}", r.id),
            })
        })
        .collect();
    Json(json!({ "regions": regions })).into_response()
}

/// `GET /api/holidays/{region}` — the bundled pack for one region as
/// `text/calendar`, or `404` for an unknown slug.
pub(crate) fn feed_response(region: &str) -> Response {
    let slug = region.trim_end_matches(".ics").to_ascii_lowercase();
    let Some(pack) = REGIONS.iter().find(|r| r.id == slug) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("unknown holiday region '{region}'") })),
        )
            .into_response();
    };
    let ics = emit_ics(pack);
    let mut resp = Response::new(ics.into());
    let h = resp.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/calendar; charset=utf-8"),
    );
    if let Ok(v) = HeaderValue::from_str(&format!("inline; filename=\"{}.ics\"", pack.id)) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    resp
}

/// Serialize one region's pack to a valid RFC 5545 `VCALENDAR` (CRLF-delimited).
fn emit_ics(region: &Region) -> String {
    let mut out = String::new();
    push_line(&mut out, "BEGIN:VCALENDAR");
    push_line(&mut out, "VERSION:2.0");
    push_line(&mut out, "PRODID:-//Mailwoman//Holidays//EN");
    push_line(&mut out, "CALSCALE:GREGORIAN");
    push_line(&mut out, "METHOD:PUBLISH");
    push_line(&mut out, &format!("X-WR-CALNAME:{} Holidays", region.name));
    for hol in region.holidays {
        push_line(&mut out, "BEGIN:VEVENT");
        // Stable UID so a re-subscribe updates rather than duplicates.
        push_line(
            &mut out,
            &format!(
                "UID:{}-{:02}{:02}@holidays.mailwoman",
                region.id, hol.month, hol.day
            ),
        );
        push_line(&mut out, "DTSTAMP:19700101T000000Z");
        // All-day, date-valued (floating) start; a yearly rule spans every year.
        push_line(
            &mut out,
            &format!("DTSTART;VALUE=DATE:1970{:02}{:02}", hol.month, hol.day),
        );
        push_line(&mut out, "RRULE:FREQ=YEARLY");
        push_line(&mut out, &format!("SUMMARY:{}", hol.summary));
        // Holidays do not consume free/busy time.
        push_line(&mut out, "TRANSP:TRANSPARENT");
        push_line(&mut out, "END:VEVENT");
    }
    push_line(&mut out, "END:VCALENDAR");
    out
}

/// Append one iCalendar content line with the required CRLF terminator.
fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push_str("\r\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_region_emits_a_wellformed_vcalendar() {
        for region in REGIONS {
            let ics = emit_ics(region);
            assert!(ics.starts_with("BEGIN:VCALENDAR\r\n"), "{}", region.id);
            assert!(ics.trim_end().ends_with("END:VCALENDAR"), "{}", region.id);
            assert!(ics.contains("VERSION:2.0"));
            // One VEVENT (with a matching END) per bundled holiday.
            assert_eq!(
                ics.matches("BEGIN:VEVENT").count(),
                region.holidays.len(),
                "{}",
                region.id
            );
            assert_eq!(
                ics.matches("END:VEVENT").count(),
                region.holidays.len(),
                "{}",
                region.id
            );
            assert!(ics.contains("RRULE:FREQ=YEARLY"));
            // CRLF line endings (no bare LF).
            assert!(!ics.contains("\n\n"));
            for line in ics.split("\r\n") {
                assert!(!line.contains('\n'), "bare LF leaked into a content line");
            }
        }
    }

    #[test]
    fn region_ids_are_unique_and_slug_safe() {
        let mut seen = std::collections::HashSet::new();
        for region in REGIONS {
            assert!(seen.insert(region.id), "duplicate region id {}", region.id);
            assert!(
                region.id.chars().all(|c| c.is_ascii_lowercase()),
                "region id {} is not a lowercase slug",
                region.id
            );
            assert!(!region.holidays.is_empty(), "{} has no holidays", region.id);
        }
    }
}
