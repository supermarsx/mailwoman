//! Format-level acceptance for the export writers (SPEC §10.5/§10.6).
//!
//! Before 26.19 the crate's tests reached `msg`/`oft`/`docx`/`html2md` and left
//! `markdown.rs` at 0%, `lib.rs` at 15%, `mbox.rs` at 20% and `text.rs` at 31%.
//! Those four are the *framing* layer: the header block, the mboxrd separator and
//! quoting, and the bulk `export_many`/`export_stream` wrappers. `docs/testing/
//! mutation.md` sampled exactly this area and found `export_many`, `export_stream`
//! and `trim_trailing_newline` replaceable by a constant with no test failing.
//!
//! So the assertions here are deliberately *exact* — byte-for-byte separators,
//! exact divider strings, exact round-trips — rather than "it returned `Ok`".
//! Each test states the behaviour it pins.

use mw_export::{Format, RawEmail, export_many, export_one, export_stream, split_mbox};

// ── fixtures ────────────────────────────────────────────────────────────────
//
// LF line endings throughout: `mail-parser` accepts them and it keeps the
// expected strings below readable and unambiguous about where a newline is.

const FULL: &[u8] = b"From: Alice Example <alice@example.org>\n\
To: bob@example.net, Carol <carol@example.org>\n\
Cc: dave@example.org\n\
Subject: Status\n\
Date: Mon, 12 Jan 2026 09:30:00 +0000\n\
Content-Type: text/plain; charset=utf-8\n\
\n\
Line one.\n\
Line two.\n";

const SECOND: &[u8] = b"From: bob@example.net\n\
Subject: Re: Status\n\
Content-Type: text/plain\n\
\n\
Ack.\n";

const HTML_ONLY: &[u8] = b"From: web@example.org\n\
Subject: Newsletter\n\
Content-Type: text/html; charset=utf-8\n\
\n\
<h1>Header</h1><p>Body <strong>text</strong>.</p><script>alert(1)</script>\n";

fn s(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("export output is UTF-8")
}

// ── text.rs: the TXT header block + body ─────────────────────────────────────

/// Pins the TXT layout: a `Key: value` header block, exactly one blank line, the
/// decoded body, and exactly one trailing newline.
#[test]
fn txt_header_block_then_blank_line_then_body() {
    let out = s(export_one(&RawEmail::from(FULL), Format::Txt).expect("txt"));

    let (headers, body) = out.split_once("\n\n").expect("blank line after headers");
    let lines: Vec<&str> = headers.lines().collect();
    assert_eq!(lines[0], "From: Alice Example <alice@example.org>");
    assert_eq!(lines[1], "To: bob@example.net, Carol <carol@example.org>");
    assert_eq!(lines[2], "Cc: dave@example.org");
    assert_eq!(lines[3], "Subject: Status");
    assert!(
        lines[4].starts_with("Date: Mon, 12 Jan 2026 09:30:00"),
        "date header rendered as {:?}",
        lines[4]
    );
    assert_eq!(lines.len(), 5, "no extra header lines: {lines:?}");

    assert_eq!(body, "Line one.\nLine two.\n");
}

/// Absent headers are omitted rather than emitted empty — a message with only a
/// Subject must not carry a `From:`/`To:`/`Cc:`/`Date:` line.
#[test]
fn txt_omits_absent_headers() {
    let raw = b"Subject: Only a subject\n\nbody\n";
    let out = s(export_one(&RawEmail::from(raw.as_slice()), Format::Txt).expect("txt"));
    assert!(out.starts_with("Subject: Only a subject\n\n"), "{out:?}");
    for absent in ["From:", "To:", "Cc:", "Date:"] {
        assert!(!out.contains(absent), "{absent} leaked into {out:?}");
    }
}

