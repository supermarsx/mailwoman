//! A display name is one RFC 5322 phrase, whatever it holds: nothing in it can
//! close the phrase and start a second address in `From`, `To`, `Cc`, `Bcc` or
//! `Reply-To`.
//!
//! Each built message is read back twice. [`strict_mailboxes`] is a reader
//! written here that follows RFC 5322 §3.4 and does not decode RFC 2047
//! encoded-words — what a provider that picks recipients from the headers may
//! do. `mw_mime::parse` (mail-parser) is the lenient reader, which decodes an
//! encoded-word even inside a quoted-string. Both must find exactly the
//! intended mailboxes, and the lenient one must give the name back.
//!
//! The last tests pin which names are still written by `mail-builder`, byte for
//! byte as before, and which are not.

use mail_builder::headers::Header;
use mail_builder::headers::address::Address;
use mw_mime::{
    ComposeExtras, ComposeRequest, EmailAddress, MdnActionMode, MdnDisposition, MdnInput,
    MdnSendingMode, build, build_mdn, build_with, parse,
};

const FIELDS: [&str; 5] = ["From", "To", "Cc", "Bcc", "Reply-To"];

fn addr(name: Option<&str>, email: &str) -> EmailAddress {
    EmailAddress {
        name: name.map(str::to_string),
        email: email.to_string(),
    }
}

/// A request in which every address header is present, with `name` on the
/// first address of `field`. The list headers hold a second address so that a
/// name is followed by a list separator.
fn request(field: &str, name: Option<&str>) -> ComposeRequest {
    let named = |f: &str, email: &str| addr(if f == field { name } else { None }, email);
    let pair = |f: &str, first: &str, second: &str| {
        vec![named(f, first), addr(Some("Second Person"), second)]
    };
    ComposeRequest {
        from: Some(named("From", "author@example.org")),
        to: pair("To", "friend@example.org", "to2@example.org"),
        cc: pair("Cc", "cc1@example.org", "cc2@example.org"),
        bcc: pair("Bcc", "bcc1@example.org", "bcc2@example.org"),
        reply_to: pair("Reply-To", "reply1@example.org", "reply2@example.org"),
        subject: Some("Names".into()),
        text_body: Some("body".into()),
        message_id: Some("names-1@example.org".into()),
        ..ComposeRequest::default()
    }
}

fn intended(req: &ComposeRequest, field: &str) -> Vec<EmailAddress> {
    match field {
        "From" => req.from.iter().cloned().collect(),
        "To" => req.to.clone(),
        "Cc" => req.cc.clone(),
        "Bcc" => req.bcc.clone(),
        "Reply-To" => req.reply_to.clone(),
        other => panic!("no such field {other}"),
    }
}

fn read_back(email: &mw_mime::Email, field: &str) -> Vec<EmailAddress> {
    match field {
        "From" => email.from.clone(),
        "To" => email.to.clone(),
        "Cc" => email.cc.clone(),
        "Bcc" => email.bcc.clone(),
        "Reply-To" => email.reply_to.clone(),
        other => panic!("no such field {other}"),
    }
    .unwrap_or_default()
}

/// The header section as `(name, unfolded value)` pairs, in order.
fn header_fields(raw: &[u8]) -> Vec<(String, String)> {
    let text = String::from_utf8_lossy(raw);
    let head = text.split("\r\n\r\n").next().expect("a header section");
    let unfolded = head.replace("\r\n\t", "\t").replace("\r\n ", " ");
    unfolded
        .split("\r\n")
        .map(|line| {
            let (name, value) = line
                .split_once(':')
                .unwrap_or_else(|| panic!("not a header line: {line:?}\n{head}"));
            (name.to_string(), value.trim().to_string())
        })
        .collect()
}

fn header_names(raw: &[u8]) -> Vec<String> {
    header_fields(raw).into_iter().map(|(n, _)| n).collect()
}

fn header_value(raw: &[u8], name: &str) -> String {
    let mut found = header_fields(raw).into_iter().filter(|(n, _)| n == name);
    let value = found.next().unwrap_or_else(|| panic!("no {name} header")).1;
    assert!(found.next().is_none(), "{name} written twice");
    value
}

