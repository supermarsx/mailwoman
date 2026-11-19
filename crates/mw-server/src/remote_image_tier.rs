//! Compatibility path; implementation lives in `mw-remote-content`.
pub(crate) use mw_remote_content::tier::*;

/// The admin readout: configured against effective values, and per-tier size,
/// entries, hit rate, hot and cold counts and promotions per minute. Hot and cold
/// are entries at or above the promote threshold and below the demote threshold.
pub(crate) fn readout(cfg: &TierConfig, st: &TierStats) -> serde_json::Value {
    let c = &st.counters;
    let rate = |hits: u64, total: u64| (total > 0).then(|| hits * 100 / total);
    let lookups = c.ram_hits + c.disk_hits + c.misses;
    let swr = super::swr::counters();
    let resized = super::resize::counters();
    serde_json::json!({
        "configured": {
            "ramBytes": cfg.ram_bytes, "ramEntries": cfg.ram_entries, "ramEntryBytes": cfg.ram_entry_bytes,
            "ttlBaseSecs": cfg.ttl_base_secs, "ttlStepSecs": cfg.ttl_step_secs, "ttlMaxSecs": cfg.ttl_max_secs,
            "maxAgeSecs": cfg.max_age_secs, "halfLifeSecs": cfg.half_life_secs,
            "demoteMilli": cfg.demote_milli, "hysteresisPercent": cfg.hysteresis_pct,
            "openPercent": cfg.open_pct, "recentPercent": cfg.recent_pct,
            "accountSharePercent": cfg.account_share_pct, "sweepSecs": cfg.sweep_secs,
            "snapshotSecs": cfg.snapshot_secs, "sweepBatch": cfg.sweep_batch,
            "disk": {
                "enabled": cfg.disk.enabled, "mode": if cfg.disk.auto { "auto" } else { "manual" },
                "path": cfg.disk.path.as_ref().map(|p| p.display().to_string()),
                "manualBytes": cfg.disk.manual_bytes, "floorBytes": cfg.disk.floor_bytes,
                "ceilingBytes": cfg.disk.ceiling_bytes, "entries": cfg.disk.entries,
                "entryBytes": cfg.disk.entry_bytes, "minHits": cfg.disk.min_hits,
                "writeQueueBytes": cfg.disk.write_queue_bytes,
            },
        },
        "ram": {
            "bytes": st.ram_bytes, "usedBytes": st.ram_used, "budgetBytes": st.ram_budget,
            "entries": st.ram_entries, "hot": st.ram_hot, "cold": st.ram_cold,
            "hits": c.ram_hits, "hitPercent": rate(c.ram_hits, lookups),
        },
        "disk": {
            "enabled": st.disk_enabled, "active": st.disk_active, "path": st.path,
            "bytes": st.disk_bytes, "budgetBytes": st.disk_budget, "entries": st.disk_entries,
            "hot": st.disk_hot, "cold": st.disk_cold, "pendingBytes": st.pending_bytes,
            "hits": c.disk_hits, "hitPercent": rate(c.disk_hits, lookups),
            "errors": c.disk_errors, "lastError": st.last_error,
        },
        "index": { "entries": st.index_entries, "driftRecoveries": c.drift_recoveries },
        "promotionsPerMin": st.promotions_per_min,
        "promotions": c.promotions, "demotions": c.demotions, "drops": c.drops,
        "misses": c.misses, "hitPercent": rate(c.ram_hits + c.disk_hits, lookups),
        "freshness": {
            "staleSecs": cfg.stale_secs, "maxStaleSecs": cfg.max_stale_secs,
            "hit": swr.hit, "stale": swr.stale, "miss": swr.miss,
            "revalidated": swr.revalidated, "replaced": swr.replaced,
            "revalidationFailures": swr.failed, "staleOnError": swr.stale_on_error,
            "revalidating": super::swr::in_progress(),
        },
        "resize": {
            "resized": resized.0, "bytesSaved": resized.1, "resizeMs": resized.2, "passedThrough": resized.3,
        },
    })
}

#[cfg(test)]
mod auth_tests {
    use super::*;
    fn png(n: u8, len: usize) -> Vec<u8> {
        vec![n; len]
    }
    fn etag(bytes: &[u8]) -> String {
        mw_remote_content::etag_for(bytes)
    }
    // ── authorisation comes first ──────────────────────────────────────────────────

    /// The cache holds public bytes, so the only thing between a requester and them is
    /// that authentication and the signed-URL check run BEFORE any lookup. Seed the
    /// real global tier with the exact key the legacy proxy would use, then ask for it
    /// without a session and with a forged signed URL: neither may be served, and the
    /// tier must not even be probed (its hit and miss counters do not move).
    /// `build_app` mounts every route and its future is large in a debug build: on the
    /// default 2 MiB test-thread stack this test overflowed as the app grew (it passes at
    /// 3 MiB), so it runs its runtime on a thread with a stack that leaves room.
    #[test]
    fn an_unauthenticated_or_unsigned_request_never_reads_the_cache() {
        std::thread::Builder::new()
            .stack_size(32 << 20)
            .spawn(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .thread_stack_size(32 << 20)
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(unauthenticated_requests_never_read_the_cache())
            })
            .unwrap()
            .join()
            .unwrap();
    }

    async fn unauthenticated_requests_never_read_the_cache() {
        let dir = std::env::temp_dir().join(format!("mw-tier-auth-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let config = crate::AppConfig {
            db_path: dir.join("mw.db").to_string_lossy().into_owned(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: crate::ServerMode::Proxy,
            hardening: crate::HardeningConfig::default(),
            security: crate::SecurityConfig::default(),
        };
        let app = crate::build_app(config).await.expect("build_app");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let url = "https://cdn.example/seeded-secret.png";
        let body = png(9, 600);
        let tag = etag(&body);
        let tier = global();
        tier.store(
            "direct:legacy:true:8388608",
            url,
            "acct",
            Context::Open,
            tag.clone(),
            body.clone(),
        );
        tier.store(
            "direct:true:8388608",
            url,
            "acct",
            Context::Open,
            tag,
            body.clone(),
        );
        assert!(
            tier.lookup("direct:legacy:true:8388608", url, Context::Open)
                .await
                .is_some(),
            "the seed is really in the cache"
        );
        let before = tier.stats(now_ms()).counters;

        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let legacy = client
            .get(format!(
                "{base}/api/image-proxy?url=https%3A%2F%2Fcdn.example%2Fseeded-secret.png"
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(legacy.status(), 401, "no session, no image");
        assert_ne!(legacy.bytes().await.unwrap().as_ref(), &body[..]);
        for path in ["/api/rc/i/forged.token", "/api/rc/i/"] {
            let r = client.get(format!("{base}{path}")).send().await.unwrap();
            assert!(
                r.status().is_client_error(),
                "{path} must be refused, got {}",
                r.status()
            );
            assert_ne!(r.bytes().await.unwrap().as_ref(), &body[..], "{path}");
        }
        let after = tier.stats(now_ms()).counters;
        assert_eq!(
            (before.ram_hits, before.disk_hits, before.misses),
            (after.ram_hits, after.disk_hits, after.misses),
            "a refused request must not read the cache"
        );
    }
}
