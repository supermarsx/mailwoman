//! mbox (mboxrd flavour) framing (plan §3 e3, SPEC §10.5).
//!
//! Each message is prefixed with a `From ` separator line and its body lines are
//! `>`-quoted where they would otherwise be mistaken for a separator: any line
//! matching `^>*From ` gains one leading `>`. [`split`] reverses both, so a
//! round-trip recovers the original message count and un-quotes the bodies.

use mail_parser::{Message, MessageParser};

use crate::{ExportError, Result};

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Serialise one message as a complete mbox entry: `From ` line, `>`-quoted
/// body, and a trailing blank line separating it from the next entry.
pub fn to_entry(raw: &[u8]) -> Result<Vec<u8>> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or_else(|| ExportError::Parse("not a recognisable RFC5322 message".into()))?;

    let mut out = from_line(&message).into_bytes();
    out.extend_from_slice(&quote(raw));
    if !out.ends_with(b"\n") {
        out.push(b'\n');
    }
    out.push(b'\n'); // blank line before the next entry
    Ok(out)
}

/// The mbox `From ` separator: `From <sender> <asctime>`.
fn from_line(message: &Message<'_>) -> String {
    let sender = message
        .from()
        .and_then(|a| a.first())
        .and_then(|a| a.address())
        .filter(|s| !s.is_empty())
        .unwrap_or("MAILER-DAEMON");
    format!("From {sender} {}\n", asctime(message))
}

/// Unix `asctime`-style date for the `From ` line, e.g. `Mon Jan  1 09:30:00 2024`.
fn asctime(message: &Message<'_>) -> String {
    let Some(d) = message.date().filter(|d| d.is_valid()) else {
        return "Thu Jan  1 00:00:00 1970".to_string();
    };
    let wd = WEEKDAYS[(d.day_of_week() as usize) % 7];
    let mon = MONTHS[(d.month.clamp(1, 12) - 1) as usize];
    format!(
        "{wd} {mon} {:>2} {:02}:{:02}:{:02} {}",
        d.day, d.hour, d.minute, d.second, d.year
    )
}

/// Apply mboxrd `>`-quoting to a message's bytes.
fn quote(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len() + 16);
    for (i, line) in split_lines(raw).enumerate() {
        if i > 0 {
            out.push(b'\n');
        }
        if is_from_line(line) {
            out.push(b'>');
        }
        out.extend_from_slice(line);
    }
    out
}

/// A line that mboxrd must quote: zero or more `>` then `From `.
fn is_from_line(line: &[u8]) -> bool {
    let rest = &line[line.iter().take_while(|&&b| b == b'>').count()..];
    rest.starts_with(b"From ")
}

/// An unquoted mbox separator: a line beginning exactly with `From ` (no `>`).
fn is_separator(line: &[u8]) -> bool {
    line.starts_with(b"From ")
}

/// Split an mbox stream back into individual raw messages, reversing the
/// `>`-quoting. Content before the first `From ` line is ignored.
#[must_use]
pub fn split(data: &[u8]) -> Vec<Vec<u8>> {
    let mut messages = Vec::new();
    let mut current: Option<Vec<u8>> = None;
    for line in split_lines(data) {
        if is_separator(line) {
            if let Some(msg) = current.take() {
                messages.push(finish_message(msg));
            }
            current = Some(Vec::new());
            continue;
        }
        if let Some(buf) = current.as_mut() {
            if !buf.is_empty() {
                buf.push(b'\n');
            }
            buf.extend_from_slice(&unquote(line));
        }
    }
    if let Some(msg) = current.take() {
        messages.push(finish_message(msg));
    }
    messages
}

/// Drop the single trailing blank line `to_entry` inserts between entries.
fn finish_message(mut msg: Vec<u8>) -> Vec<u8> {
    if msg.ends_with(b"\n") {
        msg.pop();
        if msg.ends_with(b"\r") {
            msg.pop();
        }
    }
    msg
}

/// Reverse mboxrd quoting: strip one leading `>` from a `^>+From ` line.
fn unquote(line: &[u8]) -> Vec<u8> {
    if line.first() == Some(&b'>') && is_from_line(&line[1..]) {
        line[1..].to_vec()
    } else {
        line.to_vec()
    }
}

/// Iterate the `\n`-delimited lines of `data` without allocating, keeping any
/// trailing `\r` inside each line so CRLF survives the round-trip.
fn split_lines(data: &[u8]) -> impl Iterator<Item = &[u8]> {
    data.split(|&b| b == b'\n')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_and_unquotes_from_lines() {
        assert!(is_from_line(b"From the desk of"));
        assert!(is_from_line(b">From already quoted"));
        assert!(!is_from_line(b"From: header@example.com"));
        assert_eq!(unquote(b">From the desk"), b"From the desk".to_vec());
        assert_eq!(unquote(b">>From x"), b">From x".to_vec());
        assert_eq!(unquote(b"From: header"), b"From: header".to_vec());
    }

    #[test]
    fn header_from_colon_is_not_quoted() {
        // "From:" (colon, not space) must never be treated as a separator.
        assert!(!is_from_line(b"From: a@b.test"));
    }
}
