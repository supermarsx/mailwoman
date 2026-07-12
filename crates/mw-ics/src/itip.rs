//! iTIP `METHOD` framing (RFC 5546) for iMIP scheduling messages.
//!
//! Hand-rolled over the `icalendar` component model (plan §1.2/§2.6): an
//! outbound organiser `REQUEST` / attendee `REPLY` / `COUNTER` / organiser
//! `CANCEL` is the event component wrapped in a `VCALENDAR` carrying a top-level
//! `METHOD`. Inbound, `parse_itip` reads the `METHOD` and the event projection —
//! the mail-ingest invite-detection hook the engine registers.

use icalendar::parser::{Property as PProperty, read_calendar};
use serde_json::Value;

use crate::ical::{self, ParsedIcal};
use crate::{IcsError, Result};

/// The iTIP scheduling method carried by a `text/calendar; method=…` part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ItipMethod {
    Request,
    Reply,
    Counter,
    Cancel,
}

impl ItipMethod {
    fn as_ical(self) -> &'static str {
        match self {
            ItipMethod::Request => "REQUEST",
            ItipMethod::Reply => "REPLY",
            ItipMethod::Counter => "COUNTER",
            ItipMethod::Cancel => "CANCEL",
        }
    }

    fn from_ical(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "REQUEST" => Some(ItipMethod::Request),
            "REPLY" => Some(ItipMethod::Reply),
            "COUNTER" => Some(ItipMethod::Counter),
            "CANCEL" => Some(ItipMethod::Cancel),
            _ => None,
        }
    }
}

/// Build an iTIP `text/calendar` payload framing `event_json` with `method`.
pub fn build_itip(event_json: &Value, method: ItipMethod) -> Result<String> {
    let method_prop = PProperty {
        name: "METHOD".to_string().into(),
        val: method.as_ical().to_string().into(),
        params: vec![],
    };
    Ok(ical::wrap_calendar(
        vec![ical::to_component(event_json)],
        vec![method_prop],
    ))
}

/// Parse an inbound `text/calendar` part → its method + the event projection.
pub fn parse_itip(bytes: &[u8]) -> Result<(ItipMethod, ParsedIcal)> {
    let text = String::from_utf8_lossy(bytes);
    let cal = read_calendar(&text).map_err(IcsError::Ical)?;
    let method = cal
        .properties
        .iter()
        .find(|p| p.name.as_str().eq_ignore_ascii_case("METHOD"))
        .and_then(|p| ItipMethod::from_ical(p.val.as_str()))
        .ok_or_else(|| IcsError::Itip("missing or unknown METHOD".into()))?;

    let parsed = ical::parse_ical(bytes)?
        .into_iter()
        .next()
        .ok_or_else(|| IcsError::Itip("no schedulable component in iTIP message".into()))?;
    Ok((method, parsed))
}
