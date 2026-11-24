//! Deterministic message corpus for the bulk-operation benchmark (t60).
//!
//! Every message is generated from `(tag, seed)` so the fake upstream never has
//! to keep 100,000 raw messages in memory. The size mix is declared in
//! [`distribution`] and recorded in every benchmark record.
use base64::Engine as _;
use serde_json::{Value, json};

/// Attachment payload before base64 (decoded size).
pub const ATTACHMENT_BYTES: usize = 24 * 1024;
/// Every fifth message carries this word in its body: the many-hit text search.
pub const SEARCH_WORD: &str = "quarterly";
/// Received times: `BASE_EPOCH + seed * 60` seconds, so newest = highest seed.
pub const BASE_EPOCH: i64 = 1_767_225_600; // 2026-01-01T00:00:00Z

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Plain,
    Html,
    Attachment,
}

pub fn shape(seed: u32) -> Shape {
    match seed % 20 {
        0 => Shape::Attachment,
        1..=3 => Shape::Html,
        _ => Shape::Plain,
    }
}

/// The declared size distribution, recorded with every result.
pub fn distribution() -> Value {
    json!({
        "plainText":{"share":0.80,"selector":"seed % 20 in 4..=19","bodyBytes":"600..2599 (600 + seed*37 % 2000)"},
        "html":{"share":0.15,"selector":"seed % 20 in 1..=3","parts":"multipart/alternative text ~1.5 KiB + HTML ~4 KiB"},
        "attachment":{"share":0.05,"selector":"seed % 20 == 0","parts":"multipart/mixed text ~1 KiB + application/octet-stream","attachmentDecodedBytes":ATTACHMENT_BYTES},
        "searchWord":{"word":SEARCH_WORD,"share":0.20,"selector":"seed % 5 == 0"},
        "senders":97,"receivedAt":"2026-01-01T00:00:00Z + seed minutes","generator":"crates/mw-server/tests/common/bulk_ops_corpus.rs"
    })
}

pub fn message_id(tag: &str, seed: u32) -> String {
    format!("<bulk-{tag}-{seed}@corpus.invalid>")
}

