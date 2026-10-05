//! 26.20 t28-e11b — the wire shape of `messageId`, `inReplyTo` and
//! `references` on `Email` (RFC 8621 §4.1.2.3: `String[]|null`, ids without
//! angle brackets).

use mw_jmap::Email;
use serde_json::{Value, json};

#[test]
fn ids_serialise_as_camel_case_string_arrays() {
    let email = Email {
        message_id: Some(vec!["m@x".into()]),
        in_reply_to: Some(vec!["p@x".into()]),
        references: Some(vec!["r1@x".into(), "p@x".into()]),
        ..Email::default()
    };
    let v = serde_json::to_value(&email).unwrap();
    assert_eq!(v["messageId"], json!(["m@x"]));
    assert_eq!(v["inReplyTo"], json!(["p@x"]));
    assert_eq!(v["references"], json!(["r1@x", "p@x"]));
    // No snake_case twin of the two renamed keys.
    let obj = v.as_object().unwrap();
    assert!(!obj.contains_key("message_id"));
    assert!(!obj.contains_key("in_reply_to"));
}

#[test]
fn an_absent_header_serialises_as_null_not_as_a_missing_key() {
    let v = serde_json::to_value(Email::default()).unwrap();
    let obj = v.as_object().unwrap();
    for key in ["messageId", "inReplyTo", "references"] {
        assert_eq!(obj.get(key), Some(&Value::Null), "{key}");
    }
}

#[test]
fn an_object_without_the_keys_still_deserialises() {
    // What the engine stored for a message before the properties existed.
    let old = json!({ "id": "e1", "subject": "s", "size": 3 });
    let email: Email = serde_json::from_value(old).unwrap();
    assert_eq!(email.subject.as_deref(), Some("s"));
    assert_eq!(email.message_id, None);
    assert_eq!(email.in_reply_to, None);
    assert_eq!(email.references, None);
}

#[test]
fn the_wire_shape_round_trips() {
    let wire = json!({
        "id": "e1",
        "messageId": ["m@x"],
        "inReplyTo": null,
        "references": ["r1@x", "r2@x"]
    });
    let email: Email = serde_json::from_value(wire).unwrap();
    assert_eq!(email.message_id, Some(vec!["m@x".to_string()]));
    assert_eq!(email.in_reply_to, None);
    assert_eq!(
        email.references,
        Some(vec!["r1@x".to_string(), "r2@x".to_string()])
    );
    let back = serde_json::to_value(&email).unwrap();
    assert_eq!(back["messageId"], json!(["m@x"]));
    assert_eq!(back["inReplyTo"], Value::Null);
    assert_eq!(back["references"], json!(["r1@x", "r2@x"]));
}