/// An HTML-only message still exports readable text with no `<script>` text and
/// no raw tags — the security-relevant half of the guarantee.
///
/// DEFECT (recorded, not fixed): `text.rs` documents "when a message carries only
/// HTML we fall back to the Markdown rendering", but `plain_body`'s
/// `html_to_markdown` branch is unreachable. `mail_parser::Message::body_text`
/// converts an HTML part to text itself rather than returning `None`, so the
/// first branch always wins and the crate's own `html2md` never runs on this
/// path. Observable difference: no Markdown structure survives — `<h1>Header</h1>`
/// arrives as bare `Header` and abuts the next block with no separator
/// (`HeaderBody text.`). `docx.rs` shares `plain_body`, so its module doc makes
/// the same claim about the same dead branch.
#[test]
fn txt_html_only_body_is_flattened_by_mail_parser_not_by_html2md() {
    let out = s(export_one(&RawEmail::from(HTML_ONLY), Format::Txt).expect("txt"));

    // Holds today and must keep holding: no script text, no raw tags.
    assert!(!out.contains("alert"), "script text leaked: {out:?}");
    assert!(!out.contains('<'), "raw HTML survived: {out:?}");

    // Current behaviour — mail-parser's tag stripper, not html2md. If the defect
    // above is fixed this becomes `# Header\n\nBody **text**.` and this
    // assertion is the one to update.
    assert!(out.ends_with("\n\nHeaderBody text.\n"), "{out:?}");
    assert!(
        !out.contains("# Header"),
        "unexpectedly gained Markdown: {out:?}"
    );
}

/// Exactly one trailing newline, whatever trailing whitespace the body carried.
#[test]
fn txt_normalises_the_tail_to_one_newline() {
    let raw = b"Subject: Trailing\n\nbody text   \n\n\n\t\n";
    let out = s(export_one(&RawEmail::from(raw.as_slice()), Format::Txt).expect("txt"));
    assert!(out.ends_with("body text\n"), "{out:?}");
    assert!(!out.ends_with("\n\n"), "{out:?}");
}

/// A display name that is blank/whitespace-only degrades to the bare address
/// rather than emitting `  <addr>`.
#[test]
fn txt_blank_display_name_degrades_to_bare_address() {
    let raw = b"From: \"   \" <spacey@example.org>\nSubject: s\n\nb\n";
    let out = s(export_one(&RawEmail::from(raw.as_slice()), Format::Txt).expect("txt"));
    assert!(out.starts_with("From: spacey@example.org\n"), "{out:?}");
}

// ── markdown.rs: the Markdown header block + rule ────────────────────────────

/// Pins the Markdown layout: bold labels separated by hard breaks, a `---` rule,
/// then the body. Header order is Subject, From, To, Cc, Date.
#[test]
fn markdown_bold_header_block_then_rule_then_body() {
    let out = s(export_one(&RawEmail::from(FULL), Format::Markdown).expect("md"));

    let (head, body) = out
        .split_once("\n\n---\n\n")
        .expect("rule between head and body");
    let lines: Vec<&str> = head.split("  \n").collect();
    assert_eq!(lines[0], "**Subject:** Status");
    assert_eq!(lines[1], "**From:** Alice Example <alice@example.org>");
    assert_eq!(
        lines[2],
        "**To:** bob@example.net, Carol <carol@example.org>"
    );
    assert_eq!(lines[3], "**Cc:** dave@example.org");
    assert!(
        lines[4].starts_with("**Date:** Mon, 12 Jan 2026"),
        "{lines:?}"
    );
    assert_eq!(lines.len(), 5, "{lines:?}");

    // DEFECT (recorded, not fixed): `markdown.rs` documents the body as "the
    // first `text/html` part converted to Markdown, or the first `text/plain`
    // part verbatim". The verbatim branch is unreachable —
    // `mail_parser::Message::body_html` synthesises HTML from a text part rather
    // than returning `None`, so a plain-text body is routed text → HTML →
    // Markdown. Observable: single newlines come back as Markdown hard breaks
    // (`  \n`), so a plain-text body is not verbatim. Pinning the real output.
    assert_eq!(body, "Line one.  \nLine two.\n");
}

