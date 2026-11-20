//! Bounded cache-only mail probes. Authoritative ownership/revisions stay with
//! the caller; these APIs never load or persist durable Store cache entries.
use super::*;

pub const MAX_MAIL_BATCH_KEYS: usize = 64;
const MAX_MAIL_BATCH_KEY_BYTES: usize = 64 * 1024;
const MAX_MAIL_BATCH_BYTES: usize = 8 * 1024 * 1024;

impl Cache {
    /// Probe already validated mail namespaces, positionally. Missing values
    /// are not cached. Redis probes remain sequential: this does not multiply
    /// the existing remote-response allocation with an unbounded MGET.
    pub async fn probe_mail_bytes_batch(
        &self,
        class: CacheClass,
        keys: &[String],
        account: AccountPosture,
        max_value_bytes: usize,
    ) -> CacheResult<PlaintextDerived<Vec<Option<Vec<u8>>>>> {
        if keys.len() > MAX_MAIL_BATCH_KEYS
            || keys.iter().any(|key| key.len() > 4096)
            || keys.iter().map(String::len).sum::<usize>() > MAX_MAIL_BATCH_KEY_BYTES
            || max_value_bytes == 0
            || max_value_bytes > MAIL_ENTRY_BYTES
        {
            return Err(CacheError::Store("mail probe batch limit exceeded".into()));
        }
        let mut output = Vec::with_capacity(keys.len());
        let Some((memory, redis, ttl)) = self.mail_placement(class, account) else {
            return Ok(PlaintextDerived(vec![None; keys.len()]));
        };
        let mut bytes = 0usize;
        for key in keys {
            let nk = namespaced(class, key);
            let mut value = None;
            if memory && let Some(entry) = self.inner.mail_memory.get(&nk).await {
                if entry.bytes.len() <= max_value_bytes {
                    if bytes + entry.bytes.len() > MAX_MAIL_BATCH_BYTES {
                        return Err(CacheError::Store("mail probe byte budget exceeded".into()));
                    }
                    value = Some((*entry.bytes).clone());
                }
            }
            if value.is_none()
                && redis
                && let Some(sealed) = self.redis_get(&nk).await
            {
                if sealed.len() <= max_value_bytes + 44 {
                    if bytes + sealed.len().saturating_sub(44) > MAX_MAIL_BATCH_BYTES {
                        return Err(CacheError::Store("mail probe byte budget exceeded".into()));
                    }
                    if let Some(opened) = self.open_mail_value(&nk, &sealed) {
                        if memory {
                            self.mail_put(&nk, opened.clone(), ttl).await;
                        }
                        value = Some(opened);
                    }
                }
            }
            bytes += value.as_ref().map_or(0, Vec::len);
            if bytes > MAX_MAIL_BATCH_BYTES {
                return Err(CacheError::Store("mail probe byte budget exceeded".into()));
            }
            output.push(value);
        }
        Ok(PlaintextDerived(output))
    }

    /// Populate only configured ephemeral mail tiers after the caller has
    /// validated every source witness. Request-only legacy fallbacks must not
    /// be passed here. Zero-access remains structurally excluded.
    pub async fn insert_mail_bytes(
        &self,
        class: CacheClass,
        key: &str,
        account: AccountPosture,
        value: PlaintextDerived<&[u8]>,
        max_value_bytes: usize,
    ) -> CacheResult<()> {
        if key.len() > 4096 || max_value_bytes == 0 || max_value_bytes > MAIL_ENTRY_BYTES {
            return Err(CacheError::Store("mail insertion limit exceeded".into()));
        }
        let Some((memory, redis, ttl)) = self.mail_placement(class, account) else {
            return Ok(());
        };
        if value.0.len() > max_value_bytes {
            return Ok(());
        }
        let nk = namespaced(class, key);
        if memory {
            self.mail_put(&nk, value.0.to_vec(), ttl).await;
        }
        if redis && let Some(sealed) = self.seal_mail_value(&nk, value.0) {
            self.redis_put(&nk, &sealed, ttl).await;
        }
        Ok(())
    }

