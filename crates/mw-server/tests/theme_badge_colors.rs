use mw_server::settings_registry::{REGISTRY, Scopes, validate};
use serde_json::json;

#[test]
fn warm_and_neutral_badge_accents_are_admitted_by_user_and_admin_policy() {
    let badge = REGISTRY
        .iter()
        .find(|setting| setting.key == "appearance.badgeColor")
        .unwrap();
    assert!(matches!(badge.scopes, Scopes::GlobalDomainUser));
    assert!(badge.user_visible());
    for accent in ["gold", "brown", "beige", "taupe", "silver", "stone"] {
        assert_eq!(
            validate(badge, &json!(accent), "/value").unwrap(),
            json!(accent)
        );
    }
    // Existing badges and derived accent choices remain valid.
    for accent in [
        "default",
        "accent",
        "accent-lighter",
        "accent-darker",
        "red",
        "graphite",
    ] {
        assert!(validate(badge, &json!(accent), "/value").is_ok());
    }
    for invalid in [
        json!("sepia"),
        json!("cyberpunk"),
        json!("#d4af37"),
        json!(""),
        json!(true),
        json!(null),
    ] {
        assert!(
            validate(badge, &invalid, "/value").is_err(),
            "{invalid} must not be accepted as a named badge accent"
        );
    }
}
