//! t32-e37 PGP/MIME (RFC 3156), header protection (RFC 9788) and the MIME helpers
//! both PGP and S/MIME read signed parts with.

use mw_crypto::mime::{self, CryptoPart, InlineSegment, Protection};
use mw_crypto::pgp::{self, pgpmime};
use mw_crypto::verdict;

const PW: &str = "pw-mime";

fn key(uid: &str) -> pgp::GeneratedKey {
    pgp::generate_key(uid, PW).expect("keygen")
}

fn headers() -> Vec<(String, String)> {
    vec![
        ("From".into(), "Ann <ann@example.com>".into()),
        ("To".into(), "Bob <bob@example.com>".into()),
        ("Subject".into(), "Quarterly numbers".into()),
        ("Date".into(), "Sun, 25 Oct 2026 10:00:00 +0000".into()),
        ("Message-ID".into(), "<m1@example.com>".into()),
        ("X-Mailer".into(), "test".into()),
    ]
}

const PLAIN_PAYLOAD: &[u8] =
    b"Content-Type: text/plain; charset=utf-8\r\n\r\nThe numbers are up.\r\n";

// ── helpers ──────────────────────────────────────────────────────────────────

#[test]
fn split_multipart_keeps_exact_part_bytes() {
    let body = b"preamble\r\n--b\r\nContent-Type: text/plain\r\n\r\none\r\n\r\n--b\r\n\r\ntwo\r\n--b--\r\nepilogue";
    let parts = mime::split_multipart(body, "b").expect("parts");
    assert_eq!(parts.len(), 2);
    // The CRLF before a delimiter belongs to the delimiter: "one\r\n" keeps only
    // its own line break.
    assert_eq!(parts[0], b"Content-Type: text/plain\r\n\r\none\r\n");
    assert_eq!(parts[1], b"\r\ntwo");
    assert!(mime::split_multipart(b"--b\r\nno close\r\n", "b").is_none());
    // A boundary string inside a line is not a delimiter.
    let tricky = b"--b\r\nx --b y\r\n--b--\r\n";
    assert_eq!(
        mime::split_multipart(tricky, "b").unwrap(),
        vec![&b"x --b y"[..]]
    );
}

#[test]
fn canonicalize_and_quoted_printable() {
    assert_eq!(mime::canonicalize(b"a\nb\r\nc\rd"), b"a\r\nb\r\nc\r\nd");
    let qp = mime::quoted_printable("From here \nline=end\t\n-dash");
    assert!(qp.starts_with("=46rom here=20\r\n"), "{qp}");
    assert!(qp.contains("line=3Dend=09\r\n"), "{qp}");
    assert!(qp.ends_with("=2Ddash"), "{qp}");
    let long = "x".repeat(200);
    assert!(
        mime::quoted_printable(&long)
            .split("\r\n")
            .all(|l| l.len() <= 76)
    );
}

#[test]
fn content_type_parameters() {
    let ct = mime::ContentType::parse(
        "multipart/signed; micalg=pgp-sha512; protocol=\"application/pgp-signature\"; boundary=\"a;b\\\"c\"",
    );
    assert_eq!(ct.mime, "multipart/signed");
    assert_eq!(ct.param("micalg"), Some("pgp-sha512"));
    assert_eq!(ct.param("protocol"), Some("application/pgp-signature"));
    assert_eq!(ct.param("boundary"), Some("a;b\"c"));
}

#[test]
fn fold_header_lines_stay_short() {
    let long = format!("addr=a@b.example; keydata={}", "A".repeat(500));
    let folded = mime::fold_header("Autocrypt", &long);
    assert!(folded.split("\r\n").all(|l| l.len() <= 78), "{folded}");
    let unfolded: String = mime::header_values(&format!("{folded}\r\n"), "autocrypt").concat();
    assert_eq!(
        unfolded.split_whitespace().collect::<String>(),
        long.split_whitespace().collect::<String>()
    );
}

// ── header protection ────────────────────────────────────────────────────────

#[test]
fn header_protection_round_trip_plain_text() {
    let outer = vec![("Subject".to_string(), "[...]".to_string())];
    let protected =
        mime::protect_headers(PLAIN_PAYLOAD, &headers(), &outer, Protection::Cipher, true);
    let text = String::from_utf8_lossy(&protected);
    assert!(text.contains("Subject: Quarterly numbers\r\n"));
    assert!(text.contains("HP-Outer: Subject: [...]\r\n"));
    assert!(text.contains("hp=\"cipher\""));
    assert!(text.contains("hp-legacy-display=\"1\""));
    assert!(
        !text.contains("X-Mailer"),
        "non-protected fields stay outside"
    );

    let back = mime::unprotect_headers(&protected).expect("protected");
    assert_eq!(back.scheme, "cipher");
    assert!(
        back.headers
            .contains(&("Subject".into(), "Quarterly numbers".into()))
    );
    assert_eq!(back.outer, vec![("Subject".into(), "[...]".into())]);
    let rendered = &back.entity;
    assert!(
        rendered.ends_with("\r\n\r\nThe numbers are up.\r\n"),
        "{rendered}"
    );
    assert!(
        !rendered.contains("Subject: Quarterly numbers\r\nFrom"),
        "legacy display removed"
    );
}

