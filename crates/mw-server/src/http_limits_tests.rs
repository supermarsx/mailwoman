use super::*;

const KEYS: [&str; 10] = [
    "MW_HTTP_MAX_CONNECTIONS",
    "MW_HTTP_MAX_REQUESTS",
    "MW_HTTP_MAX_HEADER_BYTES",
    "MW_HTTP_MAX_HEADERS",
    "MW_HTTP_HEADER_TIMEOUT_MS",
    "MW_HTTP_BODY_TIMEOUT_MS",
    "MW_HTTP2_MAX_STREAMS",
    "MW_HTTP_SHUTDOWN_TIMEOUT_MS",
    "MW_HTTP_REQUEST_QUEUE_MS",
    "MW_HTTP_REQUEST_QUEUE_MAX",
];

fn big() -> Resources {
    Resources {
        cpus: Some(16),
        memory_bytes: Some(32 * 1024 * 1024 * 1024),
        open_files: Some(1_048_576),
        open_files_raise: Default::default(),
    }
}

fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |name| {
        pairs
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| (*value).to_owned())
    }
}

#[test]
fn startup_values_are_bounded_and_malformed_values_refuse_startup() {
    HttpLimits::default().validate().unwrap();
    for key in KEYS {
        for value in ["-1", "99999999", "", "1.1", "x"] {
            assert!(
                HttpLimits::read(|name| (name == key).then(|| value.into()), Some(big())).is_err(),
                "{key}={value}"
            );
            assert!(
                HttpLimits::validate_env(|name| (name == key).then(|| value.into())).is_err(),
                "{key}={value}"
            );
        }
        // Zero is valid only for the queue wait (= answer at once).
        let zero = HttpLimits::read(|name| (name == key).then(|| "0".into()), None);
        assert_eq!(zero.is_ok(), key == "MW_HTTP_REQUEST_QUEUE_MS", "{key}=0");
    }
    for value in ["", "yes", "ON", "1"] {
        assert!(
            HttpLimits::validate_env(|name| (name == ADAPTIVE_VAR).then(|| value.into())).is_err()
        );
    }
    assert!(HttpLimits::validate_env(|_| None).is_ok());
}

#[test]
fn without_readings_the_values_are_todays() {
    let limits = HttpLimits::read(|_| None, None).unwrap();
    assert_eq!(
        (
            limits.connections,
            limits.requests,
            limits.http2_streams,
            limits.queue_max
        ),
        (1024, 256, 32, 256)
    );
    assert_eq!(limits.queue_wait, Duration::from_millis(200));
    assert!(!limits.adaptive);
    assert_eq!(limits.derivation.connections, Source::Default);
    // A probe that read nothing derives the same values.
    let limits = HttpLimits::read(|_| None, Some(Resources::default())).unwrap();
    assert_eq!(
        (limits.connections, limits.requests, limits.http2_streams),
        (1024, 256, 32)
    );
    assert_eq!(limits.derivation.connections, Source::Auto);
}

#[test]
fn unset_values_are_derived_and_explicit_values_win() {
    let auto = HttpLimits::read(env(&[("MW_DB_PATH", "postgres://db/m")]), Some(big())).unwrap();
    auto.validate().unwrap();
    assert_eq!(
        (auto.connections, auto.requests, auto.http2_streams),
        (32768, 1024, 100)
    );
    assert_eq!(auto.queue_max, 1024);
    assert_eq!(auto.derivation.db_pool, Some(64));
    assert_eq!(auto.derivation.fd_reserve, Some(512 + 64 + 256));
    let pinned = HttpLimits::read(
        env(&[
            ("MW_HTTP_MAX_CONNECTIONS", "1024"),
            ("MW_HTTP_MAX_REQUESTS", "256"),
            ("MW_HTTP2_MAX_STREAMS", "32"),
            ("MW_HTTP_REQUEST_QUEUE_MS", "0"),
        ]),
        Some(big()),
    )
    .unwrap();
    assert_eq!(
        (pinned.connections, pinned.requests, pinned.http2_streams),
        (1024, 256, 32)
    );
    assert_eq!(pinned.queue_wait, Duration::ZERO);
    assert_eq!(pinned.derivation.connections, Source::Explicit);
    assert_eq!(pinned.derivation.http2_streams, Source::Explicit);
    // An explicit request limit sizes the unset streams and queue.
    let requests = HttpLimits::read(env(&[("MW_HTTP_MAX_REQUESTS", "2000")]), None).unwrap();
    assert_eq!((requests.http2_streams, requests.queue_max), (100, 2000));
    assert_eq!(requests.derivation.http2_streams, Source::Auto);
    // The widened tops of the explicit ranges are accepted.
    let top = HttpLimits::read(
        env(&[
            ("MW_HTTP_MAX_CONNECTIONS", "65536"),
            ("MW_HTTP_MAX_REQUESTS", "16384"),
        ]),
        None,
    )
    .unwrap();
    top.validate().unwrap();
    assert!(HttpLimits::validate_env(env(&[("MW_HTTP_MAX_CONNECTIONS", "65537")])).is_err());
    assert!(HttpLimits::validate_env(env(&[("MW_HTTP_MAX_REQUESTS", "16385")])).is_err());
    let adaptive = HttpLimits::read(env(&[(ADAPTIVE_VAR, "on")]), None).unwrap();
    assert!(adaptive.adaptive);
}

