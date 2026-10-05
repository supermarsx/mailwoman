//! RFC 8098 reports: `build_mdn` → `parse_mdn`, reports written elsewhere,
//! values that must not add a header line, a part or a field, and inputs that
//! must not panic.

use mail_parser::{Message, MessageParser, MimeHeaders};
use mw_mime::{
    EmailAddress, MdnActionMode, MdnDisposition, MdnInput, MdnReport, MdnSendingMode, build_mdn,
    parse, parse_mdn,
};

/// After the example report in RFC 8098, with a third part filled in.
const RFC_EXAMPLE: &str = include_str!("fixtures/mdn/rfc8098_example.eml");
/// A report with a `text/rfc822-headers` third part and a named, inline
/// notification part — the layout Thunderbird writes. Hand-written.
const THREE_PART: &str = include_str!("fixtures/mdn/three_part_headers.eml");
/// A report whose notification part is base64, with folded fields, a
/// lower-case field name, `RFC822; ` with a space, `report-type` after the
/// boundary and extension fields. Hand-written.
const VARIANCE: &str = include_str!("fixtures/mdn/variance_base64_folded.eml");

/// A fixture with CRLF line ends, whatever the checkout gave it.
fn crlf(text: &str) -> Vec<u8> {
    text.replace("\r\n", "\n")
        .replace('\n', "\r\n")
        .into_bytes()
}

/// The same fixture with bare LF line ends.
fn lf(text: &str) -> Vec<u8> {
    text.replace("\r\n", "\n").into_bytes()
}

fn input() -> MdnInput {
    MdnInput {
        from: EmailAddress {
            name: Some("Bob Example".into()),
            email: "bob@example.net".into(),
        },
        to: "alice@example.org".into(),
        final_recipient: "bob@example.net".into(),
        original_recipient: None,
        original_message_id: Some("<compose-1@example.org>".into()),
        original_subject: Some("Lunch on Friday".into()),
        action_mode: MdnActionMode::Manual,
        sending_mode: MdnSendingMode::Manual,
        disposition: MdnDisposition::Displayed,
        message_id: Some("mdn-1@example.net".into()),
    }
}

/// One way of making an input hostile.
type Change = Box<dyn Fn(&mut MdnInput)>;

fn reparse(raw: &[u8]) -> Message<'_> {
    MessageParser::default()
        .parse(raw)
        .expect("built report parses")
}

fn shape(message: &Message<'_>) -> Vec<String> {
    message
        .parts
        .iter()
        .map(|p| {
            let ct = p.content_type().expect("every built part has a type");
            format!("{}/{}", ct.ctype(), ct.subtype().unwrap_or(""))
        })
        .collect()
}

fn all_header_names(message: &Message<'_>) -> Vec<String> {
    message
        .parts
        .iter()
        .flat_map(|p| p.headers.iter().map(|h| h.name.as_str().to_string()))
        .collect()
}

/// The body of the `message/disposition-notification` part.
fn fields(message: &Message<'_>) -> String {
    String::from_utf8(message.parts[2].contents().to_vec()).expect("ascii fields")
}

const REPORT_SHAPE: [&str; 3] = [
    "multipart/report",
    "text/plain",
    "message/disposition-notification",
];

#[test]
fn a_built_report_has_the_rfc_8098_layout() {
    let raw = build_mdn(&input()).expect("build");
    let message = reparse(&raw);

    assert_eq!(shape(&message), REPORT_SHAPE);
    let root = message.parts[0].content_type().expect("root type");
    assert_eq!(
        root.attribute("report-type"),
        Some("disposition-notification")
    );
    assert_eq!(
        fields(&message),
        "Final-Recipient: rfc822; bob@example.net\r\n\
         Original-Message-ID: <compose-1@example.org>\r\n\
         Disposition: manual-action/MDN-sent-manually; displayed\r\n"
    );
    assert_eq!(
        message
            .from()
            .and_then(|a| a.first())
            .and_then(|a| a.address()),
        Some("bob@example.net")
    );
    assert_eq!(
        message
            .to()
            .and_then(|a| a.first())
            .and_then(|a| a.address()),
        Some("alice@example.org")
    );
    assert_eq!(message.message_id(), Some("mdn-1@example.net"));
    assert_eq!(
        message.subject(),
        Some("Return receipt (displayed) - Lunch on Friday")
    );
    let text = message.parts[1].text_contents().expect("text part");
    assert!(text.contains("sent to bob@example.net"), "{text}");
    assert!(text.contains("\"Lunch on Friday\""), "{text}");
    assert!(text.contains("does not show that it was read"), "{text}");

    // No program, host or version is named, and a report the user approved is
    // not marked as automatic.
    let names = all_header_names(&message);
    assert!(
        !fields(&message)
            .to_ascii_lowercase()
            .contains("reporting-ua")
    );
    assert!(!names.iter().any(|n| n.eq_ignore_ascii_case("User-Agent")));
    assert!(
        !names
            .iter()
            .any(|n| n.eq_ignore_ascii_case("Auto-Submitted"))
    );
    // The report itself asks for no receipt (RFC 8098 §3).
    assert_eq!(
        parse(&raw)
            .expect("parse")
            .receipt
            .disposition_notification_to,
        None
    );
}

