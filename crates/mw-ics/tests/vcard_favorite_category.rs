//! `CATEGORIES:favorite` is the interoperable favourite flag of a contact.

use mw_ics::{emit_vcard, parse_vcard};
use serde_json::{Value, json};

fn first(text: &str) -> Value {
    parse_vcard(text.as_bytes()).unwrap().remove(0).json
}

#[test]
fn favorite_category_is_read_and_other_categories_are_kept() {
    let card = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Ada\r\nCATEGORIES:Board,Favorite\r\nEND:VCARD\r\n";
    let json = first(card);
    assert_eq!(json["isFavorite"], json!(true));
    assert_eq!(json["vcardExtra"], json!(["CATEGORIES:Board"]));
    let out = emit_vcard(&json).unwrap();
    assert!(out.contains("CATEGORIES:Board,favorite\r\n"), "{out}");
    assert_eq!(first(&out), json);
}

#[test]
fn clearing_the_flag_drops_only_the_favorite_token() {
    let card = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Ada\r\nCATEGORIES:Board,favorite\r\nEND:VCARD\r\n";
    let mut json = first(card);
    json["isFavorite"] = json!(false);
    let out = emit_vcard(&json).unwrap();
    assert!(out.contains("CATEGORIES:Board\r\n"), "{out}");
    assert!(!out.to_lowercase().contains("favorite"), "{out}");
    assert_eq!(first(&out)["isFavorite"], json!(false));
}

#[test]
fn a_favorite_without_other_categories_gets_its_own_line() {
    let mut json = first("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Ada\r\nEND:VCARD\r\n");
    assert_eq!(json["isFavorite"], json!(false));
    assert!(!emit_vcard(&json).unwrap().contains("CATEGORIES"));
    json["isFavorite"] = json!(true);
    let out = emit_vcard(&json).unwrap();
    assert!(out.contains("CATEGORIES:favorite\r\n"), "{out}");
    assert_eq!(first(&out)["isFavorite"], json!(true));
}
