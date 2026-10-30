use mw_server::settings_registry::{REGISTRY, Scopes, validate};
use serde_json::json;
#[test]
fn safe_html_modes_and_attachment_defaults_are_user_policy_backed() {
    let mode = REGISTRY
        .iter()
        .find(|d| d.key == "mail.reader.unsafeHtmlMode")
        .unwrap();
    assert_eq!(mode.default, "\"ask\"");
    for value in ["ask", "strict", "sanitized"] {
        assert!(validate(mode, &json!(value), "/value").is_ok());
    }
    for value in [json!("raw"), json!("unsafe"), json!(true)] {
        assert!(validate(mode, &value, "/value").is_err());
    }
    let density = REGISTRY
        .iter()
        .find(|d| d.key == "reader.attachmentsDensity")
        .unwrap();
    assert_eq!(density.default, "\"compact\"");
    assert!(validate(density, &json!("comfortable"), "/value").is_ok());
    assert!(validate(density, &json!("expanded"), "/value").is_err());
    let collapsed = REGISTRY
        .iter()
        .find(|d| d.key == "reader.attachmentsCollapsed")
        .unwrap();
    assert_eq!(collapsed.default, "true");
    for def in [mode, density, collapsed] {
        assert!(matches!(def.scopes, Scopes::GlobalDomainUser));
        assert!(def.user_visible());
    }
}
