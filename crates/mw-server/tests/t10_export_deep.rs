//! t10-e14 backend live-E2E — MSG/OFT deep write fidelity (§28.8, e9).
//!
//! Drives the deep-fidelity round-trip through the **public** `mw-export` boundary
//! (`export_one(_, Format::Msg|Oft)` → `read_msg`/`from_oft`) rather than the crate-
//! internal `to_msg`, so this exercises the same surface the web export routes call. A
//! message carrying an `X-*` custom header AND a `message/rfc822` embedded part must
//! survive the CFB round-trip as a `__nameid` named property + an embedded-OLE message;
//! a message WITHOUT those artefacts must stay on the byte-identical floor (the 26.9
//! regression gate). This suite needs no external services — it runs in the default
//! `cargo test` gate on every backend (SQLite/Postgres both, since it touches neither).

use mw_export::{Format, RawEmail, export_one, from_oft, read_msg};

/// A wrapper message with a custom `X-*` header AND an embedded `message/rfc822` part —
/// both deep-fidelity artefacts in one container.
const DEEP: &[u8] = b"From: outer@vogue-homes.com\r\n\
To: dest@example.com\r\n\
Subject: fwd: the sealed note\r\n\
X-Mailwoman-Case: case-4821-alpha\r\n\
Content-Type: multipart/mixed; boundary=BND\r\n\
\r\n\
--BND\r\n\
Content-Type: text/plain\r\n\
\r\n\
Please see the attached original message.\r\n\
--BND\r\n\
Content-Type: message/rfc822\r\n\
\r\n\
From: inner@partner.example\r\n\
To: outer@vogue-homes.com\r\n\
Subject: original enquiry\r\n\
X-Inner-Ref: ref-inner-999\r\n\
\r\n\
The inner body content that must survive nesting.\r\n\
--BND--\r\n";

/// A plain message with NO named props / embedded objects — the byte-identical floor.
const FLOOR: &[u8] = b"From: a@vogue-homes.com\r\n\
To: b@example.com\r\n\
Subject: quarterly numbers\r\n\
Message-ID: <floor-1@vogue-homes.com>\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Body of the floor message.\r\n";

fn to_msg(raw: &[u8]) -> Vec<u8> {
    export_one(&RawEmail::from(raw), Format::Msg).expect("MSG export")
}

// ── 1. Deep MSG: __nameid named property + embedded-OLE message survive ──────────
#[test]
fn msg_deep_named_property_and_embedded_message_survive() {
    let parsed = read_msg(&to_msg(DEEP)).expect("read_msg parses our own writer output");

    // The custom X-* header round-trips as a __nameid named property (MS-OXMSG §2.2.3).
    let props: Vec<(&str, &str)> = parsed
        .named_properties
        .iter()
        .map(|p| (p.name.as_str(), p.value.as_str()))
        .collect();
    assert!(
        props.contains(&("X-Mailwoman-Case", "case-4821-alpha")),
        "outer X-* header must survive as a __nameid named property; got {props:?}"
    );

    // The message/rfc822 part round-trips as an embedded-OLE message
    // (PidTagAttachMethod = afEmbeddedMessage under __substg1.0_3701000D).
    assert_eq!(
        parsed.embedded.len(),
        1,
        "exactly one embedded message expected (not a by-value attachment)"
    );
    let inner = &parsed.embedded[0];
    assert_eq!(inner.subject.as_deref(), Some("original enquiry"));
    assert!(
        inner
            .body
            .as_deref()
            .unwrap_or_default()
            .contains("must survive nesting"),
        "embedded body must survive"
    );
    // The embedded message carries its OWN __nameid scope (its own X-* header).
    let inner_props: Vec<(&str, &str)> = inner
        .named_properties
        .iter()
        .map(|p| (p.name.as_str(), p.value.as_str()))
        .collect();
    assert!(
        inner_props.contains(&("X-Inner-Ref", "ref-inner-999")),
        "embedded message keeps its own named-property scope; got {inner_props:?}"
    );
    // The embedded message must NOT also leak as a top-level by-value attachment.
    assert!(
        parsed.attachments.is_empty(),
        "embedded message is not surfaced as a by-value attachment"
    );
}

/// Whether `hay` contains `needle` encoded as UTF-16LE (how CFB stores storage/stream
/// directory names). Used to prove the floor writer emits NO deep-fidelity storages.
fn contains_utf16le(hay: &[u8], needle: &str) -> bool {
    let n: Vec<u8> = needle.encode_utf16().flat_map(u16::to_le_bytes).collect();
    !n.is_empty() && hay.windows(n.len()).any(|w| w == n.as_slice())
}

// ── 2. Floor: a plain message emits NO deep-fidelity machinery (byte-floor gate) ──
#[test]
fn msg_floor_emits_no_named_prop_or_embedded_machinery() {
    let first = to_msg(FLOOR);

    // The concrete byte-identical-floor gate (§28.8): a message with no X-* headers and
    // no message/rfc822 parts must write NEITHER the __nameid named-property map NOR an
    // embedded-message storage — the deep-fidelity additions are inert on the floor, so
    // its container is structurally identical to the 26.9 floor. (The raw CFB bytes are
    // NOT bit-stable across calls because the `cfb` crate stamps directory entries with
    // wall-clock FILETIMEs — benign container metadata, not payload — so the floor gate
    // is asserted structurally rather than by naive byte-equality.)
    assert!(
        !contains_utf16le(&first, "__nameid_version1.0"),
        "floor message must write NO __nameid named-property storage"
    );
    assert!(
        !contains_utf16le(&first, "__substg1.0_3701000D"),
        "floor message must write NO embedded-message storage"
    );

    let parsed = read_msg(&first).expect("floor MSG parses");
    assert!(
        parsed.named_properties.is_empty(),
        "a message with no X-* headers must write NO __nameid map (floor unchanged)"
    );
    assert!(
        parsed.embedded.is_empty(),
        "a message with no message/rfc822 parts must write NO embedded storage"
    );
    assert_eq!(parsed.subject.as_deref(), Some("quarterly numbers"));
    assert!(
        parsed
            .headers
            .as_deref()
            .unwrap_or_default()
            .contains("Message-ID: <floor-1@vogue-homes.com>"),
        "floor headers preserved"
    );
    // Valid CFB (OLE2) container.
    assert_eq!(
        &first[..8],
        &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
        "output is a well-formed compound file"
    );
}

// ── 3. OFT template path: same deep fidelity through the public Format::Oft writer ──
#[test]
fn oft_deep_named_property_round_trips() {
    let bytes = export_one(&RawEmail::from(DEEP), Format::Oft).expect("OFT export");
    let parsed = from_oft(&bytes).expect("from_oft parses");
    assert!(
        parsed
            .named_properties
            .iter()
            .any(|p| p.name == "X-Mailwoman-Case" && p.value == "case-4821-alpha"),
        "OFT template keeps the named property; got {:?}",
        parsed.named_properties
    );
    assert_eq!(parsed.embedded.len(), 1, "OFT keeps the embedded message");
}

// ── 4. Hostile input never panics (untrusted-reader guard) ───────────────────────
#[test]
fn reader_is_panic_safe_on_garbage() {
    assert!(read_msg(b"not a compound file").is_err());
    assert!(read_msg(&[]).is_err());
    assert!(from_oft(b"garbage").is_err());
    // Valid magic, truncated body → clean error, never a panic.
    let mut trunc = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    trunc.extend_from_slice(&[0u8; 24]);
    let _ = read_msg(&trunc);
}