/// An HTML body is converted to Markdown (not passed through as HTML), and the
/// `<script>` subtree is dropped.
#[test]
fn markdown_converts_html_body() {
    let out = s(export_one(&RawEmail::from(HTML_ONLY), Format::Markdown).expect("md"));
    assert!(out.contains("# Header"), "{out:?}");
    assert!(out.contains("Body **text**."), "{out:?}");
    assert!(!out.contains("<h1>"), "raw HTML survived: {out:?}");
    assert!(!out.contains("alert"), "script text leaked: {out:?}");
}

/// With no renderable headers there is no rule — the `---` must not appear on a
/// document that has nothing above it to separate.
#[test]
fn markdown_without_headers_has_no_rule() {
    let raw = b"\nJust a body, no headers.\n";
    let out = s(export_one(&RawEmail::from(raw.as_slice()), Format::Markdown).expect("md"));
    assert!(!out.contains("---"), "{out:?}");
    assert!(out.contains("Just a body"), "{out:?}");
}

/// A header block with an empty body still ends with the rule and one newline —
/// no dangling blank block.
#[test]
fn markdown_header_only_message_ends_at_the_rule() {
    let raw = b"Subject: Empty\n\n";
    let out = s(export_one(&RawEmail::from(raw.as_slice()), Format::Markdown).expect("md"));
    assert_eq!(out, "**Subject:** Empty\n\n---\n");
}

// ── mbox.rs: mboxrd framing ──────────────────────────────────────────────────

/// The `From ` separator carries the sender and an asctime date. 12 Jan 2026 was
/// a Monday — this pins that the weekday is computed, not just formatted.
#[test]
fn mbox_separator_carries_sender_and_asctime() {
    let out = s(export_one(&RawEmail::from(FULL), Format::Mbox).expect("mbox"));
    assert!(
        out.starts_with("From alice@example.org Mon Jan 12 09:30:00 2026\n"),
        "{:?}",
        out.lines().next()
    );
}

/// A single-digit day is space-padded to two columns (`Jan  1`, not `Jan 1`) —
/// the asctime column layout other mbox readers expect.
#[test]
fn mbox_asctime_pads_single_digit_day() {
    let raw = b"From: a@example.org\nDate: Thu, 1 Jan 2026 05:06:07 +0000\nSubject: s\n\nb\n";
    let out = s(export_one(&RawEmail::from(raw.as_slice()), Format::Mbox).expect("mbox"));
    assert!(
        out.starts_with("From a@example.org Thu Jan  1 05:06:07 2026\n"),
        "{:?}",
        out.lines().next()
    );
}

/// No usable `From` header → `MAILER-DAEMON`; no usable `Date` → the epoch.
/// Both are the documented fallbacks and both must be stable, since an mbox
/// reader keys message boundaries off this line.
#[test]
fn mbox_separator_falls_back_when_sender_and_date_are_absent() {
    let raw = b"Subject: anonymous\n\nbody\n";
    let out = s(export_one(&RawEmail::from(raw.as_slice()), Format::Mbox).expect("mbox"));
    assert!(
        out.starts_with("From MAILER-DAEMON Thu Jan  1 00:00:00 1970\n"),
        "{:?}",
        out.lines().next()
    );
}

