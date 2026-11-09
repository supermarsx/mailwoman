use mw_server::settings_registry::{REGISTRY, Scopes, validate};
use serde_json::json;
#[test]
fn interface_and_message_preferences_support_user_and_admin_policy() {
    for key in [
        "appearance.fontSize",
        "appearance.fontWeight",
        "appearance.fontColor",
        "appearance.lowContrast",
        "mail.messageListStyle",
        "mail.messageListOrder",
        "mail.messageListGroup",
        "mail.messageListPreview",
        "mail.afterDelete",
        "reader.attachmentsAtEnd",
    ] {
        let def = REGISTRY.iter().find(|d| d.key == key).unwrap();
        assert!(matches!(def.scopes, Scopes::GlobalDomainUser));
        assert!(def.user_visible());
        assert!(validate(def, &serde_json::from_str(def.default).unwrap(), "/value").is_ok());
    }
    let color = REGISTRY
        .iter()
        .find(|d| d.key == "appearance.fontColor")
        .unwrap();
    assert!(validate(color, &json!("#123abc"), "/value").is_ok());
    assert!(validate(color, &json!("red"), "/value").is_err());
    assert!(validate(color, &json!("#123abz"), "/value").is_err());
    let size = REGISTRY
        .iter()
        .find(|d| d.key == "appearance.fontSize")
        .unwrap();
    assert!(validate(size, &json!(25), "/value").is_err());
}

#[test]
fn empty_folder_visibility_defaults_to_off_and_supports_locked_user_policy() {
    let def = REGISTRY.iter().find(|d| d.key == "folders.hideEmpty").unwrap();
    assert!(matches!(def.scopes, Scopes::GlobalDomainUser));
    assert_eq!(def.default, "false");
    assert!(def.user_visible());
    assert!(validate(def, &json!(true), "/value").is_ok());
    assert!(validate(def, &json!(false), "/value").is_ok());
    assert!(validate(def, &json!("false"), "/value").is_err());
    let overrides = REGISTRY.iter().find(|d| d.key == "properties.folder.overrides.allowed").unwrap();
    let values: Vec<String> = serde_json::from_str(overrides.default).unwrap();
    assert!(values.iter().any(|key| key == "hideWhenEmpty"));
}

#[test]
fn all_reader_action_placements_preserve_defaults_and_allow_user_admin_policy() {
    let toolbar = ["reply", "reply-all", "forward", "edit-draft", "open-draft", "delete", "archive", "move", "conversation", "links", "read", "flag", "pin", "spam", "security", "headers", "sweep", "print", "export", "properties", "zoom", "search", "colors"];
    let menu = ["save-sender", "save-contacts", "focus-focused", "focus-other", "not-spam", "view-source", "create-filter", "block-sender", "finish-followup"];
    assert_eq!(REGISTRY.iter().filter(|d| d.key.starts_with("reader.actions.")).count(), toolbar.len() + menu.len());
    for (ids, placement) in [(toolbar.as_slice(), "toolbar"), (menu.as_slice(), "menu")] {
        for id in ids {
            let key = format!("reader.actions.{id}");
            let def = REGISTRY.iter().find(|d| d.key == key).unwrap();
            assert!(matches!(def.scopes, Scopes::GlobalDomainUser));
            assert!(def.user_visible());
            assert_eq!(serde_json::from_str::<String>(def.default).unwrap(), placement);
            for value in ["toolbar", "menu", "hidden"] {
                assert!(validate(def, &json!(value), "/value").is_ok(), "{key}: {value}");
            }
            assert!(validate(def, &json!(true), "/value").is_err());
            assert!(validate(def, &json!("nowhere"), "/value").is_err());
        }
    }
}