#[test]
fn build_then_parse_is_a_fixed_point() {
    let first_input = MdnInput {
        original_recipient: Some("bob.alias@example.net".into()),
        ..input()
    };
    let first = parse_mdn(&build_mdn(&first_input).expect("build")).expect("parse");
    assert_eq!(
        first,
        MdnReport {
            final_recipient: Some("bob@example.net".into()),
            original_recipient: Some("bob.alias@example.net".into()),
            original_message_id: Some("compose-1@example.org".into()),
            action_mode: Some(MdnActionMode::Manual),
            sending_mode: Some(MdnSendingMode::Manual),
            disposition: MdnDisposition::Displayed,
            modifiers: Vec::new(),
            reporting_ua: None,
        }
    );

    // Building again from what was parsed gives the same report.
    let second_input = MdnInput {
        final_recipient: first.final_recipient.clone().expect("final recipient"),
        original_recipient: first.original_recipient.clone(),
        original_message_id: first.original_message_id.clone(),
        action_mode: first.action_mode.expect("action mode"),
        sending_mode: first.sending_mode.expect("sending mode"),
        disposition: first.disposition.clone(),
        ..input()
    };
    let second = parse_mdn(&build_mdn(&second_input).expect("rebuild")).expect("reparse");
    assert_eq!(first, second);
}

#[test]
fn every_mode_and_type_round_trips() {
    for action in [MdnActionMode::Manual, MdnActionMode::Automatic] {
        for sending in [MdnSendingMode::Manual, MdnSendingMode::Automatic] {
            for disposition in [
                MdnDisposition::Displayed,
                MdnDisposition::Deleted,
                MdnDisposition::Dispatched,
                MdnDisposition::Processed,
            ] {
                let raw = build_mdn(&MdnInput {
                    action_mode: action,
                    sending_mode: sending,
                    disposition: disposition.clone(),
                    ..input()
                })
                .expect("build");
                let report = parse_mdn(&raw).expect("parse");
                assert_eq!(report.action_mode, Some(action));
                assert_eq!(report.sending_mode, Some(sending));
                assert_eq!(report.disposition, disposition);

                let automatic = all_header_names(&reparse(&raw))
                    .iter()
                    .any(|n| n.eq_ignore_ascii_case("Auto-Submitted"));
                assert_eq!(automatic, sending == MdnSendingMode::Automatic);
            }
        }
    }
}

#[test]
fn optional_values_may_be_absent() {
    let raw = build_mdn(&MdnInput {
        original_message_id: None,
        original_subject: None,
        message_id: None,
        ..input()
    })
    .expect("build");
    let message = reparse(&raw);
    assert_eq!(message.subject(), Some("Return receipt (displayed)"));
    assert!(message.message_id().is_some(), "an id is generated");
    let report = parse_mdn(&raw).expect("parse");
    assert_eq!(report.original_message_id, None);
    assert_eq!(report.final_recipient.as_deref(), Some("bob@example.net"));
}

#[test]
fn an_unknown_disposition_type_is_not_written() {
    let err = build_mdn(&MdnInput {
        disposition: MdnDisposition::Other("displayed\r\nX: 1".into()),
        ..input()
    })
    .unwrap_err();
    assert!(!err.to_string().chars().any(char::is_control), "{err:?}");
}