/// mboxrd quoting: a body line that looks like a separator gains one `>`, and an
/// already-quoted one gains another. A `From:` *header* is never quoted.
#[test]
fn mbox_quotes_from_lines_and_leaves_the_header_alone() {
    let raw = b"From: a@example.org\nSubject: quoting\n\n\
From the desk of Alice\n\
>From an earlier quote\n\
>>From deeper still\n\
Not a From line\n";
    let out = s(export_one(&RawEmail::from(raw.as_slice()), Format::Mbox).expect("mbox"));

    assert!(out.contains("\n>From the desk of Alice\n"), "{out:?}");
    assert!(out.contains("\n>>From an earlier quote\n"), "{out:?}");
    assert!(out.contains("\n>>>From deeper still\n"), "{out:?}");
    assert!(out.contains("\nNot a From line\n"), "{out:?}");
    // The `From:` header keeps its single unquoted form (only the separator the
    // writer emitted starts a line with `From ` + space).
    assert!(out.contains("\nFrom: a@example.org\n"), "{out:?}");
    assert_eq!(out.matches("\nFrom: a@example.org\n").count(), 1, "{out:?}");
}

/// An entry ends with a blank line so the next `From ` separator is preceded by
/// one — the framing the reader in `split` depends on.
#[test]
fn mbox_entry_ends_with_a_blank_line() {
    let out = export_one(&RawEmail::from(SECOND), Format::Mbox).expect("mbox");
    assert!(out.ends_with(b"\n\n"), "{:?}", s(out.clone()));
}

const TRICKY: &[u8] = b"From: a@example.org\nSubject: tricky\n\n\
From here on out\n\
>From a quote\n\
ordinary line\n";

/// The point of the quoting: `to_entry` → `split` recovers the message count and
/// un-quotes every `From `-looking body line, so a body line that would have been
/// read as a separator survives intact.
///
/// Compared modulo one trailing newline — see
/// `mbox_split_drops_the_trailing_newline_of_every_entry_but_the_last`.
#[test]
fn mbox_round_trip_recovers_message_count_and_unquotes_bodies() {
    let emails = [
        RawEmail::from(FULL),
        RawEmail::from(TRICKY),
        RawEmail::from(SECOND),
    ];
    let stream = export_many(&emails, Format::Mbox).expect("mbox stream");

    let back = split_mbox(&stream);
    assert_eq!(back.len(), 3, "message count preserved");
    for (got, want) in back.iter().zip([FULL, TRICKY, SECOND]) {
        assert_eq!(
            s(got.clone()).trim_end_matches('\n'),
            s(want.to_vec()).trim_end_matches('\n')
        );
    }
    // The quoted separator-lookalikes came back un-quoted, exactly once each.
    let mid = s(back[1].clone());
    assert!(mid.contains("\nFrom here on out\n"), "{mid:?}");
    assert!(mid.contains("\n>From a quote\n"), "{mid:?}");
    assert!(!mid.contains(">>From"), "over-quoted: {mid:?}");
}

/// DEFECT (recorded, not fixed): `split` loses the terminating newline of every
/// entry except the last, so `split(to_entry(m))[0] != m` byte-for-byte.
///
/// `to_entry` ends an entry with the message's own trailing `\n` plus one blank
/// separator line. In the split stream those two bytes yield a *single* empty
/// field for a non-final entry (`"…two.\n\nFrom …"` splits to `[…, "two.", "",
/// "From …"]`), so the reassembled buffer already holds just the message's own
/// newline — and `finish_message` then pops it unconditionally. The final entry
/// is followed by end-of-input rather than a separator, which yields one extra
/// empty field, so there the pop is correct and the message survives whole.
///
/// Impact: an exported-then-reimported message differs from the original by its
/// terminating line break. Re-parsing still succeeds, so this is fidelity rather
/// than correctness, but `mbox.rs`'s framing comment does not describe it.
#[test]
fn mbox_split_drops_the_trailing_newline_of_every_entry_but_the_last() {
    let emails = [RawEmail::from(FULL), RawEmail::from(SECOND)];
    let stream = export_many(&emails, Format::Mbox).expect("mbox stream");
    let back = split_mbox(&stream);

    assert!(FULL.ends_with(b"\n") && SECOND.ends_with(b"\n"));
    // Non-final entry: the newline is gone.
    assert_eq!(back[0], &FULL[..FULL.len() - 1]);
    // Final entry: intact.
    assert_eq!(back[1], SECOND);
}

