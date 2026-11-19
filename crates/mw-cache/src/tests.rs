//! Tests for the layered cache.
//!
//! Two tiers:
//!  * **Always-run** — the memory layer, the scope matrix, the store
//!    fall-through / Redis-absent degradation, and the STRUCTURAL zero-access
//!    exclusion (proved via the cache's own inspection, no live server needed).
//!  * **Live-Valkey** — marked `#[ignore]` so `cargo test -p mw-cache` stays
//!    green offline; CI runs them with `--include-ignored` and a
//!    `MW_TEST_REDIS_URL` (or `MW_TEST_REDIS` / `REDIS_URL`) env var pointing at
//!    `valkey:8`. Run without the env var they log a clear skip and pass.

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

async fn new_store() -> Store {
    Store::open_in_memory(mw_store::ServerKey::generate())
        .await
        .expect("in-memory store")
}

// ── Always-run: matrix / posture ─────────────────────────────────────────────

#[test]
fn spec_defaults_are_the_15_6_column() {
    let m = ScopeMatrix::spec_defaults();
    assert_eq!(m.classes.len(), CacheClass::ALL.len());
    let sessions = m.policy(CacheClass::Sessions).unwrap();
    assert_eq!(sessions.layers, vec![CacheLayer::Memory, CacheLayer::Store]);
    // No class ships with Redis by default (Redis is opt-in per §15.6).
    assert!(
        m.classes
            .iter()
            .all(|c| !c.layers.contains(&CacheLayer::Redis))
    );
    // Message bodies + blobs are store-only.
    assert_eq!(
        m.policy(CacheClass::MessageBodies).unwrap().layers,
        vec![CacheLayer::Store]
    );
}

#[test]
fn override_drops_redis_for_ineligible_blobs() {
    let mut m = ScopeMatrix::spec_defaults();
    let dropped = m.apply_override(ClassPolicy {
        class: CacheClass::Blobs,
        layers: vec![CacheLayer::Memory, CacheLayer::Redis, CacheLayer::Store],
        ttl_secs: 10,
    });
    assert_eq!(dropped, vec![CacheLayer::Redis]);
    assert!(
        !m.policy(CacheClass::Blobs)
            .unwrap()
            .layers
            .contains(&CacheLayer::Redis)
    );

    // Redis IS accepted for an eligible class.
    let dropped = m.apply_override(ClassPolicy {
        class: CacheClass::SearchHotSet,
        layers: vec![CacheLayer::Memory, CacheLayer::Redis],
        ttl_secs: 10,
    });
    assert!(dropped.is_empty());
    assert!(
        m.policy(CacheClass::SearchHotSet)
            .unwrap()
            .layers
            .contains(&CacheLayer::Redis)
    );
}

#[tokio::test]
async fn posture_reflects_config_and_renders() {
    let cache = Cache::in_memory(ScopeMatrix::spec_defaults());
    let p = cache.posture();
    assert_eq!(p.classes.len(), CacheClass::ALL.len());
    assert!(!p.redis_configured);
    assert!(!p.redis_connected);
    assert!(!p.store_attached);
    let text = render_posture(&p);
    assert!(text.contains("not configured"));
    assert!(text.contains("sessions"));
    assert!(text.contains("ttl="));
}

// ── Always-run: memory layer ─────────────────────────────────────────────────

#[tokio::test]
async fn memory_roundtrip_and_loader_runs_once() {
    let cache = Cache::in_memory(ScopeMatrix::spec_defaults());
    let calls = AtomicUsize::new(0);
    let load = || async {
        calls.fetch_add(1, AtomicOrdering::SeqCst);
        Ok::<_, CacheError>("hot".to_string())
    };

    // First read: miss → loader → back-fill memory (SearchHotSet is memory-tier).
    let v: String = cache
        .get(CacheClass::SearchHotSet, "q1", load)
        .await
        .unwrap();
    assert_eq!(v, "hot");
    assert!(cache.memory_contains(CacheClass::SearchHotSet, "q1"));

    // Second read: served from memory, loader NOT re-run.
    let load2 = || async {
        calls.fetch_add(1, AtomicOrdering::SeqCst);
        Ok::<_, CacheError>("SHOULD-NOT-RUN".to_string())
    };
    let v2: String = cache
        .get(CacheClass::SearchHotSet, "q1", load2)
        .await
        .unwrap();
    assert_eq!(v2, "hot");
    assert_eq!(calls.load(AtomicOrdering::SeqCst), 1);

    cache
        .invalidate(CacheClass::SearchHotSet, "q1")
        .await
        .unwrap();
    assert!(!cache.memory_contains(CacheClass::SearchHotSet, "q1"));
}

