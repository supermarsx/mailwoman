//! The cache's **degradation and default** behaviour, from outside the crate.
//!
//! `src/tests.rs` covers the happy paths and the structural zero-access exclusion.
//! What it does not cover — and what actually decides whether a deployment stays up
//! or leaks — is what happens when things are *missing*:
//!
//! * a Redis/Valkey URL is configured but nothing is listening (SPEC §15: Redis is
//!   never authoritative, so this must degrade silently, not fail construction);
//! * a cache class is absent from the matrix entirely (it must fall back to the
//!   authoritative store, **never** to a shared memory tier);
//! * a per-class TTL actually expires the memory entry, and `ttl_secs = 0` means
//!   "no expiry" rather than "expire immediately";
//! * `mailwoman doctor` renders "configured, UNREACHABLE" differently from "not
//!   configured" — an operator reading the wrong one of those two chases the wrong
//!   problem.
//!
//! These live in `tests/` rather than in the inline module on purpose: `tests/` is
//! excluded from the coverage denominator, so what they move is coverage of
//! `src/lib.rs` and nothing else.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use mw_cache::{
    AccountPosture, Cache, CacheClass, CacheConfig, CacheError, CacheLayer, CachePosture,
    ClassPolicy, PlaintextDerived, ScopeMatrix, render_posture,
};
use mw_store::{ServerKey, Store};

async fn new_store() -> Store {
    Store::open_in_memory(ServerKey::generate())
        .await
        .expect("in-memory store")
}

/// A loopback port with nothing listening — a connect there is refused promptly on
/// every supported host, so this exercises the unreachable-Redis path offline.
const DEAD_REDIS: &str = "redis://127.0.0.1:1";

// ── Redis absent / unreachable ───────────────────────────────────────────────

#[tokio::test]
async fn an_unreachable_redis_degrades_instead_of_failing_construction() {
    let store = new_store().await;
    let cache = tokio::time::timeout(
        Duration::from_secs(30),
        Cache::connect(
            CacheConfig {
                matrix: ScopeMatrix::spec_defaults(),
                redis_url: Some(DEAD_REDIS.to_string()),
                memory_capacity: 128,
            },
            Some(store.clone()),
        ),
    )
    .await
    .expect("Cache::connect must give up on an unreachable Redis, not block forever");

    // Configured, but not connected — and the distinction is visible to `doctor`.
    let posture = cache.posture();
    assert!(posture.redis_configured, "the URL was configured");
    assert!(!posture.redis_connected, "…and nothing was listening");
    assert!(!cache.redis_connected());
    assert!(posture.store_attached);
    assert!(render_posture(&posture).contains("configured, UNREACHABLE (degraded)"));

    // And the cache still works end to end: memory serves, the store persists.
    let ran = AtomicUsize::new(0);
    let v: String = cache
        .get(CacheClass::SearchHotSet, "q", || async {
            ran.fetch_add(1, Ordering::SeqCst);
            Ok::<_, CacheError>("value".to_string())
        })
        .await
        .expect("loader path");
    assert_eq!(v, "value");
    assert!(cache.memory_contains(CacheClass::SearchHotSet, "q"));

    cache
        .set(CacheClass::MessageBodies, "uid-1", &"body".to_string())
        .await
        .expect("store-tier write survives a dead Redis");
    let fresh = Cache::with_store(ScopeMatrix::spec_defaults(), store);
    let round: String = fresh
        .get(CacheClass::MessageBodies, "uid-1", || async {
            panic!("the store tier must still hold it — losing Redis loses no data")
        })
        .await
        .expect("store read");
    assert_eq!(round, "body");

    // A Redis-tier probe on a cache with no client is `false`, not an error.
    assert!(!cache.redis_contains(CacheClass::SearchHotSet, "q").await);
}

#[tokio::test]
async fn connect_without_a_redis_url_reports_not_configured_and_defaults_the_matrix() {
    // An empty matrix means "use the SPEC §15.6 defaults" rather than "no classes at
    // all" — the latter would place every class store-only and quietly cost the
    // memory tier.
    let cache = Cache::connect(CacheConfig::default(), None).await;
    let posture = cache.posture();
    assert!(!posture.redis_configured);
    assert!(!posture.redis_connected);
    assert!(!posture.store_attached);
    assert_eq!(posture.classes.len(), CacheClass::ALL.len());
    assert!(render_posture(&posture).contains("not configured"));

    // SearchHotSet is a memory class in the defaults, so this proves the defaults
    // were applied and not an empty matrix.
    cache
        .set(CacheClass::SearchHotSet, "k", &1u32)
        .await
        .expect("set");
    assert!(cache.memory_contains(CacheClass::SearchHotSet, "k"));
}