#[test]
fn a_hostile_value_is_refused_before_anything_is_written() {
    build_mdn(&input()).expect("the unmodified input builds");

    let line = "x@example.test\r\nDisposition: automatic-action/MDN-sent-automatically; deleted";
    let cases: Vec<(&str, Change)> = vec![
        (
            "to",
            Box::new(|i| i.to = "a@b\r\nBcc: v@example.test".into()),
        ),
        ("to list", Box::new(|i| i.to = "a@b, v@example.test".into())),
        (
            "to bracket",
            Box::new(|i| i.to = "a@b> <v@example.test".into()),
        ),
        ("to empty", Box::new(|i| i.to = String::new())),
        ("from", Box::new(|i| i.from.email = "a@b\nBcc: v@c".into())),
        ("from NUL", Box::new(|i| i.from.email = "a@b\0".into())),
        ("final", Box::new(|i| i.final_recipient = line.into())),
        (
            "final LF",
            Box::new(|i| i.final_recipient = "a@b\nX: 1".into()),
        ),
        (
            "final CR",
            Box::new(|i| i.final_recipient = "a@b\rX: 1".into()),
        ),
        (
            "final NUL",
            Box::new(|i| i.final_recipient = "a@b\0".into()),
        ),
        (
            "final space",
            Box::new(|i| i.final_recipient = "a@b c".into()),
        ),
        ("final bare", Box::new(|i| i.final_recipient = "bob".into())),
        (
            "final non-ascii",
            Box::new(|i| i.final_recipient = "bøb@example.net".into()),
        ),
        (
            "final U+2028",
            Box::new(|i| i.final_recipient = "a@b\u{2028}c".into()),
        ),
        (
            "original",
            Box::new(|i| i.original_recipient = Some(line.into())),
        ),
        (
            "original id",
            Box::new(|i| i.original_message_id = Some("<a@b>\r\nDisposition: x/y; deleted".into())),
        ),
        (
            "id LF",
            Box::new(|i| i.original_message_id = Some("a@b\nX: 1".into())),
        ),
        (
            "id NUL",
            Box::new(|i| i.original_message_id = Some("a\0@b".into())),
        ),
        (
            "id space",
            Box::new(|i| i.original_message_id = Some("a@b c@d".into())),
        ),
        (
            "id brackets",
            Box::new(|i| i.original_message_id = Some("<a@b> <c@d>".into())),
        ),
        (
            "id empty",
            Box::new(|i| i.original_message_id = Some("<>".into())),
        ),
        (
            "id non-ascii",
            Box::new(|i| i.original_message_id = Some("é@b".into())),
        ),
        (
            "id long",
            Box::new(|i| i.original_message_id = Some("a".repeat(801))),
        ),
        (
            "own id",
            Box::new(|i| i.message_id = Some("m@b\r\nBcc: v@c".into())),
        ),
    ];
    for (what, change) in cases {
        let mut hostile = input();
        change(&mut hostile);
        let err = build_mdn(&hostile).expect_err(&format!("{what}: accepted"));
        assert!(
            !err.to_string().chars().any(char::is_control),
            "{what}: {err:?}"
        );
    }
}

#[test]
fn a_hostile_subject_adds_no_header_part_or_field() {
    let benign = build_mdn(&input()).expect("benign");
    let benign = reparse(&benign);
    assert_eq!(shape(&benign), REPORT_SHAPE);

    // The subject of the original message is the one value here that a
    // stranger chooses. This one tries a header, a part and a field.
    let hostile = "Hi\r\nBcc: victim@example.test\r\nX-Injected: 1\r\n\r\n\
                   --mw_00000000000000000000000000000000\r\n\
                   Content-Type: message/disposition-notification\r\n\r\n\
                   Final-Recipient: rfc822; victim@example.test\r\n\
                   Disposition: automatic-action/MDN-sent-automatically; deleted\r\n\
                   \u{2028}\0\u{85}";
    let raw = build_mdn(&MdnInput {
        original_subject: Some(hostile.into()),
        ..input()
    })
    .expect("a subject is reduced to one line, not refused");
    let message = reparse(&raw);

    assert_eq!(shape(&message), REPORT_SHAPE, "the part tree changed");
    assert_eq!(all_header_names(&message), all_header_names(&benign));
    assert!(message.bcc().is_none());
    assert_eq!(fields(&message), fields(&benign));
    let subject = message.subject().expect("subject");
    assert!(!subject.chars().any(char::is_control), "{subject:?}");
    assert!(
        subject.starts_with("Return receipt (displayed) - Hi"),
        "{subject:?}"
    );

    let report = parse_mdn(&raw).expect("parse");
    assert_eq!(report.disposition, MdnDisposition::Displayed);
    assert_eq!(report.final_recipient.as_deref(), Some("bob@example.net"));
    assert_eq!(report.sending_mode, Some(MdnSendingMode::Manual));
}

