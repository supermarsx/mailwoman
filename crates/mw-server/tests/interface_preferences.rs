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
