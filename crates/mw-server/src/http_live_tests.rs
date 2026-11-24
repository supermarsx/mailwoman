use super::super::limits::{Derivation, HttpLimits, Source};
use super::super::resources::{Auto, Resources};
use super::*;

fn derived(fd_bound: Option<u64>, memory_request_bound: Option<u64>) -> HttpLimits {
    HttpLimits {
        connections: 4096,
        requests: 1024,
        http2_streams: 100,
        queue_max: 1024,
        derivation: Derivation {
            resources: Resources {
                cpus: Some(16),
                ..Resources::default()
            },
            auto: Some(Auto {
                connections: 4096,
                requests: 1024,
                http2_streams: 100,
                fd_reserve: Some(800),
                fd_short: false,
                memory_short_connections: false,
                memory_short_requests: false,
                fd_bound,
                memory_request_bound,
            }),
            connections: Source::Auto,
            requests: Source::Auto,
            http2_streams: Source::Auto,
            queue_max: Source::Auto,
            ..Derivation::default()
        },
        ..HttpLimits::default()
    }
}

#[test]
fn auto_applies_without_console_values() {
    let (live, sources) = resolve(&derived(None, None), &Console::default());
    assert_eq!(
        (
            live.connections,
            live.requests,
            live.http2_streams,
            live.queue_max
        ),
        (4096, 1024, 100, 1024)
    );
    assert_eq!(live.adaptive_floor, 256);
    assert_eq!(sources.requests, Source::Auto);
}

#[test]
fn console_values_win_over_auto_and_follow_through() {
    let console = Console {
        requests: Some(400),
        queue_wait_ms: Some(50),
        adaptive: Some(true),
        ..Console::default()
    };
    let (live, sources) = resolve(&derived(None, None), &console);
    assert_eq!(live.requests, 400);
    assert_eq!(live.http2_streams, 50, "streams follow the request limit");
    assert_eq!(live.queue_max, 400, "queue follows the request limit");
    assert_eq!(live.queue_wait_ms, 50);
    assert!(live.adaptive);
    assert_eq!(sources.requests, Source::Console);
    assert_eq!(sources.queue_wait_ms, Source::Console);
}

#[test]
fn the_environment_wins_over_the_console() {
    let mut limits = derived(None, None);
    limits.requests = 300;
    limits.derivation.requests = Source::Explicit;
    limits.derivation.adaptive = Source::Explicit;
    limits.adaptive = false;
    let (live, sources) = resolve(
        &limits,
        &Console {
            requests: Some(2000),
            adaptive: Some(true),
            ..Console::default()
        },
    );
    assert_eq!(live.requests, 300);
    assert!(!live.adaptive);
    assert_eq!(sources.requests, Source::Explicit);
}

#[test]
fn console_values_are_held_to_the_safety_bounds() {
    let limits = derived(Some(3000), Some(512));
    let (live, sources) = resolve(
        &limits,
        &Console {
            connections: Some(60000),
            requests: Some(16000),
            ..Console::default()
        },
    );
    assert_eq!(live.connections, 3000);
    assert_eq!(live.requests, 512);
    assert!(sources.connections_capped && sources.requests_capped);
    // A tiny descriptor bound never caps below the 1024 floor.
    let (live, _) = resolve(
        &derived(Some(10), Some(10)),
        &Console {
            connections: Some(2000),
            requests: Some(1000),
            ..Console::default()
        },
    );
    assert_eq!((live.connections, live.requests), (1024, 256));
    // Smaller console values are taken as they are.
    let (live, sources) = resolve(
        &limits,
        &Console {
            connections: Some(16),
            requests: Some(1),
            queue_max: Some(1),
            ..Console::default()
        },
    );
    assert_eq!(
        (live.connections, live.requests, live.queue_max),
        (16, 1, 1)
    );
    assert!(!sources.requests_capped);
}

#[test]
fn rows_parse_null_as_auto() {
    let n = serde_json::json!(512);
    let null = Value::Null;
    let t = Value::Bool(true);
    let console = Console::from_pairs([
        (MAX_REQUESTS, &n),
        (MAX_CONNECTIONS, &null),
        (ADAPTIVE, &t),
        ("server.other", &n),
    ]);
    assert_eq!(console.requests, Some(512));
    assert_eq!(console.connections, None);
    assert_eq!(console.adaptive, Some(true));
}

#[tokio::test]
async fn resizing_never_exceeds_the_size_and_waits_for_held_permits() {
    let pool = Resizable::new(4);
    let held: Vec<_> = (0..3).map(|_| pool.try_acquire().unwrap()).collect();
    pool.set(2);
    // One idle permit was taken at once; one is owed by the holders.
    assert_eq!((pool.size(), pool.available()), (2, 0));
    let mut held = held.into_iter();
    pool.release(held.next().unwrap());
    assert_eq!(
        pool.available(),
        0,
        "the first returned permit pays the debt"
    );
    pool.release(held.next().unwrap());
    assert_eq!(pool.available(), 1);
    // Growing cancels debt first, then adds.
    pool.set(1);
    assert_eq!(pool.available(), 0);
    pool.set(5);
    pool.release(held.next().unwrap());
    assert_eq!((pool.size(), pool.available()), (5, 5));
}

#[test]
fn every_key_is_registered_global_validated_and_pinned_where_it_has_a_variable() {
    use crate::settings_registry::{lookup, refuse_if_pinned, validate};
    for key in KEYS {
        let def = lookup(key).unwrap_or_else(|| panic!("{key} not registered"));
        assert_eq!(def.category, "maintenance", "{key}");
        assert!(!def.user_override, "{key}");
    }
    for (key, too_big, var) in [
        (MAX_CONNECTIONS, 65537, "MW_HTTP_MAX_CONNECTIONS"),
        (MAX_REQUESTS, 16385, "MW_HTTP_MAX_REQUESTS"),
        (HTTP2_MAX_STREAMS, 129, "MW_HTTP2_MAX_STREAMS"),
        (REQUEST_QUEUE_MS, 2001, "MW_HTTP_REQUEST_QUEUE_MS"),
        (REQUEST_QUEUE_MAX, 65537, "MW_HTTP_REQUEST_QUEUE_MAX"),
    ] {
        let def = lookup(key).unwrap();
        let refused = validate(def, &serde_json::json!(too_big), "/globalValue").unwrap_err();
        assert_eq!(
            (refused.status, refused.field.as_deref()),
            (422, Some("/globalValue")),
            "{key}"
        );
        assert!(
            validate(def, &serde_json::json!(16), "/globalValue").is_ok(),
            "{key}"
        );
        let set = |name: &str| (name == var).then(|| "64".to_owned());
        assert_eq!(
            refuse_if_pinned(def, &set).unwrap_err().status,
            409,
            "{key}"
        );
        assert!(refuse_if_pinned(def, &|_| None).is_ok(), "{key}");
    }
    for key in [
        MAX_CONNECTIONS,
        MAX_REQUESTS,
        HTTP2_MAX_STREAMS,
        REQUEST_QUEUE_MAX,
        ADAPTIVE_FLOOR,
    ] {
        let def = lookup(key).unwrap();
        assert!(def.nullable, "{key}: Auto is null");
        assert!(validate(def, &Value::Null, "/globalValue").is_ok(), "{key}");
    }
}