/// Content before the first `From ` line is not a message and is dropped.
#[test]
fn mbox_split_ignores_a_preamble() {
    let mut stream = b"junk written by some other tool\n".to_vec();
    stream.extend_from_slice(&export_one(&RawEmail::from(SECOND), Format::Mbox).expect("mbox"));
    let back = split_mbox(&stream);
    assert_eq!(back.len(), 1);
    assert_eq!(back[0], SECOND);
}

/// An empty stream yields no messages (not one empty message).
#[test]
fn mbox_split_of_empty_input_is_empty() {
    assert!(split_mbox(b"").is_empty());
    assert!(split_mbox(b"no separator here\n").is_empty());
}

// ── lib.rs: RawEmail, export_one dispatch, bulk wrappers ─────────────────────

/// The three `RawEmail` constructors agree.
#[test]
fn raw_email_constructors_agree() {
    let bytes = b"Subject: x\n\nbody\n".to_vec();
    let a = RawEmail::new(bytes.clone());
    let b = RawEmail::from(bytes.clone());
    let c = RawEmail::from(bytes.as_slice());
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a.raw, bytes);
}

/// EML is the raw bytes verbatim — the property the module docs promise (a
/// re-parse of the output equals a re-parse of the input).
#[test]
fn eml_is_byte_identical_to_the_input() {
    assert_eq!(
        export_one(&RawEmail::from(FULL), Format::Eml).unwrap(),
        FULL
    );
}

/// Every `Format` variant dispatches to a writer that produces its own container
/// magic. Pins the `export_one` match arms against a mis-wiring.
#[test]
fn export_one_dispatches_each_format_to_its_own_writer() {
    let email = RawEmail::from(FULL);
    const CFB: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

    assert!(
        export_one(&email, Format::Mbox)
            .unwrap()
            .starts_with(b"From ")
    );
    assert!(s(export_one(&email, Format::Txt).unwrap()).starts_with("From: Alice"));
    assert!(s(export_one(&email, Format::Markdown).unwrap()).starts_with("**Subject:**"));
    assert_eq!(&export_one(&email, Format::Msg).unwrap()[..8], CFB);
    assert_eq!(&export_one(&email, Format::Oft).unwrap()[..8], CFB);
    assert_eq!(
        &export_one(&email, Format::Docx).unwrap()[..4],
        b"PK\x03\x04"
    );
}

/// Bulk EML is plain concatenation — no separator is invented.
#[test]
fn export_many_eml_concatenates_verbatim() {
    let emails = [RawEmail::from(FULL), RawEmail::from(SECOND)];
    let out = export_many(&emails, Format::Eml).expect("eml");
    let mut expected = FULL.to_vec();
    expected.extend_from_slice(SECOND);
    assert_eq!(out, expected);
}

/// The conversation divider is exactly `\n\n---\n\n`, appears *between* messages
/// only, and each message's own trailing newline is trimmed so the divider alone
/// controls the spacing. This is the assertion `trim_trailing_newline` and the
/// `first` flag need in order to be pinned.
#[test]
fn export_many_txt_joins_with_exactly_one_divider() {
    let emails = [RawEmail::from(FULL), RawEmail::from(SECOND)];
    let joined = s(export_many(&emails, Format::Txt).expect("txt"));

    let one = s(export_one(&emails[0], Format::Txt).unwrap());
    let two = s(export_one(&emails[1], Format::Txt).unwrap());
    let expected = format!(
        "{}\n\n---\n\n{}",
        one.trim_end_matches(['\n', '\r']),
        two.trim_end_matches(['\n', '\r'])
    );
    assert_eq!(joined, expected);

    assert_eq!(joined.matches("\n\n---\n\n").count(), 1, "{joined:?}");
    assert!(!joined.starts_with("\n\n---"), "no leading divider");
}