#[test]
fn a_long_subject_is_cut() {
    let raw = build_mdn(&MdnInput {
        original_subject: Some("é".repeat(5000)),
        ..input()
    })
    .expect("build");
    let message = reparse(&raw);
    let subject = message.subject().expect("subject");
    assert_eq!(subject.chars().filter(|c| *c == 'é').count(), 200);
    assert_eq!(
        parse_mdn(&raw).expect("parse").disposition,
        MdnDisposition::Displayed
    );
}

#[test]
fn the_rfc_8098_example_parses() {
    let expected = MdnReport {
        final_recipient: Some("Joe_Recipient@example.com".into()),
        original_recipient: Some("Joe_Recipient@example.com".into()),
        original_message_id: Some("199509192301.23456@example.org".into()),
        action_mode: Some(MdnActionMode::Manual),
        sending_mode: Some(MdnSendingMode::Manual),
        disposition: MdnDisposition::Displayed,
        modifiers: Vec::new(),
        reporting_ua: Some("joes-pc.cs.example.com; Foomail 97.1".into()),
    };
    assert_eq!(parse_mdn(&crlf(RFC_EXAMPLE)).as_ref(), Some(&expected));
    assert_eq!(parse_mdn(&lf(RFC_EXAMPLE)).as_ref(), Some(&expected));
}

#[test]
fn a_three_part_report_with_original_headers_parses() {
    let expected = MdnReport {
        final_recipient: Some("bob@example.net".into()),
        original_recipient: None,
        original_message_id: Some("compose-77@example.org".into()),
        action_mode: Some(MdnActionMode::Manual),
        sending_mode: Some(MdnSendingMode::Manual),
        disposition: MdnDisposition::Displayed,
        modifiers: Vec::new(),
        reporting_ua: Some(
            "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Thunderbird/128.3.0".into(),
        ),
    };
    assert_eq!(parse_mdn(&crlf(THREE_PART)).as_ref(), Some(&expected));
    assert_eq!(parse_mdn(&lf(THREE_PART)).as_ref(), Some(&expected));
    // The request header quoted in the third part is not this message's.
    let parsed = parse(&crlf(THREE_PART)).expect("parse");
    assert_eq!(parsed.receipt.disposition_notification_to, None);
    assert_eq!(parsed.receipt.return_path.as_deref(), Some(""));
}

#[test]
fn a_base64_folded_report_parses() {
    let expected = MdnReport {
        final_recipient: Some("carol@corp.example".into()),
        original_recipient: None,
        original_message_id: Some("compose-91@example.org".into()),
        action_mode: Some(MdnActionMode::Automatic),
        sending_mode: Some(MdnSendingMode::Automatic),
        disposition: MdnDisposition::Displayed,
        modifiers: Vec::new(),
        reporting_ua: None,
    };
    assert_eq!(parse_mdn(&crlf(VARIANCE)).as_ref(), Some(&expected));
    assert_eq!(parse_mdn(&lf(VARIANCE)).as_ref(), Some(&expected));
}

/// A report whose notification part has the given body.
fn report_with(fields: &str) -> Vec<u8> {
    format!(
        "From: b@example.net\r\nTo: a@example.org\r\nSubject: r\r\nMIME-Version: 1.0\r\n\
         Content-Type: multipart/report; report-type=disposition-notification; boundary=\"b\"\r\n\
         \r\n--b\r\nContent-Type: text/plain\r\n\r\nhuman\r\n\
         --b\r\nContent-Type: message/disposition-notification\r\n\r\n{fields}\r\n--b--\r\n"
    )
    .into_bytes()
}

