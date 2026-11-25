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

/// [`split`] for an mbox that arrives in pieces (t55): feed it with
/// [`MboxSplitter::push`] and collect each message as soon as the next
/// separator shows it is complete; [`MboxSplitter::finish`] gives the last one.
/// Holds one message plus one partial line, never the whole mbox. The messages
/// are exactly those [`split`] returns for the same bytes.
#[derive(Debug, Default)]
pub struct MboxSplitter {
    current: Option<Vec<u8>>,
    line: Vec<u8>,
}

impl MboxSplitter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed the next piece; returns the messages it completed.
    pub fn push(&mut self, data: &[u8]) -> Vec<Vec<u8>> {
        let mut done = Vec::new();
        let mut rest = data;
        while let Some(at) = rest.iter().position(|&b| b == b'\n') {
            if self.line.is_empty() {
                self.take_line(&rest[..at], &mut done);
            } else {
                let mut line = std::mem::take(&mut self.line);
                line.extend_from_slice(&rest[..at]);
                self.take_line(&line, &mut done);
            }
            rest = &rest[at + 1..];
        }
        self.line.extend_from_slice(rest);
        done
    }

    /// The input is complete: the messages still open (the last one, and the
    /// one before it when the input ends on a separator line).
    pub fn finish(mut self) -> Vec<Vec<u8>> {
        let mut done = Vec::new();
        let line = std::mem::take(&mut self.line);
        self.take_line(&line, &mut done);
        done.extend(self.current.take().map(finish_message));
        done
    }

    /// Bytes held right now (the message being collected and a partial line).
    #[must_use]
    pub fn held(&self) -> usize {
        self.current.as_ref().map_or(0, Vec::len) + self.line.len()
    }

    fn take_line(&mut self, line: &[u8], done: &mut Vec<Vec<u8>>) {
        if is_separator(line) {
            if let Some(msg) = self.current.take() {
                done.push(finish_message(msg));
            }
            self.current = Some(Vec::new());
            return;
        }
        if let Some(buf) = self.current.as_mut() {
            if !buf.is_empty() {
                buf.push(b'\n');
            }
            buf.extend_from_slice(&unquote(line));
        }
    }
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

    /// The incremental splitter returns exactly what `split` does, for any way
    /// the input is cut into pieces.
    #[test]
    fn the_incremental_splitter_matches_split_for_every_piece_size() {
        let mut mbox = Vec::new();
        for i in 0..20 {
            mbox.extend_from_slice(format!("From sender{i}@example.org Mon Jan  1 00:00:00 2024\r\n").as_bytes());
            mbox.extend_from_slice(
                format!("Subject: m{i}\r\n\r\nbody {i}\r\n>From quoted\r\nFrom: not a separator\r\n").as_bytes(),
            );
            if i % 3 == 0 {
                mbox.extend_from_slice(b"\r\n");
            }
        }
        let inputs: Vec<Vec<u8>> = vec![
            mbox.clone(),
            b"preamble\nFrom a\nx".to_vec(),
            b"From a\n".to_vec(),
            b"no separator at all\n".to_vec(),
            Vec::new(),
            b"From a\nline\nFrom b\n\n".to_vec(),
            b"From a\nbody\nFrom b".to_vec(),
        ];
        for input in inputs {
            let want = split(&input);
            for piece in [1usize, 2, 3, 7, 64, 1000, input.len().max(1)] {
                let mut splitter = MboxSplitter::new();
                let mut got = Vec::new();
                for chunk in input.chunks(piece) {
                    got.extend(splitter.push(chunk));
                }
                got.extend(splitter.finish());
                assert_eq!(got, want, "piece size {piece}");
            }
        }
    }

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
