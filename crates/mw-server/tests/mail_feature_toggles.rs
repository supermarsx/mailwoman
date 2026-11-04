use mw_server::settings_registry::{REGISTRY, Scopes, validate};
use serde_json::json;

const KEYS: [&str; 8] = [
    "mail.allowSend",
    "mail.allowReply",
    "mail.allowForward",
    "mail.allowDownload",
    "mail.allowPrint",
    "mail.showSpam",
    "mail.showAuthStatus",
    "mail.closeOnDelete",
];

/// Each mail feature switch is a global/domain/user boolean that is on until
/// an administrator or the user turns it off.
#[test]
fn mail_feature_switches_are_user_overridable_booleans_defaulting_on() {
    for key in KEYS {
        let def = REGISTRY
            .iter()
            .find(|d| d.key == key)
            .unwrap_or_else(|| panic!("{key} is registered"));
        assert!(matches!(def.scopes, Scopes::GlobalDomainUser), "{key}");
        assert!(def.user_override, "{key}");
        assert!(def.user_visible(), "{key} reaches GET /api/settings");
        assert_eq!(def.category, "mail", "{key}");
        assert_eq!(def.default, "true", "{key}");
        assert!(validate(def, &json!(true), "/value").is_ok(), "{key}");
        assert!(validate(def, &json!(false), "/value").is_ok(), "{key}");
        assert!(validate(def, &json!("off"), "/value").is_err(), "{key}");
        assert!(validate(def, &json!(0), "/value").is_err(), "{key}");
    }
}

/// The Go to top button is opt-in: a user-overridable boolean that is off until
/// an administrator or the user turns it on.
#[test]
fn go_to_top_is_a_user_overridable_boolean_defaulting_off() {
    let def = REGISTRY
        .iter()
        .find(|d| d.key == "mail.showGoToTop")
        .expect("mail.showGoToTop is registered");
    assert!(matches!(def.scopes, Scopes::GlobalDomainUser));
    assert!(def.user_override);
    assert!(def.user_visible());
    assert_eq!(def.category, "mail");
    assert_eq!(def.default, "false");
    assert!(validate(def, &json!(true), "/value").is_ok());
    assert!(validate(def, &json!(false), "/value").is_ok());
    assert!(validate(def, &json!("on"), "/value").is_err());
}

/// The Create folder button next to Compose is opt-in, like Go to top.
#[test]
fn create_folder_button_is_a_user_overridable_boolean_defaulting_off() {
    let def = REGISTRY
        .iter()
        .find(|d| d.key == "mail.showCreateFolderButton")
        .expect("mail.showCreateFolderButton is registered");
    assert!(matches!(def.scopes, Scopes::GlobalDomainUser));
    assert!(def.user_override);
    assert!(def.user_visible());
    assert_eq!(def.category, "mail");
    assert_eq!(def.default, "false");
    assert!(validate(def, &json!(true), "/value").is_ok());
    assert!(validate(def, &json!(false), "/value").is_ok());
    assert!(validate(def, &json!("on"), "/value").is_err());
}