#[test]
fn field_variants_are_read_leniently() {
    let read = |fields: &str| parse_mdn(&report_with(fields));

    let r = read("DISPOSITION:manual-action/mdn-sent-manually;DISPLAYED\r\n").expect("case");
    assert_eq!(r.disposition, MdnDisposition::Displayed);
    assert_eq!(r.action_mode, Some(MdnActionMode::Manual));
    assert_eq!(r.sending_mode, Some(MdnSendingMode::Manual));
    assert_eq!(r.final_recipient, None);

    let r = read("Disposition: displayed\r\nFinal-Recipient: <bob@example.net>\r\n")
        .expect("no modes, no address type");
    assert_eq!(r.action_mode, None);
    assert_eq!(r.sending_mode, None);
    assert_eq!(r.final_recipient.as_deref(), Some("bob@example.net"));

    let r = read("Disposition: manual-action/MDN-sent-manually; deleted/error, Warning ,\r\n")
        .expect("modifiers");
    assert_eq!(r.disposition, MdnDisposition::Deleted);
    assert_eq!(r.modifiers, ["error", "warning"]);

    let r = read("Disposition: automatic-action/MDN-sent-automatically; denied\r\n")
        .expect("an RFC 3798 type");
    assert_eq!(r.disposition, MdnDisposition::Other("denied".into()));

    let r = read("Disposition: weird/unknown; processed\r\n").expect("unknown modes");
    assert_eq!((r.action_mode, r.sending_mode), (None, None));
    assert_eq!(r.disposition, MdnDisposition::Processed);

    // The first occurrence of a field is the one read.
    let r = read(
        "Disposition: manual-action/MDN-sent-manually; displayed\r\n\
         Disposition: manual-action/MDN-sent-manually; deleted\r\n\
         Final-Recipient: rfc822; first@example.net\r\n\
         Final-Recipient: rfc822; second@example.net\r\n",
    )
    .expect("duplicates");
    assert_eq!(r.disposition, MdnDisposition::Displayed);
    assert_eq!(r.final_recipient.as_deref(), Some("first@example.net"));
}

#[test]
fn parsed_values_carry_no_control_characters_and_are_bounded() {
    let long = "x".repeat(5000);
    let modifiers = (0..40)
        .map(|n| format!("m{n}"))
        .collect::<Vec<_>>()
        .join(",");
    let r = parse_mdn(&report_with(&format!(
        "Reporting-UA: a\0b\u{7f}c\u{85}d; {long}\r\n\
         Final-Recipient: rfc822; bob\0@example.net\x0b\r\n\
         Original-Message-ID: <a\0b\t@example.org>\r\n\
         Disposition: manual-action/MDN-sent-manually; displayed/{modifiers}\r\n"
    )))
    .expect("parse");
    let strings = [
        r.reporting_ua.clone().expect("ua"),
        r.final_recipient.clone().expect("final"),
        r.original_message_id.clone().expect("id"),
    ];
    for s in &strings {
        assert!(!s.chars().any(char::is_control), "{s:?}");
        assert!(s.chars().count() <= 1000, "{} chars", s.chars().count());
    }
    assert_eq!(strings[1], "bob@example.net");
    assert_eq!(strings[2], "ab@example.org");
    assert_eq!(r.modifiers.len(), 8);
}

#[test]
fn messages_that_are_not_reports_are_none() {
    // A report parses: the cases below differ from it in one respect each.
    assert!(
        parse_mdn(&report_with(
            "Disposition: manual-action/MDN-sent-manually; displayed"
        ))
        .is_some()
    );

    assert_eq!(
        parse_mdn(b"Subject: hi\r\n\r\nDisposition: a/b; displayed\r\n"),
        None
    );
    assert_eq!(parse_mdn(b""), None);
    // No Disposition field, or one with no type.
    assert_eq!(
        parse_mdn(&report_with("Final-Recipient: rfc822; bob@example.net")),
        None
    );
    assert_eq!(
        parse_mdn(&report_with(
            "Disposition: manual-action/MDN-sent-manually;"
        )),
        None
    );
    assert_eq!(parse_mdn(&report_with("Disposition:")), None);
    assert_eq!(parse_mdn(&report_with("")), None);

    let as_text = String::from_utf8(report_with(
        "Disposition: manual-action/MDN-sent-manually; displayed",
    ))
    .expect("utf8");
    // A delivery status report.
    let dsn = as_text.replace(
        "report-type=disposition-notification",
        "report-type=delivery-status",
    );
    assert_eq!(parse_mdn(dsn.as_bytes()), None);
    // The same parts under multipart/mixed.
    let mixed = as_text.replace(
        "multipart/report; report-type=disposition-notification;",
        "multipart/mixed;",
    );
    assert_eq!(parse_mdn(mixed.as_bytes()), None);
    // The notification part has another type.
    let other = as_text.replace("message/disposition-notification", "text/plain");
    assert_eq!(parse_mdn(other.as_bytes()), None);
    // `report-type` missing altogether is tolerated.
    let untyped = as_text.replace(" report-type=disposition-notification;", "");
    assert!(parse_mdn(untyped.as_bytes()).is_some());

    // A report forwarded as an attachment is not a report to this message's
    // recipient.
    let forwarded = format!(
        "From: a@example.org\r\nSubject: fwd\r\nMIME-Version: 1.0\r\n\
         Content-Type: multipart/mixed; boundary=\"outer\"\r\n\r\n\
         --outer\r\nContent-Type: message/rfc822\r\n\r\n{as_text}\r\n--outer--\r\n"
    );
    assert_eq!(parse_mdn(forwarded.as_bytes()), None);
}