/// The raw bytes of one header, from its name to the CRLF that ends it, folds
/// included.
fn header_bytes(raw: &[u8], name: &str) -> String {
    let text = String::from_utf8_lossy(raw).into_owned();
    let start = if text.starts_with(&format!("{name}: ")) {
        0
    } else {
        text.find(&format!("\r\n{name}: "))
            .unwrap_or_else(|| panic!("no {name} header\n{text}"))
            + 2
    };
    let mut end = start;
    loop {
        end += text[end..].find("\r\n").expect("header ends in CRLF") + 2;
        if !text[end..].starts_with([' ', '\t']) {
            return text[start..end].to_string();
        }
    }
}

/// The addr-specs of an address-list header value, read by the RFC 5322 §3.4
/// grammar: a quoted-string (with its `\` escapes) and a comment are skipped
/// whole, `<…>` is an address, `,` `;` `:` and white space separate, and a bare
/// word holding `@` is an address too. Encoded-words are not decoded.
fn strict_mailboxes(value: &str) -> Vec<String> {
    fn flush(word: &mut String, out: &mut Vec<String>) {
        if word.contains('@') {
            out.push(std::mem::take(word));
        }
        word.clear();
    }
    let mut out = vec![];
    let mut word = String::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                flush(&mut word, &mut out);
                while let Some(q) = chars.next() {
                    match q {
                        '\\' => {
                            chars.next();
                        }
                        '"' => break,
                        _ => {}
                    }
                }
            }
            '(' => {
                flush(&mut word, &mut out);
                let mut depth = 1;
                while depth > 0 {
                    match chars.next() {
                        Some('\\') => {
                            chars.next();
                        }
                        Some('(') => depth += 1,
                        Some(')') => depth -= 1,
                        Some(_) => {}
                        None => break,
                    }
                }
            }
            '<' => {
                word.clear();
                out.push(chars.by_ref().take_while(|c| *c != '>').collect());
            }
            ',' | ';' | ':' => flush(&mut word, &mut out),
            c if c.is_whitespace() => flush(&mut word, &mut out),
            c => word.push(c),
        }
    }
    flush(&mut word, &mut out);
    out
}

#[test]
fn the_strict_reader_sees_a_forged_address_when_there_is_one() {
    // The header the verifier observed for the name `é" <victim@…>, "y`.
    let forged =
        "\"=?utf-8?Q?=C3=A9\"_<victim@example.test>,_\"y?=\" <friend@example.org>, <b@example.org>";
    assert_eq!(
        strict_mailboxes(forged),
        ["victim@example.test", "friend@example.org", "b@example.org"]
    );
    assert_eq!(
        strict_mailboxes("\"a \\\" <x@y>, \\\\\" <real@example.org>, bare@example.org"),
        ["real@example.org", "bare@example.org"]
    );
    assert_eq!(
        strict_mailboxes("team: a@example.org, (c <n@example.org>) <b@example.org>;"),
        ["a@example.org", "b@example.org"]
    );
}

