//! 26.20 t28-e11b — `messageId`, `inReplyTo` and `references` on the parsed
//! `Email` (RFC 8621 §4.1.2.3): the ids a header lists, without their angle
//! brackets, or `None` when the header is absent or is not a list of ids.

use mw_jmap::Email;

/// A message whose header block is `headers` (each line CRLF-terminated by the
/// caller) followed by a one-line body.
fn email(headers: &str) -> Email {
    let raw = format!("From: a@example.org\r\nSubject: s\r\n{headers}\r\nbody\r\n");
    mw_mime::parse(raw.as_bytes())
        .expect("a message with a From header parses")
        .email
}

fn ids(list: &[&str]) -> Option<Vec<String>> {
    Some(list.iter().map(|s| (*s).to_string()).collect())
}

#[test]
fn a_single_id_loses_its_brackets() {
    let e = email("Message-ID: <one@example.org>\r\n");
    assert_eq!(e.message_id, ids(&["one@example.org"]));
    assert_eq!(e.in_reply_to, None);
    assert_eq!(e.references, None);
}

#[test]
fn each_header_feeds_its_own_property() {
    let e = email(concat!(
        "Message-ID: <m@x>\r\n",
        "In-Reply-To: <p@x>\r\n",
        "References: <r1@x> <r2@x>\r\n",
    ));
    assert_eq!(e.message_id, ids(&["m@x"]));
    assert_eq!(e.in_reply_to, ids(&["p@x"]));
    assert_eq!(e.references, ids(&["r1@x", "r2@x"]));
}

#[test]
fn several_ids_across_folded_lines_keep_their_order() {
    let e = email(concat!(
        "References: <a@x>\r\n",
        " <b@x>\r\n",
        "\t<c@x><d@x>\r\n",
        "In-Reply-To:<p1@x>\r\n <p2@x>\r\n",
    ));
    assert_eq!(e.references, ids(&["a@x", "b@x", "c@x", "d@x"]));
    assert_eq!(e.in_reply_to, ids(&["p1@x", "p2@x"]));
}

#[test]
fn comments_commas_and_odd_whitespace_between_ids_are_skipped() {
    let e = email(concat!(
        "References:   (first (nested \\) one)) <a@x>,\t <b@x>  (last)  \r\n",
        "Message-ID:\t  <m@x>   \r\n",
    ));
    assert_eq!(e.references, ids(&["a@x", "b@x"]));
    assert_eq!(e.message_id, ids(&["m@x"]));
}

#[test]
fn an_id_without_an_at_sign_and_a_non_ascii_id_are_kept() {
    let e = email("Message-ID: <20261005.abc>\r\nIn-Reply-To: <grüße@exämple.org>\r\n");
    assert_eq!(e.message_id, ids(&["20261005.abc"]));
    assert_eq!(e.in_reply_to, ids(&["grüße@exämple.org"]));
}

#[test]
fn a_missing_header_is_none() {
    let e = email("");
    assert_eq!(e.message_id, None);
    assert_eq!(e.in_reply_to, None);
    assert_eq!(e.references, None);
}

#[test]
fn a_malformed_value_is_none_not_text() {
    for value in [
        "one@example.org",               // no brackets
        "<a@x> trailing words",          // text after an id
        "Your message of Tuesday <a@x>", // obsolete phrase before the id
        "<a@x",                          // unclosed id
        "a@x>",                          // unopened id
        "<>",                            // empty id
        "<a@x> <>",                      // an empty id beside a good one
        "<a <b@x>",                      // `<` inside an id
        "<a b@x>",                       // space inside an id
        "(unclosed <a@x>",               // unclosed comment
        "(only a comment)",              // no id at all
        "",                              // empty value
        "<a@x>; <b@x>",                  // a separator that is not one
    ] {
        let e = email(&format!("References: {value}\r\nMessage-ID: {value}\r\n"));
        assert_eq!(e.references, None, "References: {value:?}");
        assert_eq!(e.message_id, None, "Message-ID: {value:?}");
    }
}