#[tokio::test]
async fn the_default_cache_is_memory_only_with_the_spec_matrix() {
    let cache = Cache::default();
    let posture = cache.posture();
    assert_eq!(posture.classes.len(), CacheClass::ALL.len());
    assert!(!posture.redis_configured && !posture.store_attached);
}

// ── unknown classes fall back to the authoritative tier ──────────────────────

#[tokio::test]
async fn a_class_absent_from_the_matrix_never_lands_in_the_shared_memory_tier() {
    // The fallback policy for an unconfigured class is store-only. That direction
    // matters: defaulting to memory would place a class nobody configured — possibly
    // a plaintext-derived one — into a shared in-process tier.
    let cache = Cache::in_memory(ScopeMatrix::default()); // no class rows at all
    let ran = AtomicUsize::new(0);
    for _ in 0..2 {
        let v: String = cache
            .get(CacheClass::MessageBodies, "k", || async {
                ran.fetch_add(1, Ordering::SeqCst);
                Ok::<_, CacheError>("plaintext".to_string())
            })
            .await
            .expect("loader");
        assert_eq!(v, "plaintext");
    }
    assert!(
        !cache.memory_contains(CacheClass::MessageBodies, "k"),
        "an unconfigured class must not be cached in memory"
    );
    assert_eq!(
        ran.load(Ordering::SeqCst),
        2,
        "with no store attached the loader is the only tier, so it runs each time"
    );
}

// ── runtime admin override ───────────────────────────────────────────────────

#[tokio::test]
async fn a_runtime_override_takes_effect_and_still_refuses_redis_for_blobs() {
    let cache = Cache::in_memory(ScopeMatrix::spec_defaults());

    // MessageBodies is store-only by default; grant it a memory tier at runtime.
    let dropped = cache.apply_override(ClassPolicy {
        class: CacheClass::MessageBodies,
        layers: vec![CacheLayer::Memory],
        ttl_secs: 60,
    });
    assert!(dropped.is_empty());
    cache
        .set(CacheClass::MessageBodies, "uid", &"b".to_string())
        .await
        .expect("set");
    assert!(cache.memory_contains(CacheClass::MessageBodies, "uid"));

    // Blobs are content-addressed and are never Redis-eligible; an admin asking for
    // it has the layer dropped and is told which one, rather than silently ignored.
    let dropped = cache.apply_override(ClassPolicy {
        class: CacheClass::Blobs,
        layers: vec![CacheLayer::Memory, CacheLayer::Redis],
        ttl_secs: 30,
    });
    assert_eq!(dropped, vec![CacheLayer::Redis]);
    let blobs = cache
        .posture()
        .classes
        .into_iter()
        .find(|c| c.class == CacheClass::Blobs)
        .expect("blobs row");
    assert!(!blobs.layers.contains(&CacheLayer::Redis));
    assert!(!CacheClass::Blobs.redis_eligible());
    assert!(
        CacheClass::ALL
            .iter()
            .filter(|c| !c.redis_eligible())
            .count()
            == 1
    );
}

// ── per-entry TTL ────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_class_ttl_expires_the_memory_entry_and_zero_means_no_expiry() {
    let mut matrix = ScopeMatrix::spec_defaults();
    matrix.apply_override(ClassPolicy {
        class: CacheClass::PushPresence,
        layers: vec![CacheLayer::Memory],
        ttl_secs: 1,
    });
    matrix.apply_override(ClassPolicy {
        class: CacheClass::RateLimit,
        layers: vec![CacheLayer::Memory],
        ttl_secs: 0,
    });
    let cache = Cache::in_memory(matrix);

    cache
        .set(CacheClass::PushPresence, "u1", &"online".to_string())
        .await
        .expect("set");
    cache
        .set(CacheClass::RateLimit, "u1", &7u32)
        .await
        .expect("set");
    assert!(cache.memory_contains(CacheClass::PushPresence, "u1"));

    tokio::time::sleep(Duration::from_millis(1_400)).await;
    cache.run_pending_memory_tasks().await;

    // The TTL'd entry is gone — proved by the loader running again, not just by the
    // inspection helper.
    assert!(!cache.memory_contains(CacheClass::PushPresence, "u1"));
    let ran = AtomicUsize::new(0);
    let v: String = cache
        .get(CacheClass::PushPresence, "u1", || async {
            ran.fetch_add(1, Ordering::SeqCst);
            Ok::<_, CacheError>("reloaded".to_string())
        })
        .await
        .expect("reload after expiry");
    assert_eq!(v, "reloaded");
    assert_eq!(ran.load(Ordering::SeqCst), 1);

    // ttl_secs = 0 means "no expiry", not "expire at once" — a rate-limit counter
    // that vanished immediately would let a caller past the limit every request.
    assert!(
        cache.memory_contains(CacheClass::RateLimit, "u1"),
        "ttl_secs = 0 must mean no expiry"
    );
    let n: u32 = cache
        .get(CacheClass::RateLimit, "u1", || async {
            panic!("a ttl=0 entry must still be cached")
        })
        .await
        .expect("cached");
    assert_eq!(n, 7);
}