    pub(super) fn mail_placement(
        &self,
        class: CacheClass,
        account: AccountPosture,
    ) -> Option<(bool, bool, u64)> {
        if account.is_zero_access()
            || !matches!(class, CacheClass::HeaderWindows | CacheClass::MessageBodies)
        {
            return None;
        }
        let policy = self.policy(class);
        let memory = policy.has(CacheLayer::Memory);
        let redis = policy.has(CacheLayer::Redis) && self.inner.store.is_some();
        let ttl = if policy.ttl_secs == 0 {
            300
        } else {
            policy.ttl_secs.min(3600)
        };
        (memory || redis).then_some((memory, redis, ttl))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn probes_keep_order_duplicates_limits_and_existing_scalar_compatibility() {
        let cache = Cache::default();
        cache
            .get_mail_bytes(
                CacheClass::HeaderWindows,
                "first",
                AccountPosture::Standard,
                || async { Ok(Some(b"one".to_vec())) },
            )
            .await
            .unwrap();
        cache
            .insert_mail_bytes(
                CacheClass::HeaderWindows,
                "second",
                AccountPosture::Standard,
                PlaintextDerived::new(b"two".as_slice()),
                128,
            )
            .await
            .unwrap();
        let keys = ["second", "missing", "first", "second"].map(String::from);
        assert_eq!(
            cache
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &keys,
                    AccountPosture::Standard,
                    128
                )
                .await
                .unwrap()
                .into_inner(),
            vec![
                Some(b"two".to_vec()),
                None,
                Some(b"one".to_vec()),
                Some(b"two".to_vec())
            ]
        );
        let scalar = cache
            .get_mail_bytes(
                CacheClass::HeaderWindows,
                "second",
                AccountPosture::Standard,
                || async { panic!("batch insertion must share the existing scalar namespace") },
            )
            .await
            .unwrap();
        assert_eq!(scalar.into_inner(), Some(b"two".to_vec()));
        assert!(
            cache
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &vec!["x".into(); 65],
                    AccountPosture::Standard,
                    128
                )
                .await
                .is_err()
        );
        assert!(
            cache
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &["x".repeat(4097)],
                    AccountPosture::Standard,
                    128
                )
                .await
                .is_err()
        );
        assert_eq!(
            cache
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &["first".into()],
                    AccountPosture::Standard,
                    2
                )
                .await
                .unwrap()
                .into_inner(),
            vec![None]
        );
    }

    #[tokio::test]
    async fn probes_and_insertion_obey_zero_access_disabled_classes_and_store_exclusion() {
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let cache = Cache::with_store(ScopeMatrix::spec_defaults(), store.clone());
        for (class, posture) in [
            (CacheClass::HeaderWindows, AccountPosture::ZeroAccess),
            (CacheClass::Sessions, AccountPosture::Standard),
        ] {
            cache
                .insert_mail_bytes(
                    class,
                    "excluded",
                    posture,
                    PlaintextDerived::new(b"secret".as_slice()),
                    128,
                )
                .await
                .unwrap();
            assert!(!cache.memory_contains(class, "excluded"));
            assert_eq!(
                cache
                    .probe_mail_bytes_batch(class, &["excluded".into()], posture, 128)
                    .await
                    .unwrap()
                    .into_inner(),
                vec![None]
            );
        }
        cache.apply_override(ClassPolicy {
            class: CacheClass::HeaderWindows,
            layers: vec![CacheLayer::Store],
            ttl_secs: 300,
        });
        cache
            .insert_mail_bytes(
                CacheClass::HeaderWindows,
                "disabled",
                AccountPosture::Standard,
                PlaintextDerived::new(b"value".as_slice()),
                128,
            )
            .await
            .unwrap();
        assert!(!cache.memory_contains(CacheClass::HeaderWindows, "disabled"));
        assert!(
            store
                .get_setting(&store_key(&namespaced(
                    CacheClass::HeaderWindows,
                    "disabled"
                )))
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn batch_insert_preserves_configured_ttl_and_rejects_oversized_values() {
        let cache = Cache::default();
        cache.apply_override(ClassPolicy {
            class: CacheClass::HeaderWindows,
            layers: vec![CacheLayer::Memory],
            ttl_secs: 1,
        });
        cache
            .insert_mail_bytes(
                CacheClass::HeaderWindows,
                "short",
                AccountPosture::Standard,
                PlaintextDerived::new(b"value".as_slice()),
                128,
            )
            .await
            .unwrap();
        cache
            .insert_mail_bytes(
                CacheClass::HeaderWindows,
                "large",
                AccountPosture::Standard,
                PlaintextDerived::new(b"too-large".as_slice()),
                2,
            )
            .await
            .unwrap();
        assert!(!cache.memory_contains(CacheClass::HeaderWindows, "large"));
        assert!(
            cache
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &["short".into()],
                    AccountPosture::Standard,
                    128
                )
                .await
                .unwrap()
                .into_inner()[0]
                .is_some()
        );
        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(
            cache
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &["short".into()],
                    AccountPosture::Standard,
                    128
                )
                .await
                .unwrap()
                .into_inner()[0]
                .is_none()
        );
    }

    #[tokio::test]
    async fn batch_probe_rejects_total_bytes_before_cloning_a_second_large_entry() {
        let cache = Cache::default();
        let bytes = vec![1u8; MAIL_ENTRY_BYTES];
        for key in ["large-one", "large-two"] {
            cache
                .insert_mail_bytes(
                    CacheClass::MessageBodies,
                    key,
                    AccountPosture::Standard,
                    PlaintextDerived::new(bytes.as_slice()),
                    MAIL_ENTRY_BYTES,
                )
                .await
                .unwrap();
        }
        // MessageBodies is Store-only by default; enable memory explicitly,
        // then populate through the same policy-gated path.
        cache.apply_override(ClassPolicy {
            class: CacheClass::MessageBodies,
            layers: vec![CacheLayer::Memory],
            ttl_secs: 300,
        });
        for key in ["large-one", "large-two"] {
            cache
                .insert_mail_bytes(
                    CacheClass::MessageBodies,
                    key,
                    AccountPosture::Standard,
                    PlaintextDerived::new(bytes.as_slice()),
                    MAIL_ENTRY_BYTES,
                )
                .await
                .unwrap();
        }
        assert!(
            cache
                .probe_mail_bytes_batch(
                    CacheClass::MessageBodies,
                    &["large-one".into(), "large-two".into()],
                    AccountPosture::Standard,
                    MAIL_ENTRY_BYTES
                )
                .await
                .is_err()
        );
    }

    /// A real remote cache is required: an ignored test invoked explicitly
    /// without its private fixture URL must fail rather than report a skip.
    #[tokio::test]
    #[ignore = "requires owned Redis/Valkey fixture via MW_TEST_REDIS_URL"]
    async fn live_mail_batch_roundtrip_is_encrypted_scoped_and_expiring() {
        let url = std::env::var("MW_TEST_REDIS_URL")
            .expect("MW_TEST_REDIS_URL must name the owned live fixture");
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let mut matrix = ScopeMatrix::spec_defaults();
        matrix.apply_override(ClassPolicy {
            class: CacheClass::HeaderWindows,
            // No memory: the second instance must actually read Redis.
            // Store here must never activate durable mail-cache fallback.
            layers: vec![CacheLayer::Redis, CacheLayer::Store],
            ttl_secs: 30,
        });
        let config = CacheConfig {
            matrix,
            redis_url: Some(url),
            ..Default::default()
        };
        let first = Cache::connect(config.clone(), Some(store.clone())).await;
        let second = Cache::connect(config.clone(), Some(store.clone())).await;
        assert!(first.inner.redis.is_some(), "live fixture unavailable");
        assert!(second.inner.redis.is_some(), "live fixture unavailable");
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let key = format!("batch-live-{}-{suffix}", std::process::id());
        let denied_key = format!("{key}-denied");
        let plaintext = b"private account-bound header metadata";
        first
            .insert_mail_bytes(
                CacheClass::HeaderWindows,
                &key,
                AccountPosture::Standard,
                PlaintextDerived::new(plaintext.as_slice()),
                128,
            )
            .await
            .unwrap();
        assert!(!first.memory_contains(CacheClass::HeaderWindows, &key));
        let namespace = namespaced(CacheClass::HeaderWindows, &key);
        let raw = first
            .inner
            .redis
            .as_ref()
            .unwrap()
            .get::<Option<Bytes>, _>(&namespace)
            .await
            .unwrap()
            .unwrap();
        assert!(raw.starts_with(b"MWB1"));
        assert!(
            !raw.windows(plaintext.len())
                .any(|window| window == plaintext)
        );
        assert_eq!(
            second
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &[key.clone(), denied_key.clone()],
                    AccountPosture::Standard,
                    128,
                )
                .await
                .unwrap()
                .into_inner(),
            vec![Some(plaintext.to_vec()), None]
        );
        // A different Store encryption key cannot authenticate the same Redis
        // entry; corrupt/foreign sealed values remain ordinary misses.
        let foreign = Cache::connect(
            config,
            Some(
                Store::open_in_memory(mw_store::ServerKey::generate())
                    .await
                    .unwrap(),
            ),
        )
        .await;
        assert!(foreign.inner.redis.is_some(), "live fixture unavailable");
        assert_eq!(
            foreign
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &[key.clone()],
                    AccountPosture::Standard,
                    128,
                )
                .await
                .unwrap()
                .into_inner(),
            vec![None]
        );
        assert_eq!(
            second
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &[key.clone()],
                    AccountPosture::ZeroAccess,
                    128,
                )
                .await
                .unwrap()
                .into_inner(),
            vec![None]
        );
        second
            .insert_mail_bytes(
                CacheClass::HeaderWindows,
                &denied_key,
                AccountPosture::ZeroAccess,
                PlaintextDerived::new(plaintext.as_slice()),
                128,
            )
            .await
            .unwrap();
        assert!(
            first
                .inner
                .redis
                .as_ref()
                .unwrap()
                .get::<Option<Bytes>, _>(namespaced(CacheClass::HeaderWindows, &denied_key))
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            second
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &[key.clone()],
                    AccountPosture::Standard,
                    2,
                )
                .await
                .unwrap()
                .into_inner(),
            vec![None]
        );
        second.apply_override(ClassPolicy {
            class: CacheClass::HeaderWindows,
            layers: vec![CacheLayer::Store],
            ttl_secs: 1,
        });
        assert_eq!(
            second
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &[key.clone()],
                    AccountPosture::Standard,
                    128,
                )
                .await
                .unwrap()
                .into_inner(),
            vec![None]
        );
        second
            .insert_mail_bytes(
                CacheClass::HeaderWindows,
                &denied_key,
                AccountPosture::Standard,
                PlaintextDerived::new(plaintext.as_slice()),
                128,
            )
            .await
            .unwrap();
        for item in [&key, &denied_key] {
            assert!(
                store
                    .get_setting(&store_key(&namespaced(CacheClass::HeaderWindows, item,)))
                    .await
                    .unwrap()
                    .is_none()
            );
        }
        assert!(
            first
                .inner
                .redis
                .as_ref()
                .unwrap()
                .get::<Option<Bytes>, _>(namespaced(CacheClass::HeaderWindows, &denied_key))
                .await
                .unwrap()
                .is_none()
        );
        // Start a distinct short lifetime after all key-isolation checks;
        // fixture startup/schema creation must not consume the TTL assertion.
        first.apply_override(ClassPolicy {
            class: CacheClass::HeaderWindows,
            layers: vec![CacheLayer::Redis],
            ttl_secs: 1,
        });
        first
            .insert_mail_bytes(
                CacheClass::HeaderWindows,
                &key,
                AccountPosture::Standard,
                PlaintextDerived::new(plaintext.as_slice()),
                128,
            )
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert_eq!(
            first
                .probe_mail_bytes_batch(
                    CacheClass::HeaderWindows,
                    &[key.clone()],
                    AccountPosture::Standard,
                    128,
                )
                .await
                .unwrap()
                .into_inner(),
            vec![None]
        );
        assert!(
            first
                .inner
                .redis
                .as_ref()
                .unwrap()
                .get::<Option<Bytes>, _>(&namespace)
                .await
                .unwrap()
                .is_none()
        );
    }
}
