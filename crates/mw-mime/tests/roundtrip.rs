//! Build → parse round-trip and stability tests (`mail-builder` ↔ `mail-parser`).

use mw_mime::{ComposeRequest, EmailAddress, build, parse};

fn addr(name: Option<&str>, email: &str) -> EmailAddress {
    EmailAddress {
        name: name.map(str::to_string),
        email: email.to_string(),
    }
}

fn sample_request() -> ComposeRequest {
    ComposeRequest {
        from: Some(addr(Some("Alice Example"), "alice@example.org")),
        to: vec![
            addr(Some("Bob"), "bob@example.net"),
            addr(None, "carol@example.org"),
        ],
        cc: vec![addr(None, "dave@example.org")],
        subject: Some("Round trip café ✅".to_string()),
        text_body: Some("Plain body with an é.".to_string()),
        html_body: Some("<p>HTML body with an é.</p>".to_string()),
        message_id: Some("compose-1@example.org".to_string()),
        in_reply_to: Some("parent-1@example.org".to_string()),
        references: vec![
            "root-0@example.org".to_string(),
            "parent-1@example.org".to_string(),
        ],
        ..ComposeRequest::default()
    }
}

#[test]
fn build_then_parse_preserves_fields() {
    let req = sample_request();
    let bytes = build(&req).unwrap();
    let p = parse(&bytes).unwrap();
    let e = &p.email;

    assert_eq!(e.subject, req.subject);
    assert_eq!(e.from.as_ref().unwrap()[0].email, "alice@example.org");
    assert_eq!(
        e.from.as_ref().unwrap()[0].name.as_deref(),
        Some("Alice Example")
    );
    let to = e.to.as_ref().unwrap();
    assert_eq!(to.len(), 2);
    assert_eq!(to[1].email, "carol@example.org");
    assert_eq!(e.cc.as_ref().unwrap()[0].email, "dave@example.org");

    // Both bodies survive as text/plain + text/html alternatives.
    assert!(
        e.text_body
            .iter()
            .any(|p| p.r#type.as_deref() == Some("text/plain"))
    );
    assert!(
        e.html_body
            .iter()
            .any(|p| p.r#type.as_deref() == Some("text/html"))
    );
    let bodies: String = e.body_values.values().map(|b| b.value.as_str()).collect();
    assert!(bodies.contains("Plain body with an é."));
    assert!(bodies.contains("HTML body with an é."));

    // Threading headers survive with angle brackets stripped on the way back.
    assert_eq!(
        p.envelope.message_id.as_deref(),
        Some("compose-1@example.org")
    );
    assert_eq!(
        p.envelope.in_reply_to.as_deref(),
        Some("parent-1@example.org")
    );
    assert_eq!(
        p.envelope.references,
        vec![
            "root-0@example.org".to_string(),
            "parent-1@example.org".to_string()
        ]
    );
}

#[test]
fn ids_are_bracket_idempotent() {
    // Ids supplied WITH angle brackets must not double-wrap.
    let req = ComposeRequest {
        from: Some(addr(None, "a@example.org")),
        to: vec![addr(None, "b@example.org")],
        message_id: Some("<already-bracketed@example.org>".to_string()),
        references: vec!["<r1@example.org>".to_string()],
        text_body: Some("hi".to_string()),
        ..ComposeRequest::default()
    };
    let bytes = build(&req).unwrap();
    let p = parse(&bytes).unwrap();
    assert_eq!(
        p.envelope.message_id.as_deref(),
        Some("already-bracketed@example.org")
    );
    assert_eq!(p.envelope.references, vec!["r1@example.org".to_string()]);
    // No literal double brackets in the wire bytes.
    let wire = String::from_utf8_lossy(&bytes);
    assert!(!wire.contains("<<"));
    assert!(!wire.contains(">>"));
}

#[test]
fn build_is_stable_across_a_second_round() {
    // build → parse → recompose from the parse → build → parse: content stable.
    let req = sample_request();
    let first = parse(&build(&req).unwrap()).unwrap();

    let recomposed = ComposeRequest {
        from: first.email.from.as_ref().map(|l| l[0].clone()),
        to: first.email.to.clone().unwrap_or_default(),
        cc: first.email.cc.clone().unwrap_or_default(),
        subject: first.email.subject.clone(),
        text_body: Some("Plain body with an é.".to_string()),
        html_body: Some("<p>HTML body with an é.</p>".to_string()),
        message_id: first.envelope.message_id.clone(),
        in_reply_to: first.envelope.in_reply_to.clone(),
        references: first.envelope.references.clone(),
        ..ComposeRequest::default()
    };
    let second = parse(&build(&recomposed).unwrap()).unwrap();

    assert_eq!(first.email.subject, second.email.subject);
    assert_eq!(first.envelope, second.envelope);
    assert_eq!(
        first.email.to.map(|l| l.len()),
        second.email.to.map(|l| l.len())
    );
}

#[test]
fn build_minimal_message_without_body() {
    let req = ComposeRequest {
        from: Some(addr(None, "noreply@example.org")),
        to: vec![addr(None, "someone@example.org")],
        subject: Some("ping".to_string()),
        ..ComposeRequest::default()
    };
    let bytes = build(&req).unwrap();
    let p = parse(&bytes).unwrap();
    assert_eq!(p.email.subject.as_deref(), Some("ping"));
    assert_eq!(p.email.from.unwrap()[0].email, "noreply@example.org");
}

#[test]
fn build_custom_header_is_emitted_verbatim() {
    let req = ComposeRequest {
        from: Some(addr(None, "a@example.org")),
        to: vec![addr(None, "b@example.org")],
        text_body: Some("x".to_string()),
        headers: vec![("User-Agent".to_string(), "Mailwoman/26.2".to_string())],
        ..ComposeRequest::default()
    };
    let bytes = build(&req).unwrap();
    let wire = String::from_utf8_lossy(&bytes);
    assert!(wire.contains("User-Agent: Mailwoman/26.2"));
}