// ── plaintext-derived values, standard account ───────────────────────────────

#[tokio::test]
async fn a_standard_account_plaintext_read_is_cached_after_the_first_load() {
    // The mirror image of the zero-access exclusion: for a conventional account the
    // derived value follows the class matrix like any other. A test that only pinned
    // the zero-access side would pass just as well if caching were broken outright.
    let mut matrix = ScopeMatrix::spec_defaults();
    matrix.apply_override(ClassPolicy {
        class: CacheClass::MessageBodies,
        layers: vec![CacheLayer::Memory],
        ttl_secs: 60,
    });
    let cache = Cache::in_memory(matrix);

    let ran = AtomicUsize::new(0);
    for _ in 0..2 {
        let v: PlaintextDerived<String> = cache
            .get_derived(
                CacheClass::MessageBodies,
                "m1",
                AccountPosture::Standard,
                || async {
                    ran.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, CacheError>("body".to_string())
                },
            )
            .await
            .expect("derived read");
        assert_eq!(v.into_inner(), "body");
    }
    assert_eq!(
        ran.load(Ordering::SeqCst),
        1,
        "second read came from memory"
    );
    assert!(cache.memory_contains(CacheClass::MessageBodies, "m1"));
    assert_eq!(PlaintextDerived::new(9u8).into_inner(), 9);
}

#[tokio::test]
async fn invalidating_an_absent_key_is_not_an_error() {
    let cache = Cache::with_store(ScopeMatrix::spec_defaults(), new_store().await);
    cache
        .invalidate(CacheClass::GalDirectory, "never-written")
        .await
        .expect("invalidate is idempotent");
    assert!(!cache.memory_contains(CacheClass::GalDirectory, "never-written"));
}

// ── the doctor rendering ─────────────────────────────────────────────────────

#[test]
fn the_posture_table_distinguishes_absent_unreachable_and_connected_redis() {
    let render = |configured, connected| {
        render_posture(&CachePosture {
            classes: vec![ClassPolicy {
                class: CacheClass::Sessions,
                layers: vec![CacheLayer::Memory, CacheLayer::Redis, CacheLayer::Store],
                ttl_secs: 3_600,
            }],
            redis_configured: configured,
            redis_connected: connected,
            store_attached: configured,
        })
    };
    assert!(render(false, false).contains("not configured"));
    assert!(render(true, true).contains("connected"));
    let degraded = render(true, false);
    assert!(degraded.contains("configured, UNREACHABLE (degraded)"));
    // `connected` alone is ambiguous — the degraded line must not read as healthy.
    assert!(!degraded.contains("  redis/valkey: connected"));

    // Layers are joined in matrix order, and the store attachment is stated.
    let full = render(true, true);
    assert!(full.contains("memory+redis+store"), "{full}");
    assert!(full.contains("store fall-through: attached"), "{full}");
    assert!(full.contains("ttl=3600s"), "{full}");
    assert!(render(false, false).contains("store fall-through: none"));
}

#[test]
fn a_class_with_no_layers_renders_as_none_rather_than_blank() {
    // A blank column reads as a rendering bug; "none" reads as the (deliberate)
    // loader-only posture it is.
    let text = render_posture(&CachePosture {
        classes: vec![ClassPolicy {
            class: CacheClass::Blobs,
            layers: Vec::new(),
            ttl_secs: 0,
        }],
        redis_configured: false,
        redis_connected: false,
        store_attached: false,
    });
    assert!(text.contains("blobs"), "{text}");
    assert!(text.contains("none"), "{text}");
}

#[test]
fn every_class_has_a_stable_slug_and_they_are_all_distinct() {
    // The slug namespaces every key in every tier; two classes sharing one would let
    // a `Sessions` read serve a `Blobs` entry.
    let slugs: Vec<&str> = CacheClass::ALL.iter().map(|c| c.as_str()).collect();
    let mut sorted = slugs.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        slugs.len(),
        "duplicate class slug in {slugs:?}"
    );
    assert!(slugs.iter().all(|s| !s.is_empty()));
}

#[test]
fn cache_errors_say_which_tier_failed() {
    assert_eq!(
        CacheError::Redis("timeout".into()).to_string(),
        "redis/valkey error: timeout"
    );
    assert_eq!(
        CacheError::Store("locked".into()).to_string(),
        "store error: locked"
    );
    let serde_err: CacheError = serde_json::from_str::<u32>("nope").unwrap_err().into();
    assert!(
        serde_err.to_string().starts_with("serialization error:"),
        "{serde_err}"
    );
}