#[tokio::test]
async fn set_then_get_from_memory() {
    let cache = Cache::in_memory(ScopeMatrix::spec_defaults());
    cache
        .set(CacheClass::HeaderWindows, "inbox:0-50", &vec![1u8, 2, 3])
        .await
        .unwrap();
    assert!(cache.memory_contains(CacheClass::HeaderWindows, "inbox:0-50"));
    let v: Vec<u8> = cache
        .get(CacheClass::HeaderWindows, "inbox:0-50", || async {
            panic!("must hit cache, loader must not run")
        })
        .await
        .unwrap();
    assert_eq!(v, vec![1u8, 2, 3]);
}

// ── Always-run: THE structural zero-access exclusion (no live server) ────────

#[tokio::test]
async fn zero_access_plaintext_never_placed_in_memory() {
    // A class that WOULD cache in memory for a standard account.
    let mut matrix = ScopeMatrix::spec_defaults();
    matrix.apply_override(ClassPolicy {
        class: CacheClass::MessageBodies,
        layers: vec![CacheLayer::Memory, CacheLayer::Store],
        ttl_secs: 60,
    });
    let cache = Cache::in_memory(matrix);

    // Standard account: the plaintext-derived value IS cached in memory.
    cache
        .set_derived(
            CacheClass::MessageBodies,
            "m-standard",
            AccountPosture::Standard,
            &PlaintextDerived::new("body".to_string()),
        )
        .await
        .unwrap();
    assert!(
        cache.memory_contains(CacheClass::MessageBodies, "m-standard"),
        "standard account plaintext SHOULD be cached"
    );

    // Zero-access account: the SAME class + value is FORCED to per-request scope
    // — the type + posture gate the placement; nothing reaches memory.
    cache
        .set_derived(
            CacheClass::MessageBodies,
            "m-zero",
            AccountPosture::ZeroAccess,
            &PlaintextDerived::new("secret-body".to_string()),
        )
        .await
        .unwrap();
    assert!(
        !cache.memory_contains(CacheClass::MessageBodies, "m-zero"),
        "zero-access plaintext MUST NOT be placed in the memory layer"
    );

    // And a zero-access READ always loads per-request, never caching.
    let loaded = AtomicUsize::new(0);
    let v: PlaintextDerived<String> = cache
        .get_derived(
            CacheClass::MessageBodies,
            "m-zero-read",
            AccountPosture::ZeroAccess,
            || async {
                loaded.fetch_add(1, AtomicOrdering::SeqCst);
                Ok::<_, CacheError>("fresh".to_string())
            },
        )
        .await
        .unwrap();
    assert_eq!(v.into_inner(), "fresh");
    assert!(!cache.memory_contains(CacheClass::MessageBodies, "m-zero-read"));
    // A second read loads AGAIN — nothing was cached between them.
    let _ = cache
        .get_derived::<String, _, _>(
            CacheClass::MessageBodies,
            "m-zero-read",
            AccountPosture::ZeroAccess,
            || async {
                loaded.fetch_add(1, AtomicOrdering::SeqCst);
                Ok("fresh".to_string())
            },
        )
        .await
        .unwrap();
    assert_eq!(loaded.load(AtomicOrdering::SeqCst), 2);
}

// ── Always-run: store fall-through + Redis-absent degradation ────────────────

#[tokio::test]
async fn redis_absent_degrades_to_store_no_data_loss() {
    let store = new_store().await;
    // MessageBodies is a store-only class in the defaults.
    let writer = Cache::with_store(ScopeMatrix::spec_defaults(), store.clone());
    assert!(!writer.posture().redis_configured);
    assert!(writer.posture().store_attached);

    writer
        .set(
            CacheClass::MessageBodies,
            "uid-42",
            &"the message".to_string(),
        )
        .await
        .unwrap();

    // A FRESH cache over the same store (simulating a restart / a replica with a
    // cold memory tier and no Redis) still serves the value from the store tier
    // — losing Redis loses performance, never data.
    let reader = Cache::with_store(ScopeMatrix::spec_defaults(), store.clone());
    assert!(!reader.memory_contains(CacheClass::MessageBodies, "uid-42"));
    let v: String = reader
        .get(CacheClass::MessageBodies, "uid-42", || async {
            panic!("must come from the store tier, not the loader")
        })
        .await
        .unwrap();
    assert_eq!(v, "the message");
}