#[test]
fn a_tiny_fd_limit_keeps_the_floor_and_is_reported() {
    let limits = HttpLimits::read(
        |_| None,
        Some(Resources {
            cpus: Some(2),
            memory_bytes: None,
            open_files: Some(1024),
            open_files_raise: Default::default(),
        }),
    )
    .unwrap();
    assert_eq!(limits.connections, 1024);
    assert!(limits.derivation.fd_short);
    assert!(
        limits.summary().contains("open files F 1024"),
        "{}",
        limits.summary()
    );
    // Explicit connections are the operator's choice: no shortfall flag.
    let explicit = HttpLimits::read(
        env(&[("MW_HTTP_MAX_CONNECTIONS", "2048")]),
        Some(Resources {
            open_files: Some(1024),
            ..Resources::default()
        }),
    )
    .unwrap();
    assert!(!explicit.derivation.fd_short);
    assert!(explicit.warnings().is_empty());
}

#[test]
fn floor_clamps_are_warned_and_dropped_terms_named() {
    let limits = HttpLimits::read(
        |_| None,
        Some(Resources {
            cpus: Some(1),
            memory_bytes: Some(512 * 1024 * 1024),
            open_files: Some(1024),
            open_files_raise: super::super::resources::Raise::Raised {
                from: 256,
                to: 1024,
                hard: 1024,
            },
        }),
    )
    .unwrap();
    let warnings = limits.warnings();
    assert_eq!(warnings.len(), 3, "{warnings:#?}");
    assert!(warnings[0].contains("open-file limit (1024)"));
    assert!(warnings[1].contains("connections"));
    assert!(warnings[2].contains("allows 128 requests"));
    assert!(
        limits.summary().contains("soft 256 -> 1024 (hard 1024)"),
        "{}",
        limits.summary()
    );
    let windows = HttpLimits::read(
        |_| None,
        Some(Resources {
            cpus: Some(4),
            ..Resources::default()
        }),
    )
    .unwrap();
    let line = windows.summary();
    assert!(
        line.contains("memory M bytes unknown (term dropped)"),
        "{line}"
    );
    assert!(
        line.contains("open files F unknown (term dropped)"),
        "{line}"
    );
    assert!(line.contains("cpus C 4"), "{line}");
}

#[test]
fn the_raise_switch_is_validated() {
    assert_eq!(parse_raise(None), Ok(true));
    assert_eq!(parse_raise(Some("off")), Ok(false));
    assert!(parse_raise(Some("no")).is_err());
    assert!(HttpLimits::validate_env(env(&[(RAISE_VAR, "maybe")])).is_err());
    assert!(HttpLimits::validate_env(env(&[(RAISE_VAR, "off")])).is_ok());
}

#[test]
fn summary_names_each_source() {
    let limits = HttpLimits::read(env(&[("MW_HTTP_MAX_REQUESTS", "300")]), Some(big())).unwrap();
    let line = limits.summary();
    assert!(line.contains("requests 300 (explicit)"), "{line}");
    assert!(line.contains("connections 32768 (auto)"), "{line}");
    assert!(line.contains("adaptive concurrency off"), "{line}");
}
