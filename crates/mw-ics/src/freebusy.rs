//! VFREEBUSY aggregation — merge a set of events into busy intervals over a
//! window (the `Calendar/freeBusy` engine helper, plan §2.2).
//!
//! Each event is expanded (recurrence-aware) within the window; `freeBusyStatus:
//! "free"` events contribute nothing. Overlapping/adjacent busy intervals are
//! coalesced; a merged interval is `"tentative"` only when every contributing
//! event was tentative, else `"busy"`.

use serde_json::Value;

use crate::Result;
use crate::recur::expand_recurrence;

/// A busy interval in a free/busy response.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BusyInterval {
    pub start_utc: String,
    pub end_utc: String,
    /// `"busy"` / `"tentative"`.
    pub status: String,
}

struct Raw {
    start: String,
    end: String,
    tentative: bool,
}

/// Aggregate events into merged VFREEBUSY busy intervals over `[window_start,
/// window_end)` (RFC3339 UTC).
pub fn aggregate_free_busy(
    events_json: &[Value],
    window_start: &str,
    window_end: &str,
) -> Result<Vec<BusyInterval>> {
    let mut raw: Vec<Raw> = vec![];
    for ev in events_json {
        if ev.get("freeBusyStatus").and_then(Value::as_str) == Some("free") {
            continue;
        }
        let tentative = ev.get("status").and_then(Value::as_str) == Some("tentative");
        for inst in expand_recurrence(ev, window_start, window_end)? {
            raw.push(Raw {
                start: inst.start_utc,
                end: inst.end_utc,
                tentative,
            });
        }
    }
    raw.sort_by(|a, b| a.start.cmp(&b.start));

    let mut merged: Vec<BusyInterval> = vec![];
    for r in raw {
        match merged.last_mut() {
            Some(last) if r.start <= last.end_utc => {
                if r.end > last.end_utc {
                    last.end_utc = r.end;
                }
                // Any non-tentative contributor makes the block busy.
                if !r.tentative {
                    last.status = "busy".into();
                }
            }
            _ => merged.push(BusyInterval {
                start_utc: r.start,
                end_utc: r.end,
                status: if r.tentative {
                    "tentative".into()
                } else {
                    "busy".into()
                },
            }),
        }
    }
    Ok(merged)
}