#[tokio::test]
async fn invalidate_tombstones_store_tier() {
    let store = new_store().await;
    let cache = Cache::with_store(ScopeMatrix::spec_defaults(), store.clone());
    cache
        .set(CacheClass::MessageBodies, "uid-9", &"x".to_string())
        .await
        .unwrap();
    cache
        .invalidate(CacheClass::MessageBodies, "uid-9")
        .await
        .unwrap();
    // After invalidation the store tier reads as absent → loader runs.
    let ran = AtomicUsize::new(0);
    let v: String = cache
        .get(CacheClass::MessageBodies, "uid-9", || async {
            ran.fetch_add(1, AtomicOrdering::SeqCst);
            Ok::<_, CacheError>("reloaded".to_string())
        })
        .await
        .unwrap();
    assert_eq!(v, "reloaded");
    assert_eq!(ran.load(AtomicOrdering::SeqCst), 1);
}

// ── Live-Valkey (env-gated, #[ignore]) ───────────────────────────────────────

/// Resolve the live Redis/Valkey URL from any of the accepted env vars, or log a
/// clear skip and return `None`.
fn live_redis_url(test: &str) -> Option<String> {
    for var in ["MW_TEST_REDIS_URL", "MW_TEST_REDIS", "REDIS_URL"] {
        if let Ok(v) = std::env::var(var)
            && !v.trim().is_empty()
        {
            return Some(v);
        }
    }
    eprintln!(
        "[mw-cache] SKIP {test}: no MW_TEST_REDIS_URL / MW_TEST_REDIS / REDIS_URL set \
         (live-Valkey path env-skipped; CI provides valkey:8)"
    );
    None
}

#[tokio::test]
#[ignore = "requires a live Redis/Valkey (MW_TEST_REDIS_URL); run with --include-ignored"]
async fn live_redis_roundtrip_and_degradation() {
    let Some(url) = live_redis_url("live_redis_roundtrip_and_degradation") else {
        return;
    };
    let mut matrix = ScopeMatrix::spec_defaults();
    matrix.apply_override(ClassPolicy {
        class: CacheClass::SearchHotSet,
        layers: vec![CacheLayer::Memory, CacheLayer::Redis],
        ttl_secs: 30,
    });
    let cache = Cache::connect(
        CacheConfig {
            matrix,
            redis_url: Some(url),
            ..Default::default()
        },
        None,
    )
    .await;
    assert!(cache.redis_connected(), "expected a live Valkey connection");

    let key = format!("rt-{}", std::process::id());
    cache
        .set(CacheClass::SearchHotSet, &key, &"live".to_string())
        .await
        .unwrap();
    assert!(cache.redis_contains(CacheClass::SearchHotSet, &key).await);

    // A cold-memory reader still gets the value from Redis.
    let v: String = cache
        .get(CacheClass::SearchHotSet, &key, || async {
            Ok::<_, CacheError>("loader-fallback".to_string())
        })
        .await
        .unwrap();
    assert_eq!(v, "live");

    cache
        .invalidate(CacheClass::SearchHotSet, &key)
        .await
        .unwrap();
    assert!(!cache.redis_contains(CacheClass::SearchHotSet, &key).await);
}

#[tokio::test]
#[ignore = "requires a live Redis/Valkey (MW_TEST_REDIS_URL); run with --include-ignored"]
async fn live_zero_access_plaintext_never_reaches_redis() {
    let Some(url) = live_redis_url("live_zero_access_plaintext_never_reaches_redis") else {
        return;
    };
    let mut matrix = ScopeMatrix::spec_defaults();
    matrix.apply_override(ClassPolicy {
        class: CacheClass::MessageBodies,
        layers: vec![CacheLayer::Memory, CacheLayer::Redis],
        ttl_secs: 30,
    });
    let cache = Cache::connect(
        CacheConfig {
            matrix,
            redis_url: Some(url),
            ..Default::default()
        },
        None,
    )
    .await;
    assert!(cache.redis_connected());

    let pid = std::process::id();
    let std_key = format!("za-standard-{pid}");
    let zero_key = format!("za-zero-{pid}");

    // Standard account: the value DOES reach Redis.
    cache
        .set_derived(
            CacheClass::MessageBodies,
            &std_key,
            AccountPosture::Standard,
            &PlaintextDerived::new("standard-body".to_string()),
        )
        .await
        .unwrap();
    assert!(
        cache
            .redis_contains(CacheClass::MessageBodies, &std_key)
            .await,
        "standard plaintext should reach Redis"
    );

    // Zero-access account: proved against a REAL Valkey instance that the value
    // is never written to Redis (or memory).
    cache
        .set_derived(
            CacheClass::MessageBodies,
            &zero_key,
            AccountPosture::ZeroAccess,
            &PlaintextDerived::new("zero-access-body".to_string()),
        )
        .await
        .unwrap();
    assert!(
        !cache
            .redis_contains(CacheClass::MessageBodies, &zero_key)
            .await,
        "zero-access plaintext MUST NOT reach Redis"
    );
    assert!(!cache.memory_contains(CacheClass::MessageBodies, &zero_key));

    // Cleanup.
    cache
        .invalidate(CacheClass::MessageBodies, &std_key)
        .await
        .unwrap();
    cache
        .invalidate(CacheClass::MessageBodies, &zero_key)
        .await
        .unwrap();
}