pub fn received_at(seed: u32) -> String {
    chrono::DateTime::from_timestamp(BASE_EPOCH + i64::from(seed) * 60, 0)
        .expect("bounded corpus time")
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub fn sender(seed: u32) -> String {
    format!("sender{:02}@corpus.invalid", seed % 97)
}

fn words(seed: u32, bytes: usize) -> String {
    const VOCABULARY: [&str; 16] = [
        "meeting", "invoice", "project", "update", "schedule", "review", "budget", "notes",
        "travel", "contract", "design", "release", "support", "summary", "planning", "draft",
    ];
    let mut text = String::with_capacity(bytes + 16);
    let mut index = seed as usize;
    let mut column = 0;
    while text.len() < bytes {
        let word = VOCABULARY[index % VOCABULARY.len()];
        index = index.wrapping_mul(31).wrapping_add(7);
        text.push_str(word);
        column += word.len() + 1;
        // Lines never start with "From " (mbox-safe) and stay short.
        if column > 70 {
            text.push_str("\r\n");
            column = 0;
        } else {
            text.push(' ');
        }
    }
    text
}

fn text_body(seed: u32, bytes: usize) -> String {
    let marker = if seed % 5 == 0 { format!("The {SEARCH_WORD} figures follow.\r\n") } else { String::new() };
    format!("Message {seed} of the bulk corpus.\r\n{marker}{}\r\n", words(seed, bytes))
}

/// The full RFC 5322 message for `(tag, seed)` addressed to `to`.
pub fn raw(tag: &str, seed: u32, to: &str) -> Vec<u8> {
    let date = chrono::DateTime::from_timestamp(BASE_EPOCH + i64::from(seed) * 60, 0)
        .expect("bounded corpus time")
        .to_rfc2822();
    let headers = format!(
        "From: Sender {} <{}>\r\nTo: {to}\r\nSubject: Corpus message {seed} topic {}\r\nDate: {date}\r\nMessage-ID: {}\r\nMIME-Version: 1.0\r\n",
        seed % 97,
        sender(seed),
        seed % 13,
        message_id(tag, seed)
    );
    let body = match shape(seed) {
        Shape::Plain => format!(
            "{headers}Content-Type: text/plain; charset=utf-8\r\n\r\n{}",
            text_body(seed, 600 + (seed as usize * 37) % 2000)
        ),
        Shape::Html => {
            let text = text_body(seed, 1500);
            let html = format!(
                "<html><body><p>{}</p><table>{}</table></body></html>",
                text.replace("\r\n", "<br>"),
                "<tr><td>row</td><td>value</td></tr>".repeat(60)
            );
            format!(
                "{headers}Content-Type: multipart/alternative; boundary=alt{seed}\r\n\r\n--alt{seed}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{text}\r\n--alt{seed}\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{html}\r\n--alt{seed}--\r\n"
            )
        }
        Shape::Attachment => {
            let payload: Vec<u8> = (0..ATTACHMENT_BYTES).map(|i| (i as u32).wrapping_mul(seed | 1) as u8).collect();
            let encoded = base64::engine::general_purpose::STANDARD.encode(payload);
            let mut wrapped = String::with_capacity(encoded.len() + encoded.len() / 76 * 2);
            for chunk in encoded.as_bytes().chunks(76) {
                wrapped.push_str(std::str::from_utf8(chunk).unwrap());
                wrapped.push_str("\r\n");
            }
            format!(
                "{headers}Content-Type: multipart/mixed; boundary=mix{seed}\r\n\r\n--mix{seed}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}\r\n--mix{seed}\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=report-{seed}.bin\r\nContent-Transfer-Encoding: base64\r\n\r\n{wrapped}--mix{seed}--\r\n",
                text_body(seed, 1000)
            )
        }
    };
    body.into_bytes()
}

/// mboxrd framing of `messages` (lines beginning with `From ` are quoted).
pub fn mbox(messages: impl IntoIterator<Item = Vec<u8>>) -> Vec<u8> {
    let mut out = Vec::new();
    for message in messages {
        out.extend_from_slice(b"From MAILER-DAEMON Thu Jan  1 00:00:00 2026\n");
        for line in message.split_inclusive(|b| *b == b'\n') {
            let bare = line.iter().skip_while(|b| **b == b'>').copied().collect::<Vec<_>>();
            if bare.starts_with(b"From ") {
                out.push(b'>');
            }
            out.extend_from_slice(line);
        }
        if !out.ends_with(b"\n") {
            out.push(b'\n');
        }
        out.push(b'\n');
    }
    out
}

#[test]
fn corpus_is_deterministic_parseable_and_matches_its_declared_mix() {
    let mut counts = [0usize; 3];
    let mut search = 0;
    for seed in 1..=2000u32 {
        let message = raw("t", seed, "user@example.invalid");
        assert_eq!(message, raw("t", seed, "user@example.invalid"));
        let shape = shape(seed);
        counts[shape as usize] += 1;
        let parsed = mw_mime::parse(&message).unwrap();
        assert_eq!(parsed.email.has_attachment, shape == Shape::Attachment, "{seed}");
        let text = String::from_utf8_lossy(&message);
        assert!(!text.lines().any(|line| line.starts_with("From ")));
        if text.contains(SEARCH_WORD) {
            search += 1;
        }
    }
    assert_eq!(counts, [1600, 300, 100]);
    assert_eq!(search, 400);
    assert!(received_at(2) > received_at(1));
    let boxed = mbox([b"Subject: a\r\n\r\nFrom here\r\n".to_vec()]);
    assert!(String::from_utf8(boxed).unwrap().contains("\n>From here"));
}
