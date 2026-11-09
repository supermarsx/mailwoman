use mw_server::settings_registry::{REGISTRY, Scopes, validate};
use serde_json::json;
#[test]
fn normal_search_defaults_to_current_folder_and_supports_user_admin_policy() {
    let def = REGISTRY.iter().find(|def| def.key == "mail.search.defaultScope").unwrap();
    assert!(matches!(def.scopes, Scopes::GlobalDomainUser));
    assert!(def.user_visible());
    assert_eq!(serde_json::from_str::<String>(def.default).unwrap(), "current");
    for value in ["current", "all"] { assert!(validate(def, &json!(value), "/value").is_ok()); }
    for value in [json!("folder"), json!(true), json!(null)] { assert!(validate(def, &value, "/value").is_err()); }
}