#[test]
fn one_bad_id_refuses_the_whole_list() {
    let e = email("References: <a@x> <b b@x> <c@x>\r\n");
    assert_eq!(e.references, None);
}

#[test]
fn an_id_folded_across_two_lines_is_refused() {
    let e = email("Message-ID: <left\r\n @right>\r\n");
    assert_eq!(e.message_id, None);
}

#[test]
fn control_characters_never_reach_a_value() {
    for hostile in [
        "\u{0}", "\u{1}", "\u{7}", "\u{8}", "\u{b}", "\u{c}", "\u{1b}", "\u{7f}", "\u{85}",
        "\u{9b}", "\t", "\r",
    ] {
        let blank = hostile == "\t" || hostile == "\r";
        // Inside an id, at the end of one, and between two ids.
        for (inside, value) in [
            (true, format!("<a{hostile}b@x>")),
            (true, format!("<a@x> <b@x{hostile}>")),
            (false, format!("<a@x>{hostile}<b@x>")),
        ] {
            let e = email(&format!("References: {value}\r\nIn-Reply-To: {value}\r\n"));
            for got in [&e.references, &e.in_reply_to] {
                for id in got.iter().flatten() {
                    assert!(
                        !id.chars().any(char::is_control),
                        "{value:?} produced {id:?}"
                    );
                }
                if inside || !blank {
                    // Refused outright, not repaired by dropping the character.
                    assert_eq!(*got, None, "{value:?}");
                } else {
                    // A tab or a bare CR between two ids is only white space.
                    assert_eq!(*got, ids(&["a@x", "b@x"]), "{value:?}");
                }
            }
        }
    }
}

#[test]
fn a_header_injected_through_an_id_adds_no_id() {
    // A bare line feed inside an id: whatever the header parser makes of the
    // line break, no id holding one comes out and no id is invented from the
    // injected line.
    let e = email("References: <a@x\nBcc: victim@example.org>\r\n");
    for id in e.references.iter().flatten() {
        assert!(!id.contains('\n') && !id.contains("victim"), "{id:?}");
    }
    assert_eq!(e.references, None);
}

#[test]
fn the_last_instance_of_a_repeated_header_is_the_one_read() {
    let e = email("Message-ID: <first@x>\r\nMessage-ID: <last@x>\r\n");
    assert_eq!(e.message_id, ids(&["last@x"]));
    // The last instance decides even when it is the malformed one.
    let e = email("Message-ID: <first@x>\r\nMessage-ID: not an id\r\n");
    assert_eq!(e.message_id, None);
}

#[test]
fn a_header_that_is_not_utf8_is_none() {
    let mut raw = b"From: a@example.org\r\nMessage-ID: <a".to_vec();
    raw.extend_from_slice(&[0xff, 0xfe]);
    raw.extend_from_slice(b"@x>\r\nReferences: <ok@x>\r\n\r\nbody\r\n");
    let e = mw_mime::parse(&raw).expect("parses").email;
    assert_eq!(e.message_id, None);
    assert_eq!(e.references, ids(&["ok@x"]));
}

#[test]
fn header_names_match_without_regard_to_case() {
    let e = email("MESSAGE-id: <m@x>\r\nin-reply-to: <p@x>\r\nREFERENCES: <r@x>\r\n");
    assert_eq!(e.message_id, ids(&["m@x"]));
    assert_eq!(e.in_reply_to, ids(&["p@x"]));
    assert_eq!(e.references, ids(&["r@x"]));
}

#[test]
fn the_threading_envelope_is_unchanged_by_the_new_properties() {
    let raw = concat!(
        "From: a@example.org\r\n",
        "Message-ID: <m@x>\r\n",
        "In-Reply-To: <p@x>\r\n",
        "References: <r1@x> <p@x>\r\n",
        "\r\nbody\r\n",
    );
    let p = mw_mime::parse(raw.as_bytes()).expect("parses");
    assert_eq!(p.envelope.message_id.as_deref(), Some("m@x"));
    assert_eq!(p.envelope.in_reply_to.as_deref(), Some("p@x"));
    assert_eq!(p.envelope.references, vec!["r1@x", "p@x"]);
}