/// Deterministic pseudo-random bytes (xorshift), so a failure reproduces.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[test]
fn malformed_input_never_panics() {
    let mut seeds: Vec<Vec<u8>> = [RFC_EXAMPLE, THREE_PART, VARIANCE]
        .iter()
        .flat_map(|f| [crlf(f), lf(f)])
        .collect();
    seeds.push(build_mdn(&input()).expect("build"));

    // Every prefix of every seed.
    for seed in &seeds {
        for len in 0..seed.len() {
            let _ = parse_mdn(&seed[..len]);
        }
    }

    // Seeds with bytes overwritten, removed and inserted.
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for seed in &seeds {
        for _ in 0..400 {
            let mut bytes = seed.clone();
            for _ in 0..(1 + rng.next() % 8) {
                let at = (rng.next() as usize) % bytes.len();
                match rng.next() % 3 {
                    0 => bytes[at] = rng.next() as u8,
                    1 => {
                        bytes.remove(at);
                    }
                    _ => bytes.insert(
                        at,
                        [b'\r', b'\n', b':', b';', b'/', 0, 0xff][(rng.next() % 7) as usize],
                    ),
                }
            }
            let _ = parse_mdn(&bytes);
            let _ = parse(&bytes);
        }
    }

    // Notification bodies built to hit the field parser's edges.
    let long_fold = format!("Disposition: a/b;\r\n{}", " x\r\n".repeat(20_000));
    let big = "D".repeat(200_000);
    let bodies: Vec<&str> = vec![
        "",
        ":",
        "::::",
        ";",
        "/",
        "\r\n\r\n\r\n",
        " leading continuation\r\n\tanother",
        "Disposition",
        "Disposition:",
        "Disposition: ;",
        "Disposition: /;/",
        "Disposition: ;;;;////,,,,",
        "Disposition: a/b/c/d; e/f/g,h;i",
        "Disposition: \u{feff}; \u{2028}",
        "Disposition: manual-action/MDN-sent-manually; displayed/",
        "Final-Recipient:\r\nDisposition: x; y",
        "Final-Recipient: ;\r\nOriginal-Recipient: ;;;<>\r\nDisposition: x; y",
        "Final-Recipient: rfc822; <\r\nOriginal-Message-ID: <<<>>>\r\nDisposition: x; y",
        "Original-Message-ID: >\r\nDisposition: x; y",
        "Disposition: x; y\r\n \r\n\t\r\n ",
        "é: é\r\nDisposition: é/é; é/é,é",
        &long_fold,
        &big,
    ];
    for body in bodies {
        let _ = parse_mdn(&report_with(body));
    }

    // Bytes that are not UTF-8, in the notification part and around it.
    let mut raw = report_with("Disposition: x; displayed");
    let at = raw.len() - 20;
    raw.splice(at..at, [0xff, 0xfe, 0x00, 0xc3, 0x28, b'\r', 0x80]);
    let _ = parse_mdn(&raw);
    for len in [0usize, 1, 2, 3, 64, 4096] {
        let noise: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
        let _ = parse_mdn(&noise);
    }
    let _ = parse_mdn(&vec![0u8; 8192]);
    let _ = parse_mdn(b"Content-Type: multipart/report; boundary=b\r\n\r\n--b\r\nContent-Type: message/disposition-notification\r\nContent-Transfer-Encoding: base64\r\n\r\n!!!!\r\n--b");
    let _ = parse_mdn(b"Content-Type: multipart/report\r\n\r\n");
    let _ = parse_mdn(b"Content-Type: multipart/report; boundary=\r\n\r\n----\r\n");
}
