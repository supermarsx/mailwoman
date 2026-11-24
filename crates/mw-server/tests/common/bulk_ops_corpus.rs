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
        "plantedTerms":{"quarterly":"seed % 5 == 0","zephyrine":"seed % 2000 == 7","zephyrus":"seed % 1000 == 501","amber lantern (phrase)":"seed % 50 == 3",
            "amber (alone)":"seed % 10 == 6","lantern (alone)":"seed % 7 == 2","subject urgent":"seed % 40 == 11","cc audit@corpus.invalid":"seed % 25 == 0"},
        "flagPatternWhereUsed":{"unread":"seed % 10 < 3 (30%)","flagged":"seed % 50 == 0 (2%)"},
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

/// The planted sentences of a message (each on its own line, so phrase
/// adjacency is exact). The planted words occur nowhere else in the corpus.
pub fn planted(seed: u32) -> Vec<&'static str> {
    let mut lines = Vec::new();
    if seed % 5 == 0 {
        lines.push("The quarterly figures follow.");
    }
    if seed % 2000 == 7 {
        lines.push("Reference zephyrine attached.");
    }
    if seed % 1000 == 501 {
        lines.push("Reference zephyrus attached.");
    }
    if seed % 50 == 3 {
        lines.push("Look for the amber lantern.");
    }
    if seed % 10 == 6 {
        lines.push("An amber note.");
    }
    if seed % 7 == 2 {
        lines.push("The lantern room.");
    }
    lines
}

fn text_body(seed: u32, bytes: usize) -> String {
    let mut marker = String::new();
    for line in planted(seed) {
        marker.push_str(line);
        marker.push_str("\r\n");
    }
    format!("Message {seed} of the bulk corpus.\r\n{marker}{}\r\n", words(seed, bytes))
}

/// `seed % 25 == 0` has this Cc.
pub const CC: &str = "audit@corpus.invalid";

pub fn subject(seed: u32) -> String {
    let extra = if seed % 40 == 11 { " urgent" } else { "" };
    format!("Corpus message {seed} topic {}{extra}", seed % 13)
}

/// `(seen, flagged)` of the flag pattern: 30% unread, 2% flagged.
pub fn pattern_flags(seed: u32) -> (bool, bool) {
    (seed % 10 >= 3, seed % 50 == 0)
}

/// The full RFC 5322 message for `(tag, seed)` addressed to `to`.
pub fn raw(tag: &str, seed: u32, to: &str) -> Vec<u8> {
    let date = chrono::DateTime::from_timestamp(BASE_EPOCH + i64::from(seed) * 60, 0)
        .expect("bounded corpus time")
        .to_rfc2822();
    let cc = if seed % 25 == 0 { format!("Cc: {CC}\r\n") } else { String::new() };
    let headers = format!(
        "From: Sender {} <{}>\r\nTo: {to}\r\n{cc}Subject: {}\r\nDate: {date}\r\nMessage-ID: {}\r\nMIME-Version: 1.0\r\n",
        seed % 97,
        sender(seed),
        subject(seed),
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

/// Tantivy's default analyzer as the index uses it: split on non-alphanumeric
/// characters, lowercase, drop tokens longer than 40 bytes.
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty() && t.len() <= 40)
        .map(str::to_lowercase)
        .collect()
}

/// The indexed fields of one corpus message, rebuilt by brute force from the
/// generator: the oracle for every search scenario.
pub struct Fields {
    pub from: Vec<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub subject: Vec<String>,
    /// Token sequence of each decoded text part (phrases need order).
    pub body: Vec<Vec<String>>,
    pub filename: Vec<String>,
    pub received: i64,
    pub size: u64,
    pub has_attachment: bool,
}
impl Fields {
    pub fn of(tag: &str, seed: u32, to: &str) -> Self {
        let raw = raw(tag, seed, to);
        let shape = shape(seed);
        let text = match shape {
            Shape::Plain => text_body(seed, 600 + (seed as usize * 37) % 2000),
            Shape::Html => text_body(seed, 1500),
            Shape::Attachment => text_body(seed, 1000),
        };
        let mut body = vec![tokenize(&text)];
        if shape == Shape::Html {
            let raw_text = String::from_utf8_lossy(&raw).into_owned();
            let html = raw_text
                .split("Content-Type: text/html; charset=utf-8\r\n\r\n")
                .nth(1)
                .unwrap_or_default()
                .split(&format!("\r\n--alt{seed}--"))
                .next()
                .unwrap_or_default()
                .to_string();
            body.push(tokenize(&html));
        }
        Self {
            from: tokenize(&format!("Sender {} {}", seed % 97, sender(seed))),
            to: tokenize(to),
            cc: if seed % 25 == 0 { tokenize(CC) } else { Vec::new() },
            subject: tokenize(&subject(seed)),
            body,
            filename: if shape == Shape::Attachment { tokenize(&format!("report-{seed}.bin")) } else { Vec::new() },
            received: BASE_EPOCH + i64::from(seed) * 60,
            size: raw.len() as u64,
            has_attachment: shape == Shape::Attachment,
        }
    }
    pub fn body_has(&self, term: &str) -> bool {
        self.body.iter().any(|part| part.iter().any(|t| t == term))
    }
    /// A bare term matches from, to, cc, subject, body or filename.
    pub fn any_has(&self, term: &str) -> bool {
        [&self.from, &self.to, &self.cc, &self.subject, &self.filename].iter().any(|f| f.iter().any(|t| t == term)) || self.body_has(term)
    }
    pub fn any_prefix(&self, prefix: &str) -> bool {
        [&self.from, &self.to, &self.cc, &self.subject, &self.filename].iter().any(|f| f.iter().any(|t| t.starts_with(prefix)))
            || self.body.iter().any(|part| part.iter().any(|t| t.starts_with(prefix)))
    }
    /// Two adjacent tokens in one body part (a quoted phrase).
    pub fn body_phrase(&self, first: &str, second: &str) -> bool {
        self.body.iter().any(|part| part.windows(2).any(|w| w[0] == first && w[1] == second))
    }
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
    // The oracle sees the planted words exactly where they were planted.
    let words: [(&str, fn(u32) -> bool); 5] = [
        ("zephyrine", |s| s % 2000 == 7),
        ("quarterly", |s| s % 5 == 0),
        ("amber", |s| s % 50 == 3 || s % 10 == 6),
        ("lantern", |s| s % 50 == 3 || s % 7 == 2),
        ("urgent", |s| s % 40 == 11),
    ];
    for seed in 1..=2000u32 {
        let fields = Fields::of("t", seed, "user@example.invalid");
        for (word, planted) in words {
            assert_eq!(fields.any_has(word), planted(seed), "{word} {seed}");
        }
        assert_eq!(fields.body_phrase("amber", "lantern"), seed % 50 == 3);
        assert_eq!(fields.any_prefix("zephyr"), seed % 2000 == 7 || seed % 1000 == 501);
        assert_eq!(fields.cc.contains(&"audit".to_string()), seed % 25 == 0);
        assert!(!fields.any_has("qua") && !fields.any_has("quarter"));
    }
    assert!(received_at(2) > received_at(1));
    let boxed = mbox([b"Subject: a\r\n\r\nFrom here\r\n".to_vec()]);
    assert!(String::from_utf8(boxed).unwrap().contains("\n>From here"));
}