#[test]
fn header_protection_wraps_non_plain_payloads() {
    let payload = b"Content-Type: multipart/alternative; boundary=\"alt\"\r\n\r\n--alt\r\nContent-Type: text/plain\r\n\r\nhi\r\n--alt\r\nContent-Type: text/html\r\n\r\n<p>hi</p>\r\n--alt--\r\n";
    let protected = mime::protect_headers(payload, &headers(), &[], Protection::Cipher, true);
    let back = mime::unprotect_headers(&protected).expect("protected");
    let rendered = &back.entity;
    assert!(rendered.contains("multipart/alternative"), "{rendered}");
    assert!(rendered.contains("<p>hi</p>"));
    assert!(!rendered.contains("hp-legacy-display"));
}

#[test]
fn older_protected_headers_markings_are_read() {
    let v1 =
        b"Content-Type: text/plain; protected-headers=\"v1\"\r\nSubject: Old style\r\n\r\nbody";
    let back = mime::unprotect_headers(v1).expect("v1");
    assert_eq!(back.scheme, "v1");
    assert_eq!(back.headers, vec![("Subject".into(), "Old style".into())]);

    let ours = pgp::wrap_protected_headers("Stanza subject", b"stanza body");
    let back = mime::unprotect_headers(&ours).expect("stanza");
    assert_eq!(back.scheme, "v1");
    assert_eq!(
        back.headers,
        vec![("Subject".into(), "Stanza subject".into())]
    );
    assert!(back.entity.ends_with("\r\n\r\nstanza body"));
    assert!(mime::unprotect_headers(PLAIN_PAYLOAD).is_none());
}

// ── PGP/MIME send and receive ────────────────────────────────────────────────

#[test]
fn encrypted_round_trip_with_header_protection_and_gossip() {
    let ann = key("Ann <ann@example.com>");
    let bob = key("Bob <bob@example.com>");
    let carl = key("Carl <carl@example.com>");
    let payload = b"Content-Type: multipart/mixed; boundary=\"mix\"\r\n\r\n--mix\r\nContent-Type: text/plain\r\n\r\nsee attached\r\n--mix\r\nContent-Type: application/octet-stream; name=\"a.bin\"\r\nContent-Transfer-Encoding: base64\r\n\r\nAAECAw==\r\n--mix--\r\n";
    let built = pgpmime::encrypt_mime(
        payload,
        &headers(),
        &[
            ann.public_key_armored.clone(),
            bob.public_key_armored.clone(),
            carl.public_key_armored.clone(),
        ],
        Some((&ann.encrypted_private_bundle, PW)),
        &[
            ("bob@example.com".into(), bob.public_key_armored.clone()),
            ("carl@example.com".into(), carl.public_key_armored.clone()),
        ],
    )
    .expect("encrypt");
    assert_eq!(built.outer_subject.as_deref(), Some("[...]"));
    assert_eq!(built.seipd_version, Some(2));
    assert!(
        built
            .content_type
            .starts_with("multipart/encrypted; protocol=\"application/pgp-encrypted\"")
    );
    assert!(!built.entity.contains("Quarterly"), "subject leaked");
    assert!(!built.entity.contains("AAECAw"), "attachment leaked");

    let analysis = mime::analyze(built.entity.as_bytes());
    assert!(matches!(
        analysis.part,
        Some(CryptoPart::PgpEncrypted { .. })
    ));
    assert!(!analysis.partial);

    let opened = pgpmime::open_mime(
        built.entity.as_bytes(),
        &bob.encrypted_private_bundle,
        PW,
        &[&ann.public_key_armored],
    )
    .expect("open");
    assert!(opened.encrypted && opened.aead);
    assert_eq!(opened.verification.state, verdict::VALID);
    let protected = opened.protected.expect("protected headers");
    assert_eq!(protected.scheme, "cipher");
    assert!(
        protected
            .headers
            .contains(&("Subject".into(), "Quarterly numbers".into()))
    );
    assert!(opened.entity.contains("see attached"));
    assert!(opened.entity.contains("AAECAw=="));
    assert_eq!(opened.gossip.len(), 2);
    assert!(opened.gossip.iter().any(|g| g.addr == "carl@example.com"));
}

