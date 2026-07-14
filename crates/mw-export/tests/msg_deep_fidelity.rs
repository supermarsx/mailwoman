//! Integration tests for MSG/OFT deep write fidelity (plan t10-e9, §1.6, SPEC §28.8).
//!
//! Two guarantees are exercised through the *public* API (`export_one` +
//! `read_msg`/`from_oft`):
//!
//! 1. **Regression gate (hard):** a message with no custom named properties and
//!    no embedded message exports **byte-identical to the 26.9 floor**. The floor
//!    bytes are vendored as `tests/fixtures/floor_note.{msg,oft}`, captured from
//!    the 26.9 writer before the deep-fidelity layer was added.
//! 2. **Deep fidelity:** custom `X-*` headers round-trip as `__nameid` named
//!    properties, and `message/rfc822` parts round-trip as embedded messages —
//!    including a real Outlook-produced interop fixture.

use mw_export::{Format, RawEmail, export_one, from_oft, read_msg};

/// The exact bytes the 26.9 floor writer produced for this message (no `X-*`
/// headers, no embedded parts). Byte-for-byte match proves the deep layer is
/// additive.
const FLOOR_SAMPLE: &[u8] = b"From: Alice <alice@example.com>\r\n\
To: Bob <bob@example.com>\r\n\
Cc: Carol <carol@example.com>\r\n\
Subject: Quarterly report\r\n\
Message-ID: <abc123@example.com>\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Hello Bob,\r\nHere is the report.\r\n";

const FLOOR_MSG: &[u8] = include_bytes!("fixtures/floor_note.msg");
const FLOOR_OFT: &[u8] = include_bytes!("fixtures/floor_note.oft");

/// The MS-CFB writer (`cfb`) stamps `Timestamp::now()` into every **storage**
/// directory entry's creation/modified-time fields (stream entries are
/// spec-zeroed). Those 16 bytes per storage are the *only* non-deterministic
/// part of the container — the 26.9 floor writer randomised them too. So the
/// honest byte-identical regression gate compares the containers with those
/// timestamp fields zeroed: everything a floor message produces — every stream,
/// every property entry, the directory tree shape — must match to the byte.
///
/// This walks the FAT to the directory chain and zeroes bytes `[100, 116)` of
/// each 128-byte directory entry (the two `FILETIME` fields, MS-CFB §2.6.1).
fn zero_dir_timestamps(bytes: &mut [u8]) {
    const ENTRY: usize = 128;
    const TS_OFF: usize = 100; // creation_time (8) + modified_time (8)
    const TS_LEN: usize = 16;
    if bytes.len() < 512 {
        return;
    }
    let u32_at = |b: &[u8], o: usize| u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
    let sector_shift = u16::from_le_bytes([bytes[30], bytes[31]]);
    let sector_size = 1usize << sector_shift; // 512 for CFB v3
    let first_dir_sector = u32_at(bytes, 48);

    // Sector N begins at (N + 1) * sector_size (sector 0 follows the 512-byte
    // header; for v3 the header occupies exactly one sector).
    let sector_offset = |n: u32| (n as usize + 1) * sector_size;

    // Assemble the FAT from the 109 header DIFAT entries (offset 76). Small
    // containers never need the DIFAT chain, which these fixtures don't.
    let mut fat: Vec<u32> = Vec::new();
    for i in 0..109u32 {
        let difat = u32_at(bytes, 76 + (i as usize) * 4);
        if difat >= 0xFFFF_FFFA {
            continue; // FREESECT / special
        }
        let base = sector_offset(difat);
        if base + sector_size > bytes.len() {
            continue;
        }
        for j in 0..(sector_size / 4) {
            fat.push(u32_at(bytes, base + j * 4));
        }
    }

    // Walk the directory chain, zeroing timestamps in each 128-byte entry.
    let mut sid = first_dir_sector;
    let mut guard = 0;
    while (sid as usize) < fat.len() && sid < 0xFFFF_FFFA && guard < 4096 {
        let base = sector_offset(sid);
        if base + sector_size > bytes.len() {
            break;
        }
        for e in 0..(sector_size / ENTRY) {
            let ts = base + e * ENTRY + TS_OFF;
            if ts + TS_LEN <= bytes.len() {
                bytes[ts..ts + TS_LEN].fill(0);
            }
        }
        sid = fat[sid as usize];
        guard += 1;
    }
}