/// Names that try to leave the phrase, or that a reader could take for
/// something other than a name.
fn awkward_names() -> Vec<String> {
    let long_ascii = format!(
        "{} \" <victim@example.test>, \"tail",
        "A very long company name".repeat(4)
    );
    let long_utf8 = format!("{}\" <victim@example.test>, \"", "Ünïcödé ".repeat(30));
    let mut names: Vec<String> = [
        // The verifier's reproduction.
        "é\" <victim@example.test>, \"y",
        // The same without the non-ASCII letter, and with a trailing space.
        "\" <victim@example.test>, \"",
        "x\" <victim@example.test>, \"y ",
        // Quotes and backslashes.
        "é\"",
        "\"",
        "é\\",
        "\\",
        "é\\\" <victim@example.test>, \\\"",
        "a \\\" <victim@example.test>, \\\\",
        "Dr. \"Bob\" Jones",
        "é \"Bob\" Jones",
        // Angle brackets, list separators, comments.
        "é <victim@example.test>",
        "<victim@example.test>",
        "a, victim@example.test",
        "é, victim@example.test",
        "a; victim@example.test",
        "é; <victim@example.test>;",
        "(é\") <victim@example.test>, (\"",
        // Group syntax.
        "a: victim@example.test;",
        "é: victim@example.test;",
        "team: \"x\" <victim@example.test>;",
        // Encoded-word look-alikes.
        "=?utf-8?Q?=22_<victim@example.test>,_=22?=",
        "=?utf-8?B?IiA8dmljdGltQGV4YW1wbGUudGVzdD4sICI=?=",
        "é =?utf-8?Q?=22_<victim@example.test>?=",
        "?= <victim@example.test>, =?utf-8?Q?x",
        // Ordinary names.
        "Alice Example",
        "O'Brien, Pat",
        "Jörg Müller",
        "山田 太郎",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    names.push(long_ascii);
    names.push(long_utf8);
    names
}

fn assert_only_the_intended_mailboxes(raw: &[u8], req: &ComposeRequest, context: &str) {
    let shown = String::from_utf8_lossy(raw);
    let parsed = parse(raw).unwrap_or_else(|e| panic!("{context}: {e}\n{shown}"));
    for field in FIELDS {
        let want = intended(req, field);
        let want_emails: Vec<&str> = want.iter().map(|a| a.email.as_str()).collect();
        let value = header_value(raw, field);
        assert_eq!(
            strict_mailboxes(&value),
            want_emails,
            "{context}: strict reading of {field}: {value}"
        );
        assert_eq!(
            read_back(&parsed.email, field),
            want,
            "{context}: lenient reading of {field}: {value}"
        );
    }
}

#[test]
fn no_display_name_adds_an_address_to_any_header() {
    let control = build(&request("To", Some("Alice Example"))).expect("control builds");
    assert_only_the_intended_mailboxes(&control, &request("To", Some("Alice Example")), "control");

    for name in awkward_names() {
        for field in FIELDS {
            let req = request(field, Some(&name));
            for (which, built) in [
                ("build", build(&req)),
                ("build_with", build_with(&req, &ComposeExtras::default())),
            ] {
                let context = format!("{which}, {field}, name {name:?}");
                let raw = built.unwrap_or_else(|e| panic!("{context}: {e}"));
                assert_eq!(
                    header_names(&raw),
                    header_names(&control),
                    "{context}: the set of headers changed\n{}",
                    String::from_utf8_lossy(&raw)
                );
                assert_only_the_intended_mailboxes(&raw, &req, &context);
            }
        }
    }
}

/// A name holding a control character is refused nowhere (`build` does not
/// check, and `build_with` checks addresses, not names), so it has to stay
/// inside the phrase too. The lenient reader's idea of such a name is not
/// asserted; the mailboxes are.
#[test]
fn a_name_with_a_line_break_or_nul_adds_no_header_and_no_address() {
    let control = build(&request("To", Some("Alice Example"))).expect("control builds");
    for name in [
        "x\r\nBcc: victim@example.test",
        "é\r\nBcc: victim@example.test",
        "é\" <victim@example.test>,\r\n \"y",
        "a\nb",
        "a\rb",
        "é\0\" <victim@example.test>, \"",
        "é\t\"",
    ] {
        for field in FIELDS {
            let req = request(field, Some(name));
            let raw = build(&req).unwrap_or_else(|e| panic!("{field} {name:?}: {e}"));
            let shown = String::from_utf8_lossy(&raw);
            assert_eq!(
                header_names(&raw),
                header_names(&control),
                "{field} {name:?}\n{shown}"
            );
            for f in FIELDS {
                let want: Vec<String> = intended(&req, f).into_iter().map(|a| a.email).collect();
                assert_eq!(
                    strict_mailboxes(&header_value(&raw, f)),
                    want,
                    "{field} {name:?}: {f}\n{shown}"
                );
            }
        }
    }
}

#[test]
fn a_written_phrase_is_folded_into_lines_a_server_accepts() {
    let name = format!("{}\"", "Ünïcödé name ".repeat(60));
    for field in FIELDS {
        let req = request(field, Some(&name));
        let raw = build(&req).expect("builds");
        let text = String::from_utf8_lossy(&raw);
        let head = text.split("\r\n\r\n").next().unwrap();
        for line in head.split("\r\n") {
            assert!(line.len() <= 78, "{} bytes: {line}", line.len());
        }
        assert_only_the_intended_mailboxes(&raw, &req, "long name");
    }
}

/// What `mail-builder` itself writes for `addresses` under `field`.
fn mail_builder_header(field: &str, addresses: &[EmailAddress]) -> String {
    let one = |a: &EmailAddress| Address::new_address(a.name.clone(), a.email.clone());
    let value = if field == "From" {
        one(&addresses[0])
    } else {
        Address::new_list(addresses.iter().map(one).collect())
    };
    let mut out = format!("{field}: ").into_bytes();
    value
        .write_header(&mut out, field.len() + 2)
        .expect("writing to a Vec");
    String::from_utf8(out).expect("utf8")
}

/// Names `mail-builder` already writes as one phrase are still written by it:
/// the bytes of every address header are the ones it produces.
#[test]
fn ordinary_names_are_written_as_before() {
    let long = "A reasonably long department name ".repeat(4);
    let names: Vec<Option<&str>> = vec![
        None,
        Some(""),
        Some("Alice Example"),
        Some("O'Brien, Pat"),
        Some("Dr. \"Bob\" Jones"),
        Some("back\\slash"),
        Some("a <b@c>; d: e"),
        Some("(comment)"),
        Some("Jörg Müller"),
        Some("Müller, Jörg <x@y>; (z)"),
        Some("山田 太郎"),
        Some("trailing space "),
        Some(long.trim()),
        Some(&long),
    ];
    for name in &names {
        for field in FIELDS {
            let req = request(field, *name);
            for built in [build(&req), build_with(&req, &ComposeExtras::default())] {
                let raw = built.expect("builds");
                for f in FIELDS {
                    assert_eq!(
                        header_bytes(&raw, f),
                        mail_builder_header(f, &intended(&req, f)),
                        "{f} with name {name:?} on {field}"
                    );
                }
            }
        }
    }

    // A list long enough to be folded several times.
    let mut req = request("To", Some("Alice Example"));
    req.to = (0..14)
        .map(|i| {
            addr(
                (i % 3 != 0).then_some(["Jörg Müller", "Pat O'Brien", "x"][i % 3]),
                &format!("recipient-number-{i}@a-fairly-long-domain.example.org"),
            )
        })
        .collect();
    let raw = build(&req).expect("builds");
    assert_eq!(header_bytes(&raw, "To"), mail_builder_header("To", &req.to));
    assert_only_the_intended_mailboxes(&raw, &req, "long list");
}

/// The address headers come first and in this order, as they did when
/// `mail-builder` wrote the whole header section.
#[test]
fn the_header_order_is_unchanged() {
    let raw = build(&request("To", Some("Alice Example"))).expect("builds");
    assert_eq!(
        header_names(&raw),
        [
            "From",
            "To",
            "Cc",
            "Bcc",
            "Reply-To",
            "Subject",
            "Message-ID",
            "Date",
            "MIME-Version",
            "Content-Type",
            "Content-Transfer-Encoding",
        ]
    );
    let mut bare = request("To", None);
    bare.cc.clear();
    bare.bcc.clear();
    bare.reply_to.clear();
    let raw = build(&bare).expect("builds");
    assert_eq!(header_names(&raw)[..3], ["From", "To", "Subject"]);
}

/// The names whose bytes differ from what `mail-builder` wrote, with the exact
/// new form.
#[test]
fn the_names_written_differently_and_how() {
    let to = |name: &str| {
        let mut req = request("To", Some(name));
        req.to.truncate(1);
        let raw = build(&req).expect("builds");
        header_bytes(&raw, "To")
    };
    // Not short printable ASCII, and holding `"` or `\`: encoded-words, unquoted.
    assert_eq!(
        to("é\" <victim@example.test>, \"y"),
        "To: =?utf-8?Q?=C3=A9=22_=3Cvictim=40example=2Etest=3E=2C_=22y?=\r\n\t<friend@example.org>\r\n"
    );
    assert_eq!(
        to("Jörg \"Jo\" Müller"),
        "To: =?utf-8?Q?J=C3=B6rg_=22Jo=22_M=C3=BCller?= <friend@example.org>\r\n"
    );
    // Short printable ASCII that looks like an encoded-word.
    assert_eq!(
        to("=?utf-8?Q?x?="),
        "To: =?utf-8?Q?=3D=3Futf=2D8=3FQ=3Fx=3F=3D?= <friend@example.org>\r\n"
    );
    // A control character.
    assert_eq!(
        to("a\r\nb"),
        "To: =?utf-8?Q?a=0D=0Ab?= <friend@example.org>\r\n"
    );
    // In a header that this crate writes, a short printable ASCII name is a
    // quoted-string with `"` and `\` escaped.
    let mut req = request("To", Some("é\""));
    req.to[1].name = Some("Pat \"P\" O\\Brien".into());
    let raw = build(&req).expect("builds");
    assert_eq!(
        header_bytes(&raw, "To"),
        "To: =?utf-8?Q?=C3=A9=22?= <friend@example.org>, \"Pat \\\"P\\\" O\\\\Brien\"\r\n\t<to2@example.org>\r\n"
    );
}

// ── the same for a disposition notification ─────────────────────────────────

fn mdn_input(name: Option<&str>) -> MdnInput {
    MdnInput {
        from: addr(name, "bob@example.net"),
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

/// The `From` of a report carries the reporting user's own display name. It is
/// one phrase there as well: the report goes to exactly the one address it was
/// built for, and says it is from exactly one.
#[test]
fn no_display_name_adds_an_address_to_a_disposition_notification() {
    let control = build_mdn(&mdn_input(Some("Bob Example"))).expect("control builds");
    assert_eq!(
        header_bytes(&control, "From"),
        "From: \"Bob Example\" <bob@example.net>\r\n"
    );
    let mut names = awkward_names();
    names.push("é\r\nBcc: victim@example.test".into());
    names.push("x\" <victim@example.test>,\r\n \"y".into());
    for name in names {
        let raw = build_mdn(&mdn_input(Some(&name))).unwrap_or_else(|e| panic!("{name:?}: {e}"));
        let shown = String::from_utf8_lossy(&raw);
        assert_eq!(
            header_names(&raw),
            header_names(&control),
            "{name:?}\n{shown}"
        );
        assert_eq!(
            strict_mailboxes(&header_value(&raw, "From")),
            ["bob@example.net"],
            "{name:?}\n{shown}"
        );
        assert_eq!(
            strict_mailboxes(&header_value(&raw, "To")),
            ["alice@example.org"],
            "{name:?}\n{shown}"
        );
        let parsed = parse(&raw).unwrap_or_else(|e| panic!("{name:?}: {e}"));
        let from = parsed.email.from.unwrap_or_default();
        assert_eq!(from.len(), 1, "{name:?}\n{shown}");
        assert_eq!(from[0].email, "bob@example.net", "{name:?}\n{shown}");
        if !name.chars().any(char::is_control) {
            assert_eq!(from[0].name.as_deref(), Some(name.as_str()), "{shown}");
        }
    }
}

/// A report whose name `mail-builder` already writes as one phrase has the
/// `From` and `To` bytes it wrote.
#[test]
fn an_ordinary_name_on_a_disposition_notification_is_written_as_before() {
    for name in [
        None,
        Some("Bob Example"),
        Some("Jörg Müller"),
        Some("O'Brien, Pat"),
    ] {
        let raw = build_mdn(&mdn_input(name)).expect("builds");
        assert_eq!(
            header_bytes(&raw, "From"),
            mail_builder_header("From", &[addr(name, "bob@example.net")])
        );
        assert_eq!(header_bytes(&raw, "To"), "To: <alice@example.org>\r\n");
        assert_eq!(header_names(&raw)[..3], ["From", "To", "Subject"]);
    }
}