#[test]
fn signed_round_trip_tamper_and_partial() {
    let ann = key("Ann <ann@example.com>");
    let built = pgpmime::sign_mime(PLAIN_PAYLOAD, &headers(), &ann.encrypted_private_bundle, PW)
        .expect("sign");
    assert!(built.content_type.contains("micalg=pgp-sha512"));
    assert_eq!(built.outer_subject.as_deref(), Some("Quarterly numbers"));

    let ok = pgpmime::verify_mime(built.entity.as_bytes(), &[&ann.public_key_armored]).unwrap();
    assert_eq!(ok.verification.state, verdict::VALID);
    assert!(!ok.partial);
    assert_eq!(
        ok.protected.as_ref().map(|p| p.scheme.as_str()),
        Some("clear")
    );

    // Line endings changed in transport (LF only) still verify: the signed part is
    // canonicalized before checking.
    let lf = built.entity.replace("\r\n", "\n");
    let ok_lf = pgpmime::verify_mime(lf.as_bytes(), &[&ann.public_key_armored]).unwrap();
    assert_eq!(ok_lf.verification.state, verdict::VALID);

    let tampered = built
        .entity
        .replace("The numbers are up.", "The numbers are down.");
    let bad = pgpmime::verify_mime(tampered.as_bytes(), &[&ann.public_key_armored]).unwrap();
    assert_eq!(bad.verification.state, verdict::BAD_SIGNATURE);

    // A list server appends an unsigned footer part around the signed entity.
    let wrapped = format!(
        "Content-Type: multipart/mixed; boundary=\"outer\"\r\n\r\n--outer\r\n{}\r\n--outer\r\nContent-Type: text/plain\r\n\r\nUnsubscribe: https://lists.example\r\n--outer--\r\n",
        built.entity
    );
    let partial = pgpmime::verify_mime(wrapped.as_bytes(), &[&ann.public_key_armored]).unwrap();
    assert!(partial.partial);
    assert_eq!(partial.verification.state, verdict::VALID);
    assert!(
        !partial.entity.contains("Unsubscribe"),
        "only the signed part is returned"
    );
}

#[test]
fn inline_pgp_is_read() {
    let ann = key("Ann <ann@example.com>");
    let bob = key("Bob <bob@example.com>");
    let signed = pgp::clear_sign("inline signed text", &ann.encrypted_private_bundle, PW).unwrap();
    let raw = format!("Content-Type: text/plain\r\n\r\nHello,\r\n{signed}\r\n-- \r\nfooter\r\n");
    let analysis = mime::analyze(raw.as_bytes());
    let Some(CryptoPart::InlinePgp { segments }) = &analysis.part else {
        panic!("no inline part: {analysis:?}");
    };
    assert!(analysis.partial, "text around the armor is unsigned");
    assert!(matches!(segments[0], InlineSegment::Text { .. }));
    assert!(matches!(segments[1], InlineSegment::Signed { .. }));
    let v = pgpmime::verify_mime(raw.as_bytes(), &[&ann.public_key_armored]).unwrap();
    assert_eq!(v.verification.state, verdict::VALID);
    assert!(v.partial);

    let ct = pgp::encrypt(
        b"inline secret",
        std::slice::from_ref(&bob.public_key_armored),
        None,
    )
    .unwrap();
    let raw = format!("Content-Type: text/plain\r\n\r\n{ct}\r\n");
    let opened =
        pgpmime::open_mime(raw.as_bytes(), &bob.encrypted_private_bundle, PW, &[]).unwrap();
    assert_eq!(opened.entity, "inline secret");
    assert_eq!(opened.verification.state, verdict::NOT_SIGNED);
}

#[test]
fn smime_structures_are_recognised() {
    let der = [0x30u8, 0x03, 0x02, 0x01, 0x01];
    let opaque = mime::smime_opaque_entity(&der, "signed-data");
    let analysis = mime::analyze(&opaque.to_bytes());
    assert_eq!(
        analysis.part,
        Some(CryptoPart::SmimeOpaque {
            der: der.to_vec(),
            smime_type: "signed-data".into()
        })
    );
    let signed = mime::smime_signed_entity(PLAIN_PAYLOAD, &der, &mime::smime_micalg("SHA384"));
    let bytes = signed.to_bytes();
    assert!(String::from_utf8_lossy(&bytes).contains("micalg=sha-384"));
    match mime::analyze(&bytes).part {
        Some(CryptoPart::SmimeSigned {
            signed,
            signature,
            micalg,
        }) => {
            assert_eq!(signed, PLAIN_PAYLOAD);
            assert_eq!(signature, der);
            assert_eq!(micalg.as_deref(), Some("sha-384"));
        }
        other => panic!("{other:?}"),
    }
}
