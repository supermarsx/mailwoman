// Exercise the actual hot-resizable limiter without code-generating the whole
// server unit-test binary for this small concurrency regression suite.
#[path = "../src/remote_image_capacity.rs"]
mod capacity;

#[test]
fn proxy_controls_have_validated_defaults_and_reject_unbounded_values() {
    use mw_server::settings_registry::{lookup, validate};
    use serde_json::json;
    for (key, min, max, default, user_visible) in [
        ("privacy.maxConcurrentGlobal", 1, 256, 32, false),
        ("privacy.maxConcurrentPipeline", 1, 256, 64, false),
        ("privacy.maxConcurrentDecodes", 1, 16, 4, false),
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