#[tokio::test]
async fn validated_mail_cache_does_not_persist_versions_or_negative_bodies() {
    let store = new_store().await;
    let cache = Cache::with_store(ScopeMatrix::spec_defaults(), store.clone());
    let key = envelope_cache_key("account", "message", "revision");
    assert_eq!(
        cache
            .get_mail_bytes(
                CacheClass::HeaderWindows,
                &key,
                AccountPosture::Standard,
                || async { Ok(Some(b"header".to_vec())) }
            )
            .await
            .unwrap()
            .into_inner(),
        Some(b"header".to_vec())
    );
    assert!(cache.memory_contains(CacheClass::HeaderWindows, &key));
    assert!(
        store
            .get_setting(&store_key(&namespaced(CacheClass::HeaderWindows, &key)))
            .await
            .unwrap()
            .is_none()
    );
    cache.apply_override(ClassPolicy {
        class: CacheClass::MessageBodies,
        layers: vec![CacheLayer::Memory, CacheLayer::Store],
        ttl_secs: 0,
    });
    let body = body_cache_key("account", "blob");
    assert!(
        cache
            .get_mail_bytes(
                CacheClass::MessageBodies,
                &body,
                AccountPosture::Standard,
                || async { Ok(None) }
            )
            .await
            .unwrap()
            .into_inner()
            .is_none()
    );
    assert!(!cache.memory_contains(CacheClass::MessageBodies, &body));
    assert_eq!(
        cache
            .get_mail_bytes(
                CacheClass::MessageBodies,
                &body,
                AccountPosture::Standard,
                || async { Ok(Some(b"arrived".to_vec())) }
            )
            .await
            .unwrap()
            .into_inner(),
        Some(b"arrived".to_vec())
    );
    assert!(
        store
            .get_setting(&store_key(&namespaced(CacheClass::MessageBodies, &body)))
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn validated_mail_excludes_zero_access_and_sessions_and_is_byte_bounded() {
    let cache = Cache::default();
    for (class, posture) in [
        (CacheClass::HeaderWindows, AccountPosture::ZeroAccess),
        (CacheClass::Sessions, AccountPosture::Standard),
    ] {
        let calls = AtomicUsize::new(0);
        for _ in 0..2 {
            cache
                .get_mail_bytes(class, "never-shared", posture, || async {
                    calls.fetch_add(1, AtomicOrdering::SeqCst);
                    Ok(Some(vec![1u8]))
                })
                .await
                .unwrap();
        }
        assert_eq!(calls.load(AtomicOrdering::SeqCst), 2);
        assert!(!cache.memory_contains(class, "never-shared"));
    }
    let large = vec![0u8; MAIL_ENTRY_BYTES + 1];
    let got = cache
        .get_mail_bytes(
            CacheClass::HeaderWindows,
            "oversized",
            AccountPosture::Standard,
            || async { Ok(Some(large)) },
        )
        .await
        .unwrap();
    assert_eq!(got.into_inner().unwrap().len(), MAIL_ENTRY_BYTES + 1);
    assert!(!cache.memory_contains(CacheClass::HeaderWindows, "oversized"));
    for id in 0..20 {
        cache
            .get_mail_bytes(
                CacheClass::HeaderWindows,
                &format!("bounded-{id}"),
                AccountPosture::Standard,
                || async { Ok(Some(vec![0u8; 4 * 1024 * 1024])) },
            )
            .await
            .unwrap();
    }
    cache.run_pending_memory_tasks().await;
    assert!(cache.mail_memory_bytes() <= MAIL_MEMORY_BYTES);
}

#[tokio::test]
async fn validated_mail_respects_disabled_and_small_configured_memory_capacity() {
    let disabled = Cache::build(ScopeMatrix::spec_defaults(), None, false, None, 0);
    disabled
        .get_mail_bytes(
            CacheClass::HeaderWindows,
            "disabled",
            AccountPosture::Standard,
            || async { Ok(Some(vec![1u8])) },
        )
        .await
        .unwrap();
    disabled.run_pending_memory_tasks().await;
    assert!(!disabled.memory_contains(CacheClass::HeaderWindows, "disabled"));
    let tiny = Cache::build(ScopeMatrix::spec_defaults(), None, false, None, 1);
    for id in ["one", "two", "three"] {
        tiny.get_mail_bytes(
            CacheClass::HeaderWindows,
            id,
            AccountPosture::Standard,
            || async { Ok(Some(vec![1u8])) },
        )
        .await
        .unwrap();
    }
    tiny.run_pending_memory_tasks().await;
    assert!(tiny.inner.mail_memory.entry_count() <= 1);
    assert!(tiny.mail_memory_bytes() <= MAIL_MEMORY_BYTES);
}

#[tokio::test]
async fn mail_ephemeral_invalidation_never_creates_store_tombstones() {
    let store = new_store().await;
    let cache = Cache::with_store(ScopeMatrix::spec_defaults(), store.clone());
    cache
        .invalidate_ephemeral(CacheClass::MessageBodies, "expunged-message")
        .await
        .unwrap();
    assert!(
        store
            .get_setting(&store_key(&namespaced(
                CacheClass::MessageBodies,
                "expunged-message"
            )))
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn mail_redis_envelopes_fail_closed_on_wrong_key_namespace_and_corruption() {
    let source = Cache::with_store(ScopeMatrix::spec_defaults(), new_store().await);
    let wrong_key = Cache::with_store(ScopeMatrix::spec_defaults(), new_store().await);
    let namespace = namespaced(
        CacheClass::MessageBodies,
        &body_cache_key("account", "blob"),
    );
    let envelope = source
        .seal_mail_value(&namespace, b"private message body")
        .unwrap();
    assert_eq!(
        source.open_mail_value(&namespace, &envelope).unwrap(),
        b"private message body"
    );
    assert!(wrong_key.open_mail_value(&namespace, &envelope).is_none());
    assert!(
        source
            .open_mail_value("other-account-and-blob", &envelope)
            .is_none()
    );
    assert!(
        source
            .open_mail_value(&namespace, b"private message body")
            .is_none()
    );
    let mut corrupt = envelope;
    corrupt[30] ^= 1;
    assert!(source.open_mail_value(&namespace, &corrupt).is_none());
    assert!(
        Cache::default()
            .seal_mail_value(&namespace, b"mail")
            .is_none()
    );
}

#[tokio::test]
#[ignore = "requires a live Redis/Valkey (MW_TEST_REDIS_URL); run with --include-ignored"]
async fn live_validated_mail_is_encrypted_across_cache_instances() {
    let Some(url) = live_redis_url("live_validated_mail_is_encrypted_across_cache_instances")
    else {
        return;
    };
    let store = new_store().await;
    let mut matrix = ScopeMatrix::spec_defaults();
    matrix.apply_override(ClassPolicy {
        class: CacheClass::MessageBodies,
        layers: vec![CacheLayer::Redis],
        ttl_secs: 30,
    });
    let config = CacheConfig {
        matrix,
        redis_url: Some(url),
        ..Default::default()
    };
    let first = Cache::connect(config.clone(), Some(store.clone())).await;
    let second = Cache::connect(config, Some(store)).await;
    let key = body_cache_key(
        &format!("encrypted-{}", std::process::id()),
        "immutable-blob",
    );
    first
        .invalidate_ephemeral(CacheClass::MessageBodies, &key)
        .await
        .unwrap();
    let plaintext = b"a private message body that must not be readable in Redis";
    first
        .get_mail_bytes(
            CacheClass::MessageBodies,
            &key,
            AccountPosture::Standard,
            || async { Ok(Some(plaintext.to_vec())) },
        )
        .await
        .unwrap();
    let raw = first
        .inner
        .redis
        .as_ref()
        .unwrap()
        .get::<Option<Bytes>, _>(namespaced(CacheClass::MessageBodies, &key))
        .await
        .unwrap()
        .unwrap();
    assert!(raw.starts_with(b"MWB1"));
    assert!(!raw.windows(plaintext.len()).any(|v| v == plaintext));
    let hit = second
        .get_mail_bytes(
            CacheClass::MessageBodies,
            &key,
            AccountPosture::Standard,
            || async { Err(CacheError::Store("unexpected cache miss".into())) },
        )
        .await
        .unwrap()
        .into_inner();
    assert_eq!(hit, Some(plaintext.to_vec()));
    first
        .invalidate_ephemeral(CacheClass::MessageBodies, &key)
        .await
        .unwrap();
}
