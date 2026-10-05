//! t28-e9: a contact survives `parse_vcard` → `emit_vcard` → `parse_vcard`.
//!
//! Three things are asserted here, each against the text or the projection a
//! second reader would see, never against the emitter's own input:
//!
//! - the projection is a fixed point for a card carrying every property the
//!   projection has a field for (BDAY, ANNIVERSARY, ADR, MEMBER, `KIND:group`
//!   among them), and properties it has no field for come out as written;
//! - no value can make the reader panic or the emitter write a second property;
//! - a document the reader cannot make sense of is an `Err`, not a guess.

use mw_ics::{emit_vcard, parse_vcard};
use serde_json::{Value, json};

/// One card with every property the projection carries, two it does not model
/// (`PHOTO`, `CATEGORIES`), one private extension, one grouped extension and one
/// name no registry knows.
const FULL_VCF: &str = "BEGIN:VCARD\r\n\
VERSION:4.0\r\n\
UID:urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1\r\n\
KIND:individual\r\n\
FN:Dr. Ada King\\, Countess of Lovelace\r\n\
N:King;Ada;Augusta Byron;Dr.;FRS\r\n\
NICKNAME:Ada,AAL\r\n\
ORG:Analytical Engines\\, Ltd.;Research\r\n\
TITLE:Mathematician\r\n\
EMAIL;TYPE=work;PREF=1:ada@example.org\r\n\
EMAIL;TYPE=home:ada@home.example\r\n\
TEL;TYPE=cell:+44 20 7946 0000\r\n\
IMPP;TYPE=work:xmpp:ada@example.org\r\n\
ADR;TYPE=home:PO Box 1;Flat 2;12 St James's Square\\; rear;London;Greater London;SW1Y 4JH;United Kingdom\r\n\
ADR;TYPE=work:;;1 Engine Way;Cambridge;;CB1 1AA;United Kingdom\r\n\
BDAY:18151210\r\n\
ANNIVERSARY:18350708\r\n\
NOTE:First programmer.\\nWrote the notes.\r\n\
KEY:data:application/pgp-keys;base64,mQENBFf3\r\n\
PHOTO:data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==\r\n\
CATEGORIES:pioneers,mathematics\r\n\
X-CUSTOM;X-PARAM=\"a:b;c\":kept: as written\\, with its escapes\r\n\
item1.X-ABLABEL:Poet's daughter\r\n\
FAVOURITE-ENGINE:Analytical\r\n\
END:VCARD\r\n";

fn one(vcf: &str) -> Value {
    let cards = parse_vcard(vcf.as_bytes()).unwrap_or_else(|e| panic!("unparseable: {e}\n{vcf}"));
    assert_eq!(cards.len(), 1, "one card expected\n{vcf}");
    cards[0].json.clone()
}

