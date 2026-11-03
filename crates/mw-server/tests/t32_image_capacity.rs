// Exercise the actual hot-resizable limiter without code-generating the whole
// server unit-test binary for this small concurrency regression suite.
#[allow(dead_code)]
#[path = "../src/remote_image_adaptive.rs"]
mod adaptive;
#[path = "../src/remote_image_capacity.rs"]
#[allow(dead_code)]
mod capacity;
#[allow(dead_code)]
#[path = "../src/remote_image_demand.rs"]
mod demand;
#[allow(dead_code)]
#[path = "../src/remote_image_resources.rs"]
mod resources;
#[allow(dead_code)]
#[path = "../src/remote_image_slots.rs"]
mod slots;

#[test]
fn proxy_controls_have_validated_defaults_and_reject_unbounded_values() {
    use mw_server::settings_registry::{lookup, validate};
    use serde_json::json;
    for (key, min, max, default, user_visible) in [
        ("privacy.maxConcurrentGlobal", 1, 256, 128, false),
        ("privacy.maxConcurrentPipeline", 1, 256, 128, false),
        ("privacy.maxConcurrentDecodes", 1, 16, 10, false),
        ("privacy.maxQueuedRequests", 16, 16384, 2048, false),
        ("privacy.imageDecodeTimeoutMs", 1000, 60000, 10000, false),
        ("privacy.rateBurst", 1, 12000, 512, false),
        ("privacy.cacheTtlSecs", 0, 86400, 3600, false),
        ("privacy.negativeCacheTtlSecs", 0, 300, 10, false),
        ("privacy.imageRetryAttempts", 0, 8, 3, true),
        ("privacy.imageRetryDelayMs", 500, 60000, 5000, true),
    ] {
        let definition = lookup(key).expect(key);
        assert_eq!(definition.default_value(), json!(default), "{key}");
        assert_eq!(definition.user_visible(), user_visible, "{key}");
        for valid in [min, default, max] {
            assert!(
                validate(definition, &json!(valid), "/value").is_ok(),
                "{key}"
            );
        }
        for invalid in [
            json!(min - 1),
            json!(max + 1),
            json!("10"),
            json!(1.5),
            json!(null),
        ] {
            assert!(
                validate(definition, &invalid, "/value").is_err(),
                "{key}={invalid}"
            );
        }
    }
}

#[test]
fn automatic_concurrency_is_the_validated_global_default() {
    use mw_server::settings_registry::{lookup, validate};
    use serde_json::json;
    let setting = lookup("privacy.imageConcurrencyMode").unwrap();
    assert_eq!(setting.default_value(), json!("auto"));
    assert!(!setting.user_visible());
    for mode in ["auto", "conservative", "balanced", "throughput", "manual"] {
        assert!(validate(setting, &json!(mode), "/value").is_ok());
    }
    assert!(validate(setting, &json!("unbounded"), "/value").is_err());
    assert_eq!(
        lookup("privacy.maxConcurrentPerMessage")
            .unwrap()
            .default_value(),
        json!(30)
    );
}