fn assert_floor_identical(out: Vec<u8>, floor: &[u8], what: &str) {
    let mut out = out;
    let mut floor = floor.to_vec();
    zero_dir_timestamps(&mut out);
    zero_dir_timestamps(&mut floor);
    assert_eq!(
        out, floor,
        "{what}: a message without named props/embedded objects must export \
         byte-identically to the 26.9 floor (modulo cfb's inherent per-storage \
         timestamps, which 26.9 randomised too) — regression gate"
    );
}

#[test]
fn floor_msg_is_byte_identical_to_26_9() {
    let out = export_one(&RawEmail::from(FLOOR_SAMPLE), Format::Msg).unwrap();
    assert_floor_identical(out, FLOOR_MSG, "msg");
}

#[test]
fn floor_oft_is_byte_identical_to_26_9() {
    let out = export_one(&RawEmail::from(FLOOR_SAMPLE), Format::Oft).unwrap();
    assert_floor_identical(out, FLOOR_OFT, "oft");
}

#[test]
fn named_property_round_trips_through_public_api() {
    let raw = b"From: a@example.com\r\n\
Subject: tagged\r\n\
X-Campaign-Id: spring-2026\r\n\
\r\n\
hi\r\n";
    let bytes = export_one(&RawEmail::from(&raw[..]), Format::Msg).unwrap();
    let parsed = read_msg(&bytes).unwrap();
    assert!(
        parsed
            .named_properties
            .iter()
            .any(|p| p.name == "X-Campaign-Id" && p.value == "spring-2026"),
        "named props were {:?}",
        parsed.named_properties
    );
}

#[test]
fn embedded_message_round_trips_through_public_api() {
    let raw = b"From: outer@example.com\r\n\
Subject: wrapper\r\n\
Content-Type: multipart/mixed; boundary=B\r\n\
\r\n\
--B\r\n\
Content-Type: text/plain\r\n\
\r\n\
top\r\n\
--B\r\n\
Content-Type: message/rfc822\r\n\
\r\n\
From: inner@example.com\r\n\
Subject: nested item\r\n\
\r\n\
nested body\r\n\
--B--\r\n";
    let bytes = export_one(&RawEmail::from(&raw[..]), Format::Msg).unwrap();
    let parsed = read_msg(&bytes).unwrap();
    assert_eq!(parsed.embedded.len(), 1);
    assert_eq!(parsed.embedded[0].subject.as_deref(), Some("nested item"));
    assert!(
        parsed.embedded[0]
            .body
            .as_deref()
            .unwrap()
            .contains("nested body")
    );
}

/// The OFT import path shares the reader, so deep fidelity works through
/// `from_oft` too.
#[test]
fn oft_import_recovers_named_property() {
    let raw = b"From: t@example.com\r\n\
Subject: template\r\n\
X-Template-Kind: weekly\r\n\
\r\n\
fill me\r\n";
    let bytes = export_one(&RawEmail::from(&raw[..]), Format::Oft).unwrap();
    let parsed = from_oft(&bytes).unwrap();
    assert!(
        parsed
            .named_properties
            .iter()
            .any(|p| p.name == "X-Template-Kind" && p.value == "weekly")
    );
}

/// Interop: a `.msg` synthesised in the exact shape Outlook writes deep-fidelity
/// containers — a `__nameid` map with a string-named property in
/// `PS_INTERNET_HEADERS`, NUL-terminated UTF-16 value stream, 32-byte header,
/// named value at id `0x8000` — must decode with the named property recovered.
///
/// The fixture (`tests/fixtures/outlook_named_prop.msg`) is byte-shaped
/// independently of our own writer (hand-authored container), so it exercises the
/// reader against a genuinely foreign layout rather than our own round-trip.
const OUTLOOK_NAMED_PROP_MSG: &[u8] = include_bytes!("fixtures/outlook_named_prop.msg");

#[test]
fn reads_outlook_shaped_named_property_fixture() {
    let parsed = read_msg(OUTLOOK_NAMED_PROP_MSG).unwrap();
    assert_eq!(parsed.subject.as_deref(), Some("Outlook deep fixture"));
    assert!(
        parsed
            .named_properties
            .iter()
            .any(|p| p.name == "X-Outlook-Class" && p.value == "Confidential"),
        "named props were {:?}",
        parsed.named_properties
    );
}