/// The content lines of an emitted card, with CRLF, a bare LF and a bare CR each
/// taken as a line end — what any reader of the text would see.
fn lines(vcf: &str) -> Vec<String> {
    vcf.replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// The property name of every content line, in order.
fn shape(vcf: &str) -> Vec<String> {
    lines(vcf)
        .iter()
        .map(|l| {
            let end = l.find([':', ';']).unwrap_or(l.len());
            l[..end].to_ascii_uppercase()
        })
        .collect()
}

// ── the fixed point ──────────────────────────────────────────────────────────

#[test]
fn every_projected_property_is_read() {
    let c = one(FULL_VCF);
    assert_eq!(c["uid"], "urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1");
    assert_eq!(c["kind"], "individual");
    assert_eq!(
        c["name"],
        json!({
            "full": "Dr. Ada King, Countess of Lovelace",
            "given": "Ada",
            "surname": "King",
            "additional": "Augusta Byron",
            "prefix": "Dr.",
            "suffix": "FRS",
        })
    );
    assert_eq!(c["nicknames"], json!(["Ada", "AAL"]));
    assert_eq!(
        c["organizations"],
        json!(["Analytical Engines, Ltd.;Research"])
    );
    assert_eq!(c["titles"], json!(["Mathematician"]));
    assert_eq!(
        c["emails"],
        json!([
            { "context": "work", "value": "ada@example.org", "pref": 1 },
            { "context": "home", "value": "ada@home.example", "pref": 0 },
        ])
    );
    assert_eq!(
        c["phones"],
        json!([{ "context": "cell", "value": "+44 20 7946 0000" }])
    );
    assert_eq!(
        c["onlineServices"],
        json!([{ "context": "work", "value": "xmpp:ada@example.org" }])
    );
    assert_eq!(
        c["addresses"],
        json!([
            {
                "context": "home",
                "pobox": "PO Box 1",
                "ext": "Flat 2",
                "street": "12 St James's Square; rear",
                "locality": "London",
                "region": "Greater London",
                "postcode": "SW1Y 4JH",
                "country": "United Kingdom",
            },
            {
                "context": "work",
                "pobox": "",
                "ext": "",
                "street": "1 Engine Way",
                "locality": "Cambridge",
                "region": "",
                "postcode": "CB1 1AA",
                "country": "United Kingdom",
            },
        ])
    );
    assert_eq!(
        c["anniversaries"],
        json!([
            { "kind": "birthday", "date": "1815-12-10" },
            { "kind": "anniversary", "date": "1835-07-08" },
        ])
    );
    assert_eq!(c["notes"], "First programmer.\nWrote the notes.");
    assert_eq!(c["pgpKey"], "data:application/pgp-keys;base64,mQENBFf3");
}

#[test]
fn parse_emit_parse_is_a_fixed_point() {
    let p1 = one(FULL_VCF);
    // Precondition: the card is not trivially stable because it read as empty.
    assert_eq!(p1["addresses"].as_array().unwrap().len(), 2);
    assert_eq!(p1["anniversaries"].as_array().unwrap().len(), 2);

    let emitted = emit_vcard(&p1).unwrap();
    let p2 = one(&emitted);
    assert_eq!(p1, p2, "projection changed across emit\n{emitted}");

    // And the text itself is stable from the first emit on.
    assert_eq!(emit_vcard(&p2).unwrap(), emitted);
}

#[test]
fn properties_without_a_field_come_out_as_written() {
    let emitted = emit_vcard(&one(FULL_VCF)).unwrap();
    let got = lines(&emitted);
    for want in [
        "PHOTO:data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==",
        "CATEGORIES:pioneers,mathematics",
        "X-CUSTOM;X-PARAM=\"a:b;c\":kept: as written\\, with its escapes",
        "item1.X-ABLABEL:Poet's daughter",
        "FAVOURITE-ENGINE:Analytical",
    ] {
        assert!(
            got.iter().any(|l| l == want),
            "missing line {want:?}\n{emitted}"
        );
    }
    // The dates go out in the vCard 4.0 basic form they came in as.
    assert!(got.iter().any(|l| l == "BDAY:18151210"), "{emitted}");
    assert!(got.iter().any(|l| l == "ANNIVERSARY:18350708"), "{emitted}");
    assert!(
        got.iter().any(|l| l == "N:King;Ada;Augusta Byron;Dr.;FRS"),
        "{emitted}"
    );
}

#[test]
fn a_group_card_stays_a_group_with_its_members() {
    let vcf = "BEGIN:VCARD\r\n\
               VERSION:4.0\r\n\
               UID:group-1\r\n\
               KIND:group\r\n\
               FN:The Engine Club\r\n\
               MEMBER:urn:uuid:03a0e51f-d1aa-4385-8a53-e29025acd8af\r\n\
               MEMBER:mailto:ada@example.org\r\n\
               END:VCARD\r\n";
    let p1 = one(vcf);
    assert_eq!(p1["kind"], "group");
    assert_eq!(
        p1["members"],
        json!([
            "urn:uuid:03a0e51f-d1aa-4385-8a53-e29025acd8af",
            "mailto:ada@example.org"
        ])
    );
    let emitted = emit_vcard(&p1).unwrap();
    assert!(
        lines(&emitted).iter().any(|l| l == "KIND:group"),
        "{emitted}"
    );
    assert_eq!(
        shape(&emitted).iter().filter(|n| *n == "MEMBER").count(),
        2,
        "{emitted}"
    );
    assert_eq!(one(&emitted), p1);
}

#[test]
fn vcard3_spellings_are_read_and_stay_stable() {
    // What iOS and Google export: 3.0, upper-case and repeated TYPE parameters,
    // `TYPE=pref`, an extended-format date, a folded line and bare LF endings.
    let vcf = "BEGIN:VCARD\n\
               VERSION:3.0\n\
               N:Hopper;Grace;Brewster Murray;Rear Admiral;\n\
               FN:Grace Hopper\n\
               EMAIL;type=INTERNET;type=WORK;type=pref:grace@example.org\n\
               TEL;TYPE=CELL,VOICE:+1-555-0100\n\
               ADR;TYPE=HOME:;;1 Navy Yard;Arlington;VA;22202;USA\n\
               BDAY:1906-12-09\n\
               NOTE:Found the first\n  bug.\n\
               END:VCARD\n";
    let p1 = one(vcf);
    assert_eq!(p1["name"]["additional"], "Brewster Murray");
    assert_eq!(
        p1["emails"],
        json!([{ "context": "work", "value": "grace@example.org", "pref": 1 }])
    );
    assert_eq!(p1["phones"][0]["context"], "cell");
    assert_eq!(p1["addresses"][0]["locality"], "Arlington");
    assert_eq!(
        p1["anniversaries"],
        json!([{ "kind": "birthday", "date": "1906-12-09" }])
    );
    assert_eq!(p1["notes"], "Found the first bug.");
    assert_eq!(one(&emit_vcard(&p1).unwrap()), p1);
}

#[test]
fn a_date_that_is_not_a_date_survives_as_text() {
    let mut c = one(FULL_VCF);
    c["anniversaries"] = json!([{ "kind": "birthday", "date": "circa 1815; winter" }]);
    let emitted = emit_vcard(&c).unwrap();
    assert_eq!(
        shape(&emitted).iter().filter(|n| *n == "BDAY").count(),
        1,
        "{emitted}"
    );
    assert_eq!(
        one(&emitted)["anniversaries"],
        json!([{ "kind": "birthday", "date": "circa 1815; winter" }])
    );
}

// ── the reader does not panic ────────────────────────────────────────────────

/// Each of these aborted the calling thread inside `uriparse` 0.6.4
/// (`uri.rs:915`, an `unwrap` on an error conversion that has no arm for a
/// colon in the first path segment of a scheme-less reference).
#[test]
fn a_colon_in_a_value_that_is_not_a_uri_is_read_as_text() {
    for (line, field) in [
        ("UID:a@b:c", "uid"),
        ("KEY:a@b:c", "pgpKey"),
        ("TEL:+1:x", "phones"),
        ("IMPP:a@b:c", "onlineServices"),
        ("MEMBER:a@b:c", "members"),
        ("PHOTO:a@b:c", ""),
        ("URL:a@b:c", ""),
        ("X-THING;VALUE=uri:a@b:c", ""),
    ] {
        let vcf = format!("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Mallory\r\n{line}\r\nEND:VCARD\r\n");
        let c = one(&vcf);
        assert_eq!(c["name"]["full"], "Mallory", "{line}");
        let (_, value) = line.split_once(':').unwrap();
        match field {
            "uid" | "pgpKey" => assert_eq!(c[field], value, "{line}"),
            "phones" | "onlineServices" => assert_eq!(c[field][0]["value"], value, "{line}"),
            "members" => assert_eq!(c[field][0], value, "{line}"),
            _ => {}
        }
        // The value is still there after a save.
        let emitted = emit_vcard(&c).unwrap();
        assert!(
            lines(&emitted).iter().any(|l| l == line),
            "{line} lost\n{emitted}"
        );
        assert_eq!(one(&emitted), c, "{line}");
    }
}

#[test]
fn arbitrary_bytes_are_an_error_or_a_card_never_a_panic() {
    let inputs: [&[u8]; 12] = [
        b"",
        b"\xff\xfe garbage",
        b"BEGIN:VCARD",
        b"BEGIN:VCARD\r\nEND:VCARD\r\n",
        b"BEGIN:VCARD\r\nVERSION:4.0\r\n:\r\n;\r\n;;;:::\r\n\"\r\nEND:VCARD\r\n",
        b"BEGIN:VCARD\r\nVERSION:4.0\r\nTEL;TYPE=\"unterminated:+1\r\nEND:VCARD\r\n",
        b"BEGIN:VCARD\r\nVERSION:4.0\r\nADR:;\r\nN:\r\nORG:;;;\r\nNICKNAME:,,\r\nEND:VCARD\r\n",
        b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:trailing backslash\\\r\nEND:VCARD\r\n",
        b"BEGIN:VCARD\r\nVERSION:4.0\r\nEMAIL;PREF=999999999999999999999:x@y\r\nEND:VCARD\r\n",
        b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID://[:::\r\nKEY:http://u:p:q@[::1\r\nEND:VCARD\r\n",
        b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:\xc3\x28\r\nEND:VCARD\r\n",
        b" \r\n\t\r\nBEGIN:VCARD\r\n continuation of nothing\r\nEND:VCARD\r\n",
    ];
    for input in inputs {
        if let Ok(cards) = parse_vcard(input) {
            for c in cards {
                // Whatever was read can be written and read again.
                let emitted = emit_vcard(&c.json).unwrap();
                let again = one(&emitted);
                assert_eq!(one(&emit_vcard(&again).unwrap()), again);
            }
        }
    }
}

#[test]
fn a_document_the_reader_cannot_delimit_is_an_error() {
    assert!(parse_vcard(b"").is_err());
    assert!(parse_vcard(b"FN:no card here\r\n").is_err());
    // Cut off before END: the last card may be missing properties.
    assert!(parse_vcard(b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Cut off\r\n").is_err());
    // A card inside a card has no place in the projection.
    assert!(
        parse_vcard(
            b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Outer\r\nBEGIN:VCARD\r\nFN:Inner\r\nEND:VCARD\r\nEND:VCARD\r\n"
        )
        .is_err()
    );
}

#[test]
fn one_unregistered_property_does_not_fail_the_other_cards() {
    // `LABEL` is a vCard 3.0 property; `vcard4` refused the whole document for it.
    let vcf = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:One\r\nLABEL;TYPE=HOME:1 Main St\r\nEND:VCARD\r\n\
               BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Two\r\nEND:VCARD\r\n";
    let cards = parse_vcard(vcf.as_bytes()).unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].json["name"]["full"], "One");
    assert_eq!(cards[1].json["name"]["full"], "Two");
    let emitted = emit_vcard(&cards[0].json).unwrap();
    assert!(
        lines(&emitted)
            .iter()
            .any(|l| l == "LABEL;TYPE=HOME:1 Main St"),
        "{emitted}"
    );
}

// ── no value leaves its place in the emitted card ────────────────────────────

fn plain() -> Value {
    json!({
        "uid": "card-28@mailwoman",
        "kind": "individual",
        "name": { "full": "Bob Jones", "given": "Bob", "surname": "Jones", "prefix": "", "suffix": "" },
        "emails": [{ "context": "work", "value": "bob@acme.com", "pref": 1 }],
        "phones": [{ "context": "work", "value": "+15551234" }],
        "onlineServices": [{ "context": "work", "value": "xmpp:bob@acme.com" }],
        "addresses": [{ "context": "work", "street": "1 Main St", "locality": "Springfield" }],
    })
}

/// A `:` or `;` in a `context` ended the TYPE parameter: `TYPE=a:b` made `b…`
/// the value, `TYPE=a;X=y` added a parameter.
#[test]
fn a_context_cannot_leave_the_type_parameter() {
    for ctx in [
        "work:victim@example.test",
        "work;PREF=1",
        "work;VALUE=uri:http://evil.example/",
        "a,b",
        "say \"hi\":x",
        "work\r\nEMAIL:victim@example.test",
    ] {
        for field in ["emails", "phones", "onlineServices", "addresses"] {
            let control = one(&emit_vcard(&plain()).unwrap());
            let mut c = plain();
            c[field][0]["context"] = json!(ctx);
            let emitted = emit_vcard(&c).unwrap();
            assert_eq!(
                shape(&emitted),
                shape(&emit_vcard(&plain()).unwrap()),
                "{field} {ctx:?}\n{emitted}"
            );
            let got = one(&emitted);
            // Nothing but the context of that one entry differs from the control.
            let mut expected = control.clone();
            expected[field][0]["context"] = got[field][0]["context"].clone();
            assert_eq!(got, expected, "{field} {ctx:?}\n{emitted}");
            // The context is still one value, with its `:` and `;` inside it.
            let read = got[field][0]["context"].as_str().unwrap();
            for mark in [':', ';', ','] {
                assert_eq!(
                    read.contains(mark),
                    ctx.contains(mark),
                    "{field} {ctx:?} read back as {read:?}\n{emitted}"
                );
            }
            // And it is stable from here on.
            assert_eq!(one(&emit_vcard(&got).unwrap()), got, "{field} {ctx:?}");
        }
    }
}

#[test]
fn an_out_of_range_pref_is_not_written() {
    let mut c = plain();
    c["emails"][0]["pref"] = json!(5000);
    let emitted = emit_vcard(&c).unwrap();
    assert!(!emitted.contains("PREF"), "{emitted}");
}

/// The carried-over lines are server-held, but the emitter is the last gate:
/// whatever sits in that list, it writes one property per entry or nothing.
#[test]
fn a_carried_line_cannot_end_the_card_or_add_a_property() {
    let control = shape(&emit_vcard(&plain()).unwrap());
    let mut c = plain();
    c["vcardExtra"] = json!([
        "END:VCARD",
        "BEGIN:VCARD",
        "VERSION:3.0",
        "EMAIL:victim@example.test",
        "UID:someone-else",
        "no colon here",
        "bad name!:x",
        "X-OK:1\r\nEMAIL:victim@example.test",
        "X-FINE;A=\"b\":kept",
        42,
    ]);
    let emitted = emit_vcard(&c).unwrap();
    let mut want = control.clone();
    let end = want.pop().unwrap();
    want.extend(["X-OK".to_string(), "X-FINE".to_string(), end]);
    assert_eq!(shape(&emitted), want, "{emitted}");
    let got = one(&emitted);
    assert_eq!(got["uid"], "card-28@mailwoman");
    assert_eq!(got["emails"].as_array().unwrap().len(), 1);
}

#[test]
fn a_multi_line_key_keeps_its_line_breaks() {
    let armored = "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\nmQENBFf3\n=abcd\n-----END PGP PUBLIC KEY BLOCK-----";
    let mut c = plain();
    c["pgpKey"] = json!(armored);
    let emitted = emit_vcard(&c).unwrap();
    assert_eq!(
        shape(&emitted).iter().filter(|n| *n == "KEY").count(),
        1,
        "{emitted}"
    );
    assert_eq!(one(&emitted)["pgpKey"], armored);
}

/// Every awkward value in every property position and in a parameter, bare and
/// quoted: whatever the reader makes of the line, writing it and reading it
/// again changes nothing.
#[test]
fn awkward_values_are_stable_wherever_they_sit() {
    let values = [
        r"a\\b",
        r"a\,b",
        r"a\;b",
        "a;b",
        "a,b",
        r"a\nb",
        r"\n",
        " lead",
        "trail ",
        "a:b",
        "\"q\"",
        "a\"b\"c",
        r"a\xb",
        r"trailing\",
        r"a\;",
        r"a\,",
        r"\;",
        r"\,x",
        r"a\\",
        "\u{fc}n\u{ef}",
        "19900415",
        " 1990 ",
        "1990-04-15",
        "--1210",
        "a=b",
        ";",
        ",",
        ";;;;;;;;",
        "pref",
        "INTERNET,Work",
        "org",
        "GROUP",
        "",
    ];
    let heads = [
        "UID",
        "KIND",
        "FN",
        "N",
        "NICKNAME",
        "ORG",
        "TITLE",
        "EMAIL",
        "EMAIL;PREF=7",
        "TEL",
        "IMPP",
        "ADR",
        "BDAY",
        "ANNIVERSARY",
        "BDAY;VALUE=text",
        "MEMBER",
        "NOTE",
        "KEY",
        "PHOTO",
        "X-ANY",
        "item2.URL",
    ];
    let mut docs = vec![];
    for v in values {
        for head in heads {
            // Twice, so that "first one wins" and "later ones are carried" are
            // both on the path.
            docs.push(format!("{head}:{v}\r\n{head}:second {v}\r\n"));
        }
        for prop in ["EMAIL", "TEL", "IMPP", "ADR"] {
            let value = if prop == "ADR" {
                ";;1 Main St;;;;"
            } else {
                "x@y"
            };
            docs.push(format!("{prop};TYPE={v}:{value}\r\n"));
            docs.push(format!("{prop};TYPE=\"{v}\":{value}\r\n"));
            docs.push(format!("{prop};{v}:{value}\r\n"));
            docs.push(format!("{prop};PREF={v};TYPE=home,{v}:{value}\r\n"));
        }
    }
    assert!(docs.len() > 1000);
    for body in docs {
        let vcf = format!("BEGIN:VCARD\r\nVERSION:4.0\r\n{body}END:VCARD\r\n");
        let p1 = one(&vcf);
        let emitted = emit_vcard(&p1).unwrap();
        let p2 = one(&emitted);
        assert_eq!(p1, p2, "from {body:?}\nemitted {emitted:?}");
        assert_eq!(emit_vcard(&p2).unwrap(), emitted, "from {body:?}");
        // One property per line, and the card is still one card.
        let names = shape(&emitted);
        assert_eq!(
            names.iter().filter(|n| *n == "BEGIN").count(),
            1,
            "{emitted:?}"
        );
        assert_eq!(
            names.iter().filter(|n| *n == "END").count(),
            1,
            "{emitted:?}"
        );
    }
}

// ── the t27 verifier's reproduction (26.20 t27-f2) ──────────────────────────

/// The t27 verification report's S4, run against the reader and emitter in
/// this file's crate as they are now. Before the hand-rolled reader and
/// `param_value`, this context gave
/// `EMAIL;TYPE=work;PREF=1;X-EVIL="a:b":evil@e.testEMAIL:real@example.com`
/// and `parse_vcard` panicked on that line inside `uriparse`.
#[test]
fn the_t27_context_reproduction_stays_one_parameter_and_is_read_back() {
    let hostile = "work;PREF=1;X-EVIL=\"a:b\":evil@e.test\r\nEMAIL";
    let mut c = plain();
    c["emails"][0] = json!({ "context": hostile, "value": "real@example.com" });
    let emitted = emit_vcard(&c).unwrap();

    let email_lines: Vec<String> = lines(&emitted)
        .into_iter()
        .filter(|l| l.to_ascii_uppercase().starts_with("EMAIL"))
        .collect();
    assert_eq!(
        email_lines,
        ["EMAIL;TYPE=\"work;PREF=1;X-EVIL='a:b':evil@e.testEMAIL\":real@example.com"],
        "{emitted}"
    );
    assert_eq!(shape(&emitted), shape(&emit_vcard(&plain()).unwrap()));

    // `one` panics on an unreadable card, and a panic in the reader fails the
    // test by itself.
    let got = one(&emitted);
    assert_eq!(
        got["emails"],
        json!([{
            "context": "work;pref=1;x-evil='a:b':evil@e.testemail",
            "value": "real@example.com",
            "pref": 0,
        }])
    );
    assert_eq!(one(&emit_vcard(&got).unwrap()), got);
}

/// Every pairing of a head, a parameter section and a value from the lists
/// below, as one content line of a card. The lists hold the characters the
/// reader splits on, unbalanced quotes, escapes cut short, multi-byte
/// characters next to each of them, and the date shapes `read_date` and the
/// emitter slice by position. The reader returns for each, and what it
/// returns can be written and read again.
#[test]
fn no_content_line_makes_the_reader_or_the_emitter_panic() {
    let heads = [
        "EMAIL",
        "TEL",
        "IMPP",
        "ADR",
        "N",
        "ORG",
        "NICKNAME",
        "BDAY",
        "ANNIVERSARY",
        "UID",
        "KEY",
        "KIND",
        "MEMBER",
        "NOTE",
        "FN",
        "TITLE",
        "X-É",
        "item1.EMAIL",
        ".EMAIL",
        "a.b.TEL",
        "é.TEL",
        "",
        "PHOTO",
        "X-A",
    ];
    let params = [
        "",
        ";",
        ";;",
        ";TYPE=",
        ";TYPE=\"",
        ";TYPE=\"a:b",
        ";TYPE=\"a\";PREF=\"",
        ";TYPE=é,\"é;:\",",
        ";=",
        ";=;=,",
        ";PREF=-1;PREF=101;PREF=é",
        ";VALUE=text",
        ";VALUE=\"TEXT\";TYPE=pref,internet",
        ";WORK;é",
        ";TYPE=work;PREF=1;X-EVIL=\"a:b\"",
    ];
    let values = [
        "",
        ":",
        "\\",
        "é\\",
        "\\é",
        ";;;;;;;;;",
        ",,,",
        "\\;\\,\\\\\\n\\N\\x",
        "a@b:c",
        "http://u:p:q@[::1",
        "18151210",
        "1815121é",
        "é8151210",
        // Eight bytes, with a two-byte character across the fourth.
        "181é210",
        "1815-12-10",
        "1815-12-1é",
        "é815-12-10",
        "--1210",
        "ééééé",
        "éé-éé-éé",
        "\u{feff}\u{2028}\u{85}",
        "\"",
        "x\ty",
    ];
    let mut cards = 0;
    for head in heads {
        for param in params {
            for value in values {
                let vcf =
                    format!("BEGIN:VCARD\r\nVERSION:4.0\r\n{head}{param}:{value}\r\nEND:VCARD\r\n");
                let parsed = parse_vcard(vcf.as_bytes())
                    .unwrap_or_else(|e| panic!("a delimited card is not an error: {e}\n{vcf}"));
                for card in parsed {
                    let emitted = emit_vcard(&card.json).unwrap();
                    let again = one(&emitted);
                    assert_eq!(one(&emit_vcard(&again).unwrap()), again, "{vcf}");
                    cards += 1;
                }
            }
        }
    }
    assert_eq!(cards, heads.len() * params.len() * values.len());

    // The same values arriving through the projection instead of a document.
    for value in values.iter().chain(&params).chain(&heads) {
        let mut c = plain();
        for field in ["emails", "phones", "onlineServices", "addresses"] {
            c[field][0]["context"] = json!(value);
            c[field][0]["value"] = json!(value);
        }
        c["addresses"][0]["street"] = json!(value);
        c["anniversaries"] = json!([
            { "kind": "birthday", "date": value },
            { "kind": "anniversary", "date": value },
        ]);
        c["uid"] = json!(value);
        c["kind"] = json!(value);
        c["pgpKey"] = json!(value);
        c["members"] = json!([value]);
        c["vcardExtra"] = json!([value, format!("X-A{value}:{value}")]);
        let emitted = emit_vcard(&c).unwrap();
        let again = one(&emitted);
        assert_eq!(one(&emit_vcard(&again).unwrap()), again, "{value:?}");
    }
}