/// Same divider contract for Markdown.
#[test]
fn export_many_markdown_joins_with_exactly_one_divider() {
    let emails = [RawEmail::from(FULL), RawEmail::from(SECOND)];
    let joined = s(export_many(&emails, Format::Markdown).expect("md"));
    assert_eq!(joined.matches("\n\n---\n\n").count(), 3, "{joined:?}");
    // Two of those three are each message's own header rule; the middle one is
    // the divider. What must hold is that the second document starts right after
    // a divider that follows the first document's trimmed tail.
    let two = s(export_one(&emails[1], Format::Markdown).unwrap());
    assert!(joined.ends_with(two.trim_end_matches('\n')), "{joined:?}");
}

/// One message gets no divider at all.
#[test]
fn export_many_single_message_has_no_divider() {
    let emails = [RawEmail::from(FULL)];
    let joined = s(export_many(&emails, Format::Txt).expect("txt"));
    assert!(!joined.contains("\n\n---\n\n"), "{joined:?}");
    let one = s(export_one(&emails[0], Format::Txt).unwrap());
    assert_eq!(joined, one.trim_end_matches('\n'));
}

/// An empty set produces an empty document for every format — not a stray
/// divider, not a header.
#[test]
fn export_many_of_nothing_is_empty() {
    for format in [
        Format::Eml,
        Format::Mbox,
        Format::Txt,
        Format::Markdown,
        Format::Msg,
        Format::Oft,
        Format::Docx,
    ] {
        assert!(
            export_many(&[], format).expect("empty export").is_empty(),
            "{format:?} invented bytes for an empty set"
        );
    }
}

/// `export_stream` accepts any iterator of things that borrow a `RawEmail` — the
/// signature that lets a caller stream out of a store instead of collecting
/// first — and produces the same bytes as `export_many`.
#[test]
fn export_stream_matches_export_many_over_a_lazy_iterator() {
    let emails = vec![RawEmail::from(FULL), RawEmail::from(SECOND)];

    let mut streamed = Vec::new();
    export_stream(emails.iter(), Format::Txt, &mut streamed).expect("stream");
    assert_eq!(streamed, export_many(&emails, Format::Txt).unwrap());

    // By value, through a lazy map — nothing is collected into a slice first.
    let mut owned = Vec::new();
    export_stream(
        [FULL, SECOND].into_iter().map(RawEmail::from),
        Format::Mbox,
        &mut owned,
    )
    .expect("stream");
    assert_eq!(owned, export_many(&emails, Format::Mbox).unwrap());
}

/// A write failure is surfaced as `ExportError::Io`, not swallowed or panicked
/// on — the streaming path writes into a caller-supplied sink.
#[test]
fn export_stream_propagates_a_sink_io_error() {
    struct Failing;
    impl std::io::Write for Failing {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("sink is full"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let emails = [RawEmail::from(FULL)];
    let err = export_stream(&emails, Format::Eml, &mut Failing).expect_err("io error");
    assert!(err.to_string().contains("sink is full"), "{err}");
}

/// Binary per-message formats stream as plain concatenation of each writer's
/// output — two CFB containers back to back, not one merged container.
///
/// Compared structurally rather than byte-for-byte: the CFB writer is not
/// reproducible across calls (two `to_msg` runs over the same input differ), so
/// what is pinned here is the framing — a container starts at offset 0 and
/// another starts exactly where the first ends.
#[test]
fn export_many_msg_concatenates_per_message_containers() {
    const CFB: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    let emails = [RawEmail::from(FULL), RawEmail::from(SECOND)];

    let first = export_one(&emails[0], Format::Msg).unwrap();
    let second = export_one(&emails[1], Format::Msg).unwrap();
    let out = export_many(&emails, Format::Msg).expect("msg");

    assert_eq!(out.len(), first.len() + second.len());
    assert_eq!(&out[..8], CFB);
    assert_eq!(&out[first.len()..first.len() + 8], CFB);
}
